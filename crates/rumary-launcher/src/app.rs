use crate::config::LauncherConfig;
use crate::i18n::Translator;
use crate::models::{
    LaunchCommand, LauncherClient, Profile, Version, VersionJson,
    VersionManifest,
};
use crate::ui::AppWindow;
use slint::ComponentHandle;
use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::rc::Rc;
use std::sync::mpsc;
use std::time::Duration;
use tokio::runtime::Runtime;
use uuid::Uuid;
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use crate::util;

type AppResult<T> = Result<T, Box<dyn Error>>;

pub struct LauncherApp {
    ui: AppWindow,
    _poll_timer: slint::Timer,
}

pub struct AppChannels {
    // files: (mpsc::Sender<Vec<FileEntry>>, mpsc::Receiver<Vec<FileEntry>>),
    pub manifest: (
        mpsc::Sender<VersionManifest>,
        mpsc::Receiver<VersionManifest>,
    ),
    pub  clients: (
        mpsc::Sender<Vec<LauncherClient>>,
        mpsc::Receiver<Vec<LauncherClient>>,
    ),
    pub profiles: (mpsc::Sender<Vec<Profile>>, mpsc::Receiver<Vec<Profile>>),
    pub launch: (mpsc::Sender<LaunchCommand>, mpsc::Receiver<LaunchCommand>),
    pub minecraft: (mpsc::Sender<VersionJson>, mpsc::Receiver<VersionJson>),
    pub read_json: (mpsc::Sender<VersionJson>, mpsc::Receiver<VersionJson>),
    pub status: (mpsc::Sender<String>, mpsc::Receiver<String>),
}

pub struct AppState {
    pub translator: Translator,
    pub config: LauncherConfig,
    pub show_settings: bool,
    pub clients: Vec<LauncherClient>,
    pub profiles: Vec<Profile>,
    pub versions: Vec<Version>,
    pub selected_client: Option<usize>,
    pub selected_profile: Option<usize>,
    pub selected_version: Option<usize>,
    pub status: String,
    pub rt: Runtime,
    pub channels: AppChannels,
    pub reqwest_client: ClientWithMiddleware,
}

impl AppState {
    fn new() -> AppResult<Self> {
        let config = confy::load("rumary-launcher", None).unwrap_or_default();
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(5);
        let reqwest_client = ClientBuilder::new(reqwest::Client::new())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();

        Ok(Self {
            translator: Translator::new("ru"),
            config,
            show_settings: false,
            clients: Vec::new(),
            profiles: Vec::new(),
            versions: Vec::new(),
            selected_client: None,
            selected_profile: None,
            selected_version: None,
            status: "Ready".to_string(),
            rt: Runtime::new()?,
            channels: AppChannels {
                // files: mpsc::channel(),
                clients: mpsc::channel(),
                profiles: mpsc::channel(),
                manifest: mpsc::channel(),
                minecraft: mpsc::channel(),
                launch: mpsc::channel(),
                read_json: mpsc::channel(),
                status: mpsc::channel(),
            },
            reqwest_client,
        })
    }

    pub fn selected_version_name(&self) -> Option<String> {
        let index = self.selected_version?;
        Some(self.versions[index].name.clone())
    }

    pub fn get_libraries_path(&self) -> Option<String> {
        let client_path = self.config.client_path.clone();
        let selected_ver = self.selected_version_name();
        if let Some(s) = selected_ver {
            let path = Path::new(&client_path).join("libraries").join(s);

            Some(path.as_os_str().to_string_lossy().into())
        } else {
            None
        }
    }

    fn get_version(&self) -> Option<Version> {
        let index = self.selected_version?;
        Some(self.versions[index].clone())
    }

    fn save_config(&self) {
        let _ = confy::store("rumary-launcher", None, &self.config);
    }

    fn set_language(&mut self, lang: &str) {
        &self.translator.set_language(lang);
    }

    fn fetch_clients(&self) {
        let api_url = self.config.api_url.clone();
        let tx = self.channels.clients.0.clone();
        self.rt.spawn(async move {
            let client = reqwest::Client::new();
            if let Ok(response) = client.get(format!("{api_url}/clients")).send().await
                && let Ok(clients) = response.json::<Vec<LauncherClient>>().await
            {
                let _ = tx.send(clients);
            }
        });
    }

    fn fetch_profiles(&self, client_id: Uuid) {
        let api_url = self.config.api_url.clone();
        let tx = self.channels.profiles.0.clone();
        self.rt.spawn(async move {
            let client = reqwest::Client::new();
            if let Ok(response) = client
                .get(format!("{api_url}/clients/{client_id}/profiles"))
                .send()
                .await
                && let Ok(profiles) = response.json::<Vec<Profile>>().await
            {
                let _ = tx.send(profiles);
            }
        });
    }

    // fn download_client_files(&self, client_id: Uuid) {
    //     let api_url = self.config.api_url.clone();
    //     let tx = self.channels.files.0.clone();
    //     self.rt.spawn(async move {
    //         let client = reqwest::Client::new();
    //         if let Ok(response) = client
    //             .get(format!("{api_url}/clients/{client_id}/files"))
    //             .send()
    //             .await
    //             && let Ok(files) = response.json::<Vec<FileEntry>>().await
    //         {
    //             let _ = tx.send(files);
    //         }
    //     });
    // }

    fn launch_game(&self) {
        let tx = self.channels.launch.0.clone();

        let root_path = self.config.client_path.clone();
        let version_json = self.get_version().unwrap().version_json.clone();

        if version_json.is_none() {
            println!("version_json is none");
            return;
        }

        // self.fetch_download_version_json(self.versions[index].clone()); - для проверки валидности
        self.rt.spawn(async move {
            let version_json = version_json.unwrap();
            let version = version_json.id.clone();

            let mut jars = Vec::new();
            let lib_path = Path::new(&root_path).join("libraries");
            collect_jars(lib_path.as_path(), &mut jars);

            let client_jar_path = PathBuf::from(&root_path)
                .join("versions")
                .join(&version)
                .join("client.jar")
                .to_string_lossy()
                .to_string();

            let sep = if cfg!(windows) { ";" } else { ":" };
            jars.push(client_jar_path);

            let classpath = jars.join(sep);

            println!("classpath: {classpath}");

            let assets_path = PathBuf::from(&root_path).join("assets").join(&version);
            if !assets_path.as_path().exists()
                && let Err(e) = fs::create_dir_all(&assets_path)
            {
                eprintln!("{e}");
                return;
            };
            let assets_path = assets_path.as_path().to_string_lossy().to_string();

            let game_dir = PathBuf::from(&root_path).join("profiles").join(&version);
            if !game_dir.as_path().exists()
                && let Err(e) = fs::create_dir_all(&game_dir)
            {
                eprintln!("{e}");
                return;
            };
            let game_dir = game_dir.as_path().to_string_lossy().to_string();

            let asset_index = version_json.id;

            let mut game_args: HashMap<String, String> = HashMap::new();
            game_args.insert("username".into(), "Daimon".into());
            game_args.insert("uuid".into(), "00000000-0000-0000-0000-000000000000".into());
            game_args.insert("accessToken".into(), "1234567890abcdef".into());
            game_args.insert("gameDir".into(), game_dir);
            game_args.insert("userType".into(), "msa".into());
            game_args.insert("versionType".into(), "release".into());
            game_args.insert("version".into(), version);
            game_args.insert("assetsDir".into(), assets_path);
            game_args.insert("assetIndex".into(), asset_index);

            let command = LaunchCommand {
                main_class: version_json.main_class,
                jvm_args: vec!["-Xmx2G".into()],
                game_args,
                classpath,
            };

            let _ = tx.send(command);
        });
    }

    fn checking(&self) {
        let tx = self.channels.read_json.0.clone();

        let root_path = self.config.client_path.clone();
        let selected_ver = self.selected_version_name();

        self.rt.spawn(async move {
            let r = match read_version_json(&root_path, selected_ver).await {
                Some(r) => r,
                None => return,
            };

            let _ = tx.send(r);
        });
    }

    fn run_game(&mut self, command: LaunchCommand) {
        self.status = util::t(&self.translator, "launching");

        let root_path = &self.config.client_path;
        let classpath = &command.classpath;

        let mut cmd = Command::new("java");
        cmd.current_dir(root_path);

        for arg in command.jvm_args {
            cmd.arg(arg);
        }

        cmd.arg("-cp");
        cmd.arg(classpath);
        cmd.arg(command.main_class);

        for arg in command.game_args {
            let key = arg.0;
            let value = arg.1;
            let game_arg = format!("--{key}");
            cmd.arg(game_arg);
            cmd.arg(value);
        }

        println!("{:?}", cmd);
        match cmd.spawn() {
            Ok(_) => self.status = util::t(&self.translator, "launched"),
            Err(error) => self.status = format!("Failed to launch game: {error}"),
        }
    }

    fn apply_manifest(&mut self, manifest: VersionManifest) {
        self.versions.clear();
        if let Some(versions) = manifest.versions.as_array() {
            for version in versions {
                if let Some(object) = version.as_object() {
                    let name = object
                        .get("id")
                        .and_then(|value| value.as_str())
                        .unwrap_or_default()
                        .to_string();
                    let url = object
                        .get("url")
                        .and_then(|value| value.as_str())
                        .unwrap_or_default()
                        .to_string();
                    self.versions.push(Version {
                        id: Uuid::new_v4(),
                        name,
                        url,
                        version_json: None,
                    });
                }
            }
        }
        self.selected_version = if self.versions.is_empty() {
            None
        } else {
            Some(0)
        };
    }



    // fn create_needed_dirs(&self) {
    //     let client_path = self.config.client_path.clone();
    //
    //     let dirs = vec!["assets", "libraries", "profiles", "others"];
    //
    //     for dir in dirs {
    //         let path = Path::new(&client_path).join(dir);
    //         if !path.exists() {
    //             fs::create_dir_all(path).unwrap();
    //         }
    //     }
    // }
}

impl LauncherApp {
    pub fn new() -> AppResult<Self> {
        let ui = AppWindow::new()?;
        let state = Rc::new(RefCell::new(AppState::new()?));

        {
            let state_mut = state.borrow_mut();
            state_mut.fetch_clients();
            set_common_ui_values(&ui, &state_mut);
        }

        wire_callbacks(&ui, state.clone());

        let poll_timer = slint::Timer::default();
        let ui_weak = ui.as_weak();
        poll_timer.start(
            slint::TimerMode::Repeated,
            Duration::from_millis(100),
            move || {
                let Some(window) = ui_weak.upgrade() else {
                    return;
                };
                let mut state = state.borrow_mut();
                process_channels(&window, &mut state);
            },
        );

        Ok(Self {
            ui,
            _poll_timer: poll_timer,
        })
    }

    pub fn run(self) -> Result<(), slint::PlatformError> {
        self.ui.run()
    }
}



pub async fn save_version_json(
    client_path: &str,
    selected_ver: Option<String>,
    version_json: &VersionJson,
) {
    if let Some(s) = selected_ver {
        let path = Path::new(&client_path).join("versions").join(s);

        if !path.exists() {
            fs::create_dir_all(&path).unwrap();
        }

        let file_path = path.join("version.json");
        let file = fs::File::create(file_path).unwrap();
        serde_json::to_writer_pretty(file, &version_json).unwrap();
        println!("version.json saved");
    }
}

async fn read_version_json(client_path: &str, selected_ver: Option<String>) -> Option<VersionJson> {
    if let Some(s) = selected_ver {
        let path = Path::new(&client_path)
            .join("versions")
            .join(s)
            .join("version.json");

        if !path.exists() {
            return None;
        }
        println!("{:?}", path);

        let bytes = tokio::fs::read(path).await.unwrap();

        let version_json: VersionJson = serde_json::from_slice(&bytes).unwrap();
        return Some(version_json);
    }
    None
}
fn collect_jars(dir: &Path, jars: &mut Vec<String>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                collect_jars(&path, jars);
            } else if let Some(ext) = path.extension()
                && ext == "jar"
            {
                jars.push(path.to_string_lossy().to_string());
            }
        }
    }
}

fn wire_callbacks(ui: &AppWindow, state: Rc<RefCell<AppState>>) {
    let weak = ui.as_weak();
    ui.on_language_en({
        let state = state.clone();
        move || {
            if let Some(window) = weak.upgrade() {
                let mut state = state.borrow_mut();
                state.set_language("en");
                set_common_ui_values(&window, &state);
            }
        }
    });

    let weak = ui.as_weak();
    ui.on_language_ru({
        let state = state.clone();
        move || {
            if let Some(window) = weak.upgrade() {
                let mut state = state.borrow_mut();
                state.set_language("ru");
                set_common_ui_values(&window, &state);
            }
        }
    });

    let weak = ui.as_weak();
    ui.on_previous_client({
        let state = state.clone();
        move || {
            if let Some(window) = weak.upgrade() {
                let mut state = state.borrow_mut();
                select_previous_client(&mut state);
                set_selection_labels(&window, &state);
            }
        }
    });

    let weak = ui.as_weak();
    ui.on_next_client({
        let state = state.clone();
        move || {
            if let Some(window) = weak.upgrade() {
                let mut state = state.borrow_mut();
                select_next_client(&mut state);
                set_selection_labels(&window, &state);
            }
        }
    });

    let weak = ui.as_weak();
    ui.on_previous_profile({
        let state = state.clone();
        move || {
            if let Some(window) = weak.upgrade() {
                let mut state = state.borrow_mut();
                select_previous_profile(&mut state);
                set_selection_labels(&window, &state);
            }
        }
    });

    let weak = ui.as_weak();
    ui.on_next_profile({
        let state = state.clone();
        move || {
            if let Some(window) = weak.upgrade() {
                let mut state = state.borrow_mut();
                select_next_profile(&mut state);
                set_selection_labels(&window, &state);
            }
        }
    });

    let weak = ui.as_weak();
    ui.on_previous_version({
        let state = state.clone();
        move || {
            if let Some(window) = weak.upgrade() {
                let mut state = state.borrow_mut();
                select_previous_version(&mut state);
                set_selection_labels(&window, &state);
            }
        }
    });

    let weak = ui.as_weak();
    ui.on_next_version({
        let state = state.clone();
        move || {
            if let Some(window) = weak.upgrade() {
                let mut state = state.borrow_mut();
                select_next_version(&mut state);
                set_selection_labels(&window, &state);
            }
        }
    });

    let weak = ui.as_weak();
    ui.on_play({
        let state = state.clone();
        move || {
            if let Some(window) = weak.upgrade() {
                let state = state.borrow_mut();

                state.checking();
                window.set_status_text(state.status.clone().into());
            }
        }
    });

    let weak = ui.as_weak();
    ui.on_toggle_settings({
        let state = state.clone();
        move || {
            if let Some(window) = weak.upgrade() {
                let mut state = state.borrow_mut();
                state.show_settings = !state.show_settings;
                window.set_settings_visible(state.show_settings);
            }
        }
    });

    ui.on_fetch_versions({
        let state = state.clone();
        move || {
            let state = state.borrow();
            state.download_manifest();
        }
    });

    ui.on_download_selected_version({
        let state = state.clone();
        move || {
            let state = state.borrow();
            if let Some(index) = state.selected_version {
                state.fetch_download_version_json(state.versions[index].clone());
            }
        }
    });

    let weak = ui.as_weak();
    ui.on_save_settings(move || {
        if let Some(window) = weak.upgrade() {
            let mut state = state.borrow_mut();
            state.config.api_url = window.get_api_url().to_string();
            state.config.client_path = window.get_client_path().to_string();
            state.config.username = window.get_username().to_string();
            state.save_config();
            state.fetch_clients();
        }
    });
}

fn process_channels(ui: &AppWindow, state: &mut AppState) {
    while let Ok(clients) = state.channels.clients.1.try_recv() {
        state.clients = clients;
        state.selected_client = if state.clients.is_empty() {
            None
        } else {
            Some(0)
        };
        state.profiles.clear();
        state.selected_profile = None;
        if let Some(index) = state.selected_client {
            state.fetch_profiles(state.clients[index].id);
        }
    }

    while let Ok(profiles) = state.channels.profiles.1.try_recv() {
        state.profiles = profiles;
        state.selected_profile = if state.profiles.is_empty() {
            None
        } else {
            Some(0)
        };
    }

    while let Ok(command) = state.channels.launch.1.try_recv() {
        state.run_game(command);
    }

    while let Ok(manifest) = state.channels.manifest.1.try_recv() {
        state.apply_manifest(manifest);
    }

    while let Ok(minecraft) = state.channels.minecraft.1.try_recv() {
        //..тут нужна функция валидности нынешнего майна
        // например: let is_valid = validation(minecraft)

        state.download_minecraft_version(minecraft)
    }

    while let Ok(read_json) = state.channels.read_json.1.try_recv() {
        if state.get_version().is_none() {
            break;
        }

        let index = state.selected_version.unwrap();

        state.versions[index].version_json = Some(read_json);
        state.launch_game();
    }
    while let Ok(status) = state.channels.status.1.try_recv() {
        state.status = status;
    }

    ui.set_status_text(state.status.clone().into());
    set_selection_labels(ui, state);
}


fn set_common_ui_values(ui: &AppWindow, state: &AppState) {
    ui.set_window_title("Rumary Launcher".into());
    ui.set_play_label(util::t(&state.translator, "play").into());
    ui.set_settings_label(util::t(&state.translator, "settings").into());
    ui.set_fetch_versions_label("Get versions".into());
    ui.set_download_version_label("Download selected version".into());
    ui.set_client_label(util::t(&state.translator, "client").into());
    ui.set_profile_label(util::t(&state.translator, "profile").into());
    ui.set_version_label(util::t(&state.translator, "version").into());
    ui.set_api_url_label(util::t(&state.translator, "api_url").into());
    ui.set_client_path_label(util::t(&state.translator, "client_path").into());
    ui.set_username_label(util::t(&state.translator, "username").into());
    ui.set_status_text(state.status.clone().into());
    ui.set_api_url(state.config.api_url.clone().into());
    ui.set_client_path(state.config.client_path.clone().into());
    ui.set_username(state.config.username.clone().into());
    ui.set_settings_visible(state.show_settings);
    set_selection_labels(ui, state);
}

fn set_selection_labels(ui: &AppWindow, state: &AppState) {
    let client = state
        .selected_client
        .and_then(|idx| state.clients.get(idx))
        .map(|client| client.name.clone())
        .unwrap_or_else(|| util::t(&state.translator, "selected_client_name_empty"));
    let profile = state
        .selected_profile
        .and_then(|idx| state.profiles.get(idx))
        .map(|profile| profile.name.clone())
        .unwrap_or_else(|| util::t(&state.translator, "selected_profile_name_empty"));
    let version = state
        .selected_version
        .and_then(|idx| state.versions.get(idx))
        .map(|version| version.name.clone())
        .unwrap_or_else(|| util::t(&state.translator, "selected_version_empty"));
    ui.set_selected_client_name(client.into());
    ui.set_selected_profile_name(profile.into());
    ui.set_selected_version_name(version.into());
}

fn select_previous_client(state: &mut AppState) {
    if state.clients.is_empty() {
        state.selected_client = None;
        return;
    }
    let len = state.clients.len();
    let next = match state.selected_client {
        Some(0) | None => len - 1,
        Some(index) => index.saturating_sub(1),
    };
    state.selected_client = Some(next);
    state.profiles.clear();
    state.selected_profile = None;
    state.fetch_profiles(state.clients[next].id);
}

fn select_next_client(state: &mut AppState) {
    if state.clients.is_empty() {
        state.selected_client = None;
        return;
    }
    let len = state.clients.len();
    let next = match state.selected_client {
        Some(index) => (index + 1) % len,
        None => 0,
    };
    state.selected_client = Some(next);
    state.profiles.clear();
    state.selected_profile = None;
    state.fetch_profiles(state.clients[next].id);
}

fn select_previous_profile(state: &mut AppState) {
    if state.profiles.is_empty() {
        state.selected_profile = None;
        return;
    }
    let len = state.profiles.len();
    let next = match state.selected_profile {
        Some(0) | None => len - 1,
        Some(index) => index.saturating_sub(1),
    };
    state.selected_profile = Some(next);
}

fn select_next_profile(state: &mut AppState) {
    if state.profiles.is_empty() {
        state.selected_profile = None;
        return;
    }
    let len = state.profiles.len();
    let next = match state.selected_profile {
        Some(index) => (index + 1) % len,
        None => 0,
    };
    state.selected_profile = Some(next);
}

fn select_previous_version(state: &mut AppState) {
    if state.versions.is_empty() {
        state.selected_version = None;
        return;
    }
    let len = state.versions.len();
    let next = match state.selected_version {
        Some(0) | None => len - 1,
        Some(index) => index.saturating_sub(1),
    };
    state.selected_version = Some(next);
}

fn select_next_version(state: &mut AppState) {
    if state.versions.is_empty() {
        state.selected_version = None;
        return;
    }
    let len = state.versions.len();
    let next = match state.selected_version {
        Some(index) => (index + 1) % len,
        None => 0,
    };
    state.selected_version = Some(next);
}
