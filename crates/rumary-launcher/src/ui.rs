use crate::app::AppState;
use crate::util;
use crate::validation::ValidationService;
use rumary_dto::domain::launcher::state::OsType;
use rumary_dto::domain::launcher::{ChosenVersion, MinecraftLaunchArgs};
use rumary_dto::dto::api::response::{GetConfigurationResponse, GetInstanceResponse};
use rumary_dto::mojang::dto::response::VersionManifest;
use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::process::Command;
use std::rc::Rc;
use std::sync::Arc;
use uuid::Uuid;

slint::include_modules!();

const BOOTSTRAP_MAIN_CLASS: &str = "net.daimon.bootstrap.Main";

impl AppState {
    fn set_language(&mut self, lang: &str) {
        self.translator.set_language(lang);
    }

    pub(crate) fn fetch_clients(&self) {
        // либо удалить, либо переделать, ибо у нас нет больше моих "Клиентов"
        let tx = self.channels.clients.0.clone();

        let reqwest_client = self.reqwest_client.clone();
        let api_url = self.config.api_url.clone();
        let url = format!("{api_url}/clients");

        self.rt.spawn(async move {
            if let Ok(response) = util::get_response(&reqwest_client, &url).await
                && let Ok(clients) = response.json::<Vec<GetInstanceResponse>>().await
            {
                let _ = tx.send(clients);
            }
        });
    }

    fn fetch_profiles(&self) {
        let tx = self.channels.profiles.0.clone();

        let reqwest_client = self.reqwest_client.clone();
        let api_url = self.config.api_url.clone();
        let url = format!("{api_url}/profiles");

        self.rt.spawn(async move {
            if let Ok(response) = util::get_response(&reqwest_client, &url).await
                && let Ok(profiles) = response.json::<Vec<GetConfigurationResponse>>().await
            {
                let _ = tx.send(profiles);
            }
        });
    }

    fn prelude_play(&self) {
        let tx = self.channels.validation_service.0.clone();
        let id = self.get_selected_version_name().clone();

        let client = self.reqwest_client.clone();
        let root_path = self.config.root_path.clone();

        self.rt.spawn(async move {
            if let Some(id) = id {
                let url = Self::get_version_url(&client, &id).await.unwrap();
                let validation_service = ValidationService::new(&client, url, &root_path).await;

                if let Ok(v) = validation_service {
                    let _ = tx.send(v);
                }
            }
        });
    }

    fn launch_game(&self, validation_service: ValidationService) {
        let tx = self.channels.launch.0.clone();

        let root_path = self.config.root_path.clone();
        let os = self.os;

        self.rt.spawn(async move {
            if validation_service
                .validate_version()
                .await
                .unwrap_or_else(|e| {
                    eprintln!("validate checking has found error: {e}");
                    false
                })
            {
                let version_json = validation_service.version_json.clone();
                let version = version_json.id.clone();

                let mut jars = Vec::new();
                // jars.push("rumary-bootstrap-1.0.jar".into());
                jars.push("rumary-bootstrap-1.0-without-token.jar".into());

                let lib_path = util::get_libraries_path(&root_path, &version);
                util::collect_jars(lib_path.as_path(), &mut jars);

                let client_jar_path = util::minecraft_jar_path(&root_path, &version)
                    .join("client.jar")
                    .to_string_lossy()
                    .to_string();

                let sep = if os == OsType::Windows { ";" } else { ":" };

                jars.push(client_jar_path);

                let classpath = jars.join(sep);

                let assets_path = util::assets_path(&root_path, &version);
                util::verify_path(assets_path.as_path()).await.unwrap();

                let assets_path = assets_path.as_path().to_string_lossy().to_string();

                let game_dir = util::game_path(&root_path, &version);
                util::verify_path(game_dir.as_path()).await.unwrap();

                let game_dir = game_dir.as_path().to_string_lossy().to_string();

                let asset_index = version_json.asset_index.id.clone();

                // let game_args = version_json.clone().arguments.clone().unwrap().game.unwrap();
                // do to struct with that var-s

                let mut game_args: HashMap<String, String> = HashMap::new();
                game_args.insert("username".into(), "Daimon".into());
                game_args.insert("uuid".into(), "00000000-0000-0000-0000-000000000000".into());
                game_args.insert("accessToken".into(), "1234567890abcdef".into());
                game_args.insert("gameDir".into(), game_dir);
                game_args.insert("versionType".into(), "release".into());
                game_args.insert("version".into(), version);
                game_args.insert("assetsDir".into(), assets_path);
                game_args.insert("assetIndex".into(), asset_index);

                let command = MinecraftLaunchArgs {
                    // main_class: version_json.main_class.clone(),
                    main_class: BOOTSTRAP_MAIN_CLASS.to_string(),
                    jvm_args: vec!["-Xmx2G".into()],
                    game_args,
                    classpath,
                };

                let _ = tx.send(command);
            }
        });
    }

    fn run_game(&mut self, command: MinecraftLaunchArgs) {
        self.status = util::t(&self.translator, "launching");

        let root_path = &self.config.root_path;
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
        let versions = manifest.versions;
        for version in versions {
            let name = version.id;
            let url = version.url;

            let launcher_version = ChosenVersion {
                id: Uuid::new_v4(),
                name,
                url,
                // version_json: None,
            };

            self.versions.push(launcher_version);
        }
        self.selected_version = if self.versions.is_empty() {
            None
        } else {
            Some(0)
        };
    }
}

pub fn wire_callbacks(ui: &AppWindow, state: Rc<RefCell<AppState>>) {
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

                state.prelude_play();
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
            state.config.root_path = window.get_client_path().to_string();
            state.config.username = window.get_username().to_string();
            state.save_config();
            state.fetch_clients();
        }
    });
}

pub fn process_channels(ui: &AppWindow, state: &mut AppState) {
    while let Ok(clients) = state.channels.clients.1.try_recv() {
        state.clients = clients;
        state.selected_client = if state.clients.is_empty() {
            None
        } else {
            Some(0)
        };
        state.profiles.clear();
        state.selected_profile = None;
        state.fetch_profiles();
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
        // let client = state.reqwest_client.clone();
        // let root_path = state.config.root_path.clone();
        // let selected_version = state.selected_version.unwrap_or(1);
        //  let version_json_url = state.versions[selected_version].url;
        //
        // let validation = ValidationService::new(&client, "https://a.com", &root_path);

        let minecraft = Arc::new(minecraft);
        state.download_minecraft_version(minecraft.clone());
    }

    while let Ok(service) = state.channels.validation_service.1.try_recv() {
        if state.get_version().is_none() {
            break;
        }
        state.launch_game(service);
    }
    while let Ok(status) = state.channels.status.1.try_recv() {
        state.status = status;
    }

    ui.set_status_text(state.status.clone().into());
    set_selection_labels(ui, state);
}

pub fn set_common_ui_values(ui: &AppWindow, state: &AppState) {
    ui.set_window_title("Rumary Launcher".into());

    ui.set_play_label(util::t(&state.translator, "play").into());
    ui.set_settings_label(util::t(&state.translator, "settings").into());
    ui.set_fetch_versions_label(util::t(&state.translator, "get_versions").into());
    ui.set_download_version_label(util::t(&state.translator, "download_selected_version").into());
    ui.set_client_label(util::t(&state.translator, "client").into());
    ui.set_profile_label(util::t(&state.translator, "profile").into());
    ui.set_version_label(util::t(&state.translator, "version").into());
    ui.set_api_url_label(util::t(&state.translator, "api_url").into());
    ui.set_client_path_label(util::t(&state.translator, "client_path").into());
    ui.set_username_label(util::t(&state.translator, "username").into());

    ui.set_status_text(state.status.clone().into());
    ui.set_api_url(state.config.api_url.clone().into());
    ui.set_client_path(state.config.root_path.clone().into());
    ui.set_username(state.config.username.clone().into());
    ui.set_settings_visible(state.show_settings);

    set_selection_labels(ui, state);
}

fn set_selection_labels(ui: &AppWindow, state: &AppState) {
    let client = state
        .selected_client
        .and_then(|idx| state.clients.get(idx))
        .map(|client| client.display_name.clone())
        .unwrap_or_else(|| util::t(&state.translator, "selected_client_name_empty"));
    let profile = state
        .selected_profile
        .and_then(|idx| state.profiles.get(idx))
        .map(|profile| profile.display_name.clone())
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

// Бесполезная функция в будущем
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
    state.fetch_profiles();
}

// Бесполезная функция в будущем
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
    state.fetch_profiles();
}

// Бесполезная функция в будущем
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

// Бесполезная функция в будущем
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

// Бесполезная функция в будущем
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
// Бесполезная функция в будущем
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
