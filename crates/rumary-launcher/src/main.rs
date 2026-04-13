use eframe::egui;
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc;
use sha2::{Sha256, Digest};
use tokio::runtime::Runtime;
use uuid::Uuid;

#[derive(RustEmbed)]
#[folder = "translations/"]
struct Asset;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Client {
    id: Uuid,
    name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Profile {
    id: Uuid,
    name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct FileEntry {
    path: String,
    hash: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct LaunchCommand {
    main_class: String,
    jvm_args: Vec<String>,
    game_args: Vec<String>,
    classpath: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MyConfig {
    version: u8,
    client_path: String,
    api_url: String,
    username: String,
    access_token: String,
    uuid: String,
}

impl Default for MyConfig {
    fn default() -> Self {
        Self {
            version: 1,
            client_path: get_default_client_path(),
            api_url: "http://127.0.0.1:3000".to_string(),
            username: "Player".to_string(),
            access_token: "token".to_string(),
            uuid: Uuid::new_v4().to_string(),
        }
    }
}

fn get_default_client_path() -> String {
    if cfg!(windows) {
        format!("{}/.rumary", std::env::var("APPDATA").unwrap())
    } else {
        format!("{}/.rumary", std::env::var("HOME").unwrap())
    }
}

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Rumary Launcher",
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::default()))),
    )
}

struct MyApp {
    translations: HashMap<String, String>,
    current_lang: String,
    config: MyConfig,
    show_settings: bool,
    clients: Vec<Client>,
    selected_client: Option<Uuid>,
    profiles: Vec<Profile>,
    selected_profile: Option<Uuid>,
    rt: Runtime,
    clients_channel: (
        mpsc::Sender<Vec<Client>>,
        mpsc::Receiver<Vec<Client>>,
    ),
    profiles_channel: (
        mpsc::Sender<Vec<Profile>>,
        mpsc::Receiver<Vec<Profile>>,
    ),
    files_channel: (
        mpsc::Sender<Vec<FileEntry>>,
        mpsc::Receiver<Vec<FileEntry>>,
    ),
    launch_channel: (
        mpsc::Sender<LaunchCommand>,
        mpsc::Receiver<LaunchCommand>,
    ),
    status: String,
}

impl Default for MyApp {
    fn default() -> Self {
        let mut s = Self {
            translations: HashMap::new(),
            current_lang: "ru".to_owned(),
            config: confy::load("rumary-launcher", None).unwrap_or_default(),
            show_settings: false,
            clients: vec![],
            selected_client: None,
            profiles: vec![],
            selected_profile: None,
            rt: Runtime::new().unwrap(),
            clients_channel: mpsc::channel(),
            profiles_channel: mpsc::channel(),
            files_channel: mpsc::channel(),
            launch_channel: mpsc::channel(),
            status: "Ready".to_string(),
        };
        s.load_translations();
        s.fetch_clients();
        s
    }
}

impl MyApp {
    fn load_translations(&mut self) {
        let lang = &self.current_lang;
        let filename = format!("{}.json", lang);
        if let Some(file) = Asset::get(&filename) {
            let json: Value = serde_json::from_slice(&file.data).unwrap();
            if let Value::Object(map) = json {
                self.translations = map
                    .into_iter()
                    .map(|(k, v)| (k, v.as_str().unwrap_or("").to_owned()))
                    .collect();
            }
        }
    }

    fn t<'a>(&'a self, key: &'a str) -> &'a str {
        self.translations.get(key).map(|s| s.as_str()).unwrap_or(key)
    }

    fn save_config(&self) {
        confy::store("rumary-launcher", None, &self.config).unwrap();
    }

    fn launch_game(&mut self) {
        if let Some(client_id) = self.selected_client {
            self.status = self.t("Checking files...").to_string();
            self.download_client_files(client_id);
        } else {
            self.status = self.t("Client not selected!").to_string();
        }
    }

    fn fetch_clients(&self) {
        let api_url = self.config.api_url.clone();
        let tx = self.clients_channel.0.clone();
        self.rt.spawn(async move {
            let client = reqwest::Client::new();
            if let Ok(res) = client
                .get(format!("{}/clients", api_url))
                .send()
                .await
                .unwrap()
                .json::<Vec<Client>>()
                .await
            {
                tx.send(res).unwrap();
            }
        });
    }

    fn fetch_profiles(&self, client_id: Uuid) {
        let api_url = self.config.api_url.clone();
        let tx = self.profiles_channel.0.clone();
        self.rt.spawn(async move {
            let client = reqwest::Client::new();
            if let Ok(res) = client
                .get(format!("{}/clients/{}/profiles", api_url, client_id))
                .send()
                .await
                .unwrap()
                .json::<Vec<Profile>>()
                .await
            {
                tx.send(res).unwrap();
            }
        });
    }

    fn download_client_files(&self, client_id: Uuid) {
        let api_url = self.config.api_url.clone();
        let tx = self.files_channel.0.clone();
        self.rt.spawn(async move {
            let client = reqwest::Client::new();
            if let Ok(res) = client
                .get(format!("{}/clients/{}/files", api_url, client_id))
                .send()
                .await
                .unwrap()
                .json::<Vec<FileEntry>>()
                .await
            {
                tx.send(res).unwrap();
            }
        });
    }

    fn process_files(&mut self, files: Vec<FileEntry>) {
        let client_path = self.config.client_path.clone();
        let api_url = self.config.api_url.clone();
        let client_id = self.selected_client.unwrap();
        let launch_tx = self.launch_channel.0.clone();

        self.rt.spawn(async move {
            for file in files {
                let local_path = Path::new(&client_path).join(&file.path);
                let mut needs_download = true;
                if local_path.exists() {
                    if let Ok(file_content) = fs::read(&local_path) {
                        let mut hasher = Sha256::new();
                        hasher.update(&file_content);
                        let hash = format!("{:x}", hasher.finalize());
                        if hash == file.hash {
                            needs_download = false;
                        }
                    }
                }

                if needs_download {
                    println!("Downloading: {:?}", local_path);
                    download_file(&api_url, &file, &client_path).await;
                }
            }

            println!("File check finished. Fetching launch command...");
            fetch_launch_command(&api_url, client_id, launch_tx).await;
        });
    }

    fn run_game(&mut self, command: LaunchCommand) {
        self.status = self.t("Launching game...").to_string();
        let client_path = self.config.client_path.clone();

        let classpath = command.classpath.join(if cfg!(windows) { ";" } else { ":" });

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

        println!("Running command: {:?}", cmd);

        match cmd.spawn() {
            Ok(_) => self.status = self.t("launched").to_string(),
            Err(e) => self.status = format!("Failed to launch game: {}", e),
        }
    }
}

async fn download_file(api_url: &str, file: &FileEntry, client_path: &str) {
    let local_path = Path::new(client_path).join(&file.path);
    if let Some(parent) = local_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let url = format!("{}/files/{}", api_url, file.path);
    if let Ok(response) = reqwest::get(&url).await {
        if let Ok(mut dest) = fs::File::create(&local_path) {
            if let Ok(content) = response.bytes().await {
                dest.write_all(&content).unwrap();
                println!("File downloaded: {:?}", local_path);
            }
        }
    }
}

async fn fetch_launch_command(api_url: &str, client_id: Uuid, tx: mpsc::Sender<LaunchCommand>) {
    let client = reqwest::Client::new();
    if let Ok(res) = client
        .get(format!("{}/clients/{}/launch_command", api_url, client_id))
        .send()
        .await
        .unwrap()
        .json::<LaunchCommand>()
        .await
    {
        tx.send(res).unwrap();
    }
}

fn replace_placeholders(arg: &str, config: &MyConfig, client_path: &str) -> String {
    arg.replace("${auth_player_name}", &config.username)
        .replace("${auth_uuid}", &config.uuid)
        .replace("${auth_access_token}", &config.access_token)
        .replace("${game_directory}", client_path)
        // Add other placeholders as needed
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(clients) = self.clients_channel.1.try_recv() {
            self.clients = clients;
            self.profiles.clear();
            self.selected_profile = None;
        }
        if let Ok(profiles) = self.profiles_channel.1.try_recv() {
            self.profiles = profiles;
        }
        if let Ok(files) = self.files_channel.1.try_recv() {
            self.process_files(files);
        }
        if let Ok(command) = self.launch_channel.1.try_recv() {
            self.run_game(command);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Rumary Launcher");

            ui.horizontal(|ui| {
                if ui.button(self.t("eng")).clicked() {
                    self.current_lang = "en".to_owned();
                    self.load_translations();
                }
                if ui.button(self.t("rus")).clicked() {
                    self.current_lang = "ru".to_owned();
                    self.load_translations();
                }
            });

            let selected_client_name = self
                .selected_client
                .and_then(|id| self.clients.iter().find(|c| c.id == id))
                .map(|c| c.name.clone())
                .unwrap_or_else(|| self.t("selected_client_name_empty").to_string());

            egui::ComboBox::from_label(self.t("client"))
                .selected_text(selected_client_name)
                .show_ui(ui, |ui| {
                    for client in &self.clients {
                        if ui
                            .selectable_value(&mut self.selected_client, Some(client.id), &client.name)
                            .clicked()
                        {
                            self.fetch_profiles(client.id);
                        }
                    }
                });

            let selected_profile_name = self
                .selected_profile
                .and_then(|id| self.profiles.iter().find(|p| p.id == id))
                .map(|p| p.name.clone())
                .unwrap_or_else(|| self.t("selected_profile_name_empty").to_string());

            egui::ComboBox::from_label(self.t("Profile"))
                .selected_text(selected_profile_name)
                .show_ui(ui, |ui| {
                    for profile in &self.profiles {
                        ui.selectable_value(
                            &mut self.selected_profile,
                            Some(profile.id),
                            &profile.name,
                        );
                    }
                });

            if ui.button(self.t("play")).clicked() {
                self.launch_game();
            }

            ui.label(&self.status);

            if ui.button(self.t("settings")).clicked() {
                self.show_settings = !self.show_settings;
            }

            if self.show_settings {
                ui.separator();
                ui.label(self.t("API URL:"));
                if ui.text_edit_singleline(&mut self.config.api_url).changed() {
                    self.save_config();
                    self.fetch_clients();
                }
                ui.label(self.t("client_path"));
                if ui.text_edit_singleline(&mut self.config.client_path).changed() {
                    self.save_config();
                }
                ui.label(self.t("Username:"));
                if ui.text_edit_singleline(&mut self.config.username).changed() {
                    self.save_config();
                }
            }
        });
    }
}
