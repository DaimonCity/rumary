use crate::config::LauncherConfig;
use crate::i18n::Translator;
use crate::models::{FileEntry, LaunchCommand, LauncherClient, Profile, Version, VersionJson, VersionManifest};
use crate::ui::AppWindow;
use sha2::{Digest, Sha256};
use std::cell::RefCell;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::rc::Rc;
use std::sync::mpsc;
use std::time::Duration;
use slint::ComponentHandle;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::runtime::Runtime;
use uuid::Uuid;
use log::log;

type AppResult<T> = Result<T, Box<dyn std::error::Error>>;

pub struct LauncherApp {
    ui: AppWindow,
    _poll_timer: slint::Timer,
}

struct AppChannels {
    clients: (mpsc::Sender<Vec<LauncherClient>>, mpsc::Receiver<Vec<LauncherClient>>),
    profiles: (mpsc::Sender<Vec<Profile>>, mpsc::Receiver<Vec<Profile>>),
    files: (mpsc::Sender<Vec<FileEntry>>, mpsc::Receiver<Vec<FileEntry>>),
    manifest: (mpsc::Sender<VersionManifest>, mpsc::Receiver<VersionManifest>),
    launch: (mpsc::Sender<LaunchCommand>, mpsc::Receiver<LaunchCommand>),
}

struct AppState {
    translator: Translator,
    config: LauncherConfig,
    show_settings: bool,
    clients: Vec<LauncherClient>,
    profiles: Vec<Profile>,
    versions: Vec<Version>,
    selected_client: Option<usize>,
    selected_profile: Option<usize>,
    selected_version: Option<usize>,
    status: String,
    rt: Runtime,
    channels: AppChannels,
}

impl AppState {
    fn new() -> AppResult<Self> {
        let config = confy::load("rumary-launcher", None).unwrap_or_default();
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
                clients: mpsc::channel(),
                profiles: mpsc::channel(),
                files: mpsc::channel(),
                manifest: mpsc::channel(),
                launch: mpsc::channel(),
            },
        })
    }

    fn save_config(&self) {
        let _ = confy::store("rumary-launcher", None, &self.config);
    }

    fn t(&self, key: &str) -> String {
        self.translator.t(key)
    }

    fn set_language(&mut self, lang: &str) {
        self.translator.set_language(lang);
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

    fn download_client_files(&self, client_id: Uuid) {
        let api_url = self.config.api_url.clone();
        let tx = self.channels.files.0.clone();
        self.rt.spawn(async move {
            let client = reqwest::Client::new();
            if let Ok(response) = client
                .get(format!("{api_url}/clients/{client_id}/files"))
                .send()
                .await
                && let Ok(files) = response.json::<Vec<FileEntry>>().await
            {
                let _ = tx.send(files);
            }
        });
    }

    fn download_manifest(&self) {
        let tx = self.channels.manifest.0.clone();
        self.rt.spawn(async move {
            let result = reqwest::Client::new()
                .get("https://launchermeta.mojang.com/mc/game/version_manifest.json")
                .send()
                .await;
            if let Ok(response) = result
                && let Ok(manifest) = response.json::<VersionManifest>().await
            {
                let _ = tx.send(manifest);
            }
        });
    }

    fn download_minecraft_jar(&self, version: &Version) {
        println!("ФУНКЦИЯ ВООБЩЕ ВЫЗВАЛАСЬ");


        let url = version.url.clone();
        let version_name = version.name.clone();
        let client_path = self.config.client_path.clone();

        println!("{}", &version_name);
        println!("{}", &client_path);

        self.rt.spawn(async move {
            let version_json = reqwest::Client::new().get(url).send().await;


            let response = match version_json {
                Ok(response) => response,
                Err(e) => {
                    println!("{}", e);
                    return;
                },
            };

            let json = match response.json::<VersionJson>().await {
                Ok(json) => json,
                Err(e) => {
                    println!("{}", e);
                    return;
                },
            };

            let client_download = reqwest::Client::new()
                .get(json.downloads.client.url)
                .send()
                .await;

            let client_download = match client_download {
                Ok(client) => client,
                Err(e) => {
                    println!("{}", e);
                    return;
                },
            };

            let bytes = match client_download.bytes().await {
                Ok(byte) => byte,
                Err(e) => {
                    println!("{}", e);
                    return;
                }
            };

            let local_path = Path::new(&client_path)
                .join("assets/versions")
                .join(&version_name);

            if !local_path.exists() {
                let _ = fs::create_dir_all(&local_path);
            }

            let file_name = format!("{}.jar", version_name.replace('.', "_"));
            println!("{}", &file_name);
            let full_path = local_path.join(file_name);
            println!("{:?}", &full_path);
            match File::create(full_path).await {
                Ok(mut file) => {let _ = file.write_all(&bytes).await;}
                Err(e) => {
                    println!("{}", e);
                    return;
                },
            };
        });
    }

    fn launch_game(&mut self) {
        if let Some(index) = self.selected_client {
            self.status = self.t("checking_files");
            let client_id = self.clients[index].id;
            self.download_client_files(client_id);
        } else {
            self.status = self.t("client_not_selected");
        }
    }

    fn process_files(&self, files: Vec<FileEntry>) {
        let client_path = self.config.client_path.clone();
        let api_url = self.config.api_url.clone();
        let Some(client_index) = self.selected_client else {
            return;
        };
        let client_id = self.clients[client_index].id;
        let launch_tx = self.channels.launch.0.clone();

        self.rt.spawn(async move {
            for file in files {
                let local_path = Path::new(&client_path).join(&file.path);
                let mut needs_download = true;

                if local_path.exists() && let Ok(file_content) = fs::read(&local_path) {
                    let mut hasher = Sha256::new();
                    hasher.update(&file_content);
                    let hash = format!("{:x}", hasher.finalize());
                    if hash == file.hash {
                        needs_download = false;
                    }
                }

                if needs_download {
                    download_file(&api_url, &file, &client_path).await;
                }
            }

            fetch_launch_command(&api_url, client_id, launch_tx).await;
        });
    }

    fn run_game(&mut self, command: LaunchCommand) {
        self.status = self.t("launching");
        let client_path = self.config.client_path.clone();
        let classpath = command
            .classpath
            .join(if cfg!(windows) { ";" } else { ":" });

        let mut cmd = Command::new("java");
        cmd.current_dir(&client_path);
        cmd.arg("-cp").arg(classpath);

        for arg in command.jvm_args {
            cmd.arg(replace_placeholders(&arg, &self.config, &client_path));
        }

        cmd.arg(command.main_class);

        for arg in command.game_args {
            cmd.arg(replace_placeholders(&arg, &self.config, &client_path));
        }

        match cmd.spawn() {
            Ok(_) => self.status = self.t("launched"),
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
                    });
                }
            }
        }
        self.selected_version = if self.versions.is_empty() { None } else { Some(0) };
    }
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
        poll_timer.start(slint::TimerMode::Repeated, Duration::from_millis(100), move || {
            let Some(window) = ui_weak.upgrade() else {
                return;
            };
            let mut state = state.borrow_mut();
            process_channels(&window, &mut state);
        });

        Ok(Self {
            ui,
            _poll_timer: poll_timer,
        })
    }

    pub fn run(self) -> Result<(), slint::PlatformError> {
        self.ui.run()
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
                let mut state = state.borrow_mut();
                state.launch_game();
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
                state.download_minecraft_jar(&state.versions[index]);
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
        state.selected_client = if state.clients.is_empty() { None } else { Some(0) };
        state.profiles.clear();
        state.selected_profile = None;
        if let Some(index) = state.selected_client {
            state.fetch_profiles(state.clients[index].id);
        }
    }

    while let Ok(profiles) = state.channels.profiles.1.try_recv() {
        state.profiles = profiles;
        state.selected_profile = if state.profiles.is_empty() { None } else { Some(0) };
    }

    while let Ok(files) = state.channels.files.1.try_recv() {
        state.process_files(files);
    }

    while let Ok(command) = state.channels.launch.1.try_recv() {
        state.run_game(command);
    }

    while let Ok(manifest) = state.channels.manifest.1.try_recv() {
        state.apply_manifest(manifest);
    }

    ui.set_status_text(state.status.clone().into());
    set_selection_labels(ui, state);
}

fn set_common_ui_values(ui: &AppWindow, state: &AppState) {
    ui.set_window_title("Rumary Launcher".into());
    ui.set_play_label(state.t("play").into());
    ui.set_settings_label(state.t("settings").into());
    ui.set_fetch_versions_label("Get versions".into());
    ui.set_download_version_label("Download selected version".into());
    ui.set_client_label(state.t("client").into());
    ui.set_profile_label(state.t("profile").into());
    ui.set_version_label(state.t("version").into());
    ui.set_api_url_label(state.t("api_url").into());
    ui.set_client_path_label(state.t("client_path").into());
    ui.set_username_label(state.t("username").into());
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
        .unwrap_or_else(|| state.t("selected_client_name_empty"));
    let profile = state
        .selected_profile
        .and_then(|idx| state.profiles.get(idx))
        .map(|profile| profile.name.clone())
        .unwrap_or_else(|| state.t("selected_profile_name_empty"));
    let version = state
        .selected_version
        .and_then(|idx| state.versions.get(idx))
        .map(|version| version.name.clone())
        .unwrap_or_else(|| state.t("selected_version_empty"));
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

async fn download_file(api_url: &str, file: &FileEntry, client_path: &str) {
    let local_path = Path::new(client_path).join(&file.path);
    if let Some(parent) = local_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let url = format!("{api_url}/files/{}", file.path);
    if let Ok(response) = reqwest::get(&url).await
        && let Ok(mut dest) = fs::File::create(&local_path)
        && let Ok(content) = response.bytes().await
    {
        let _ = dest.write_all(&content);
    }
}

async fn fetch_launch_command(api_url: &str, client_id: Uuid, tx: mpsc::Sender<LaunchCommand>) {
    let client = reqwest::Client::new();
    if let Ok(response) = client
        .get(format!("{api_url}/clients/{client_id}/launch_command"))
        .send()
        .await
        && let Ok(command) = response.json::<LaunchCommand>().await
    {
        let _ = tx.send(command);
    }
}

fn replace_placeholders(arg: &str, config: &LauncherConfig, client_path: &str) -> String {
    arg.replace("${auth_player_name}", &config.username)
        .replace("${auth_uuid}", &config.uuid)
        .replace("${auth_access_token}", &config.access_token)
        .replace("${game_directory}", client_path)
}
