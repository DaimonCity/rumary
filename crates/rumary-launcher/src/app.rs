use crate::config::LauncherConfig;
use crate::download::MANIFEST_URL;
use crate::i18n::Translator;
use rumary_dto::domain::launcher::ChosenVersion;
use crate::result::AppResult;
use crate::ui::AppWindow;
use crate::validation::ValidationService;
use crate::{ui, util};
use reqwest::IntoUrl;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use rumary_dto::mojang::dto::response::{Version, VersionJson, VersionManifest};
use slint::ComponentHandle;
use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;
use std::sync::mpsc;
use std::time::Duration;
use tokio::runtime::Runtime;
use rumary_dto::domain::launcher::MinecraftLaunchArgs;
use rumary_dto::dto::api::response::LauncherClientDto;
use rumary_dto::dto::api::response::ProfileDto;


// channels for json
// pub struct JsonChannels
pub struct AppChannels {
    // files: (mpsc::Sender<Vec<FileEntry>>, mpsc::Receiver<Vec<FileEntry>>),
    pub manifest: (
        mpsc::Sender<VersionManifest>,
        mpsc::Receiver<VersionManifest>,
    ),
    pub clients: (
        mpsc::Sender<Vec<LauncherClientDto>>,
        mpsc::Receiver<Vec<LauncherClientDto>>,
    ),
    pub profiles: (mpsc::Sender<Vec<ProfileDto>>, mpsc::Receiver<Vec<ProfileDto>>),
    pub launch: (mpsc::Sender<MinecraftLaunchArgs>, mpsc::Receiver<MinecraftLaunchArgs>),
    pub minecraft: (mpsc::Sender<VersionJson>, mpsc::Receiver<VersionJson>),
    pub validation_service: (
        mpsc::Sender<ValidationService>,
        mpsc::Receiver<ValidationService>,
    ),
    pub status: (mpsc::Sender<String>, mpsc::Receiver<String>),
}

pub struct AppState {
    pub translator: Translator,
    pub config: LauncherConfig,
    pub show_settings: bool,
    pub clients: Vec<LauncherClientDto>,
    pub profiles: Vec<ProfileDto>,
    pub versions: Vec<ChosenVersion>,
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
                validation_service: mpsc::channel(),
                status: mpsc::channel(),
            },
            reqwest_client,
        })
    }

    pub(crate) fn get_selected_version_name(&self) -> Option<String> {
        let index = self.selected_version?;
        Some(self.versions[index].name.clone())
    }

    pub(crate) fn get_version(&self) -> Option<ChosenVersion> {
        let index = self.selected_version?;
        Some(self.versions[index].clone())
    }

    pub(crate) fn save_config(&self) {
        let _ = confy::store("rumary-launcher", None, &self.config);
    }

    pub(crate) async fn get_version_url(
        reqwest_client: &ClientWithMiddleware,
        version_id: &str,
    ) -> Result<impl IntoUrl, Box<dyn Error + Send + Sync>> {
        let manifest = Self::get_manifest(reqwest_client).await?;
        let version = Version::get_version(&manifest, version_id).await?;
        let url = version.url.clone();
        Ok(url)
    }

    async fn get_manifest(
        reqwest_client: &ClientWithMiddleware,
    ) -> Result<VersionManifest, Box<dyn Error + Send + Sync>> {
        Ok(util::get_response(reqwest_client, MANIFEST_URL)
            .await?
            .json::<VersionManifest>()
            .await?)
    }
}

pub struct LauncherApp {
    ui: AppWindow,
    _poll_timer: slint::Timer,
}

impl LauncherApp {
    pub async fn new() -> AppResult<Self> {
        let ui = AppWindow::new()?;
        let state = Rc::new(RefCell::new(AppState::new()?));

        {
            let state_mut = state.borrow_mut();
            state_mut.fetch_clients();
            ui::set_common_ui_values(&ui, &state_mut);
        }

        ui::wire_callbacks(&ui, state.clone());

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
                ui::process_channels(&window, &mut state);
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
