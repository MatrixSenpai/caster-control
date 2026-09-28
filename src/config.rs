use std::sync::Arc;
use anyhow::{Result, Context as _};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

pub struct ObsData {
    pub scene: &'static str,
    pub name: &'static str,
    pub cam: &'static str,
    pub audio_scene: &'static str,
    pub audio: &'static str,
}

pub const LOGO: &'static str = "TLS Logo";
pub const LEFT: ObsData = ObsData {
    scene: "Left Caster",
    name: "Left Caster Name",
    cam: "Left Caster Cam",
    audio_scene: "Left Caster Audio",
    audio: "Left Caster Audio VDO"
};
pub const RIGHT: ObsData = ObsData {
    scene: "Right Caster",
    name: "Right Caster Name",
    cam: "Right Caster Cam",
    audio_scene: "Right Caster Audio",
    audio: "Right Caster Audio VDO"
};

#[derive(Clone, Debug, Default)]
pub struct AppConfig {
    pub local: LocalConfig,
    pub live: Arc<Mutex<LiveConfig>>,
}

impl AppConfig {
    pub fn new() -> Self {
        let local = LocalConfig::new().unwrap_or_else(|e| {
            error!("Invalid config found!");
            LocalConfig::default()
        });
        
        Self {
            local,
            live: Arc::new(Mutex::new(LiveConfig::new())),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct LocalConfig {
    pub leagues: std::collections::BTreeMap<String, String>,
    #[serde(default = "base_obs_port")]
    pub obs_port: u16,
    pub obs_password: Option<String>,
    pub broadcast_bar_file: String,
}
impl LocalConfig {
    fn new() -> Result<Self> {
        let config = std::fs::read_to_string("./config.toml")
            .context("Could not read local context")?;
        let local = toml::from_str(&config).context("Invalid config!")?;
        info!("Loaded local config: {local:?}");
        Ok(local)
    }
}

fn base_logo_dir() -> String { "./logos".to_string() }
fn base_obs_port() -> u16 { 4455 }

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct LiveConfig {
    pub team_one: TeamConfig,
    pub team_two: TeamConfig,
    #[serde(default)]
    pub caster_left: CasterConfig,
    #[serde(default)]
    pub caster_right: CasterConfig,
    pub scroll_items: Vec<String>,
}
impl LiveConfig {
    pub fn new() -> Self {
        let config = std::fs::read_to_string("./live_config.toml").unwrap_or_default();
        let live = toml::from_str(&config).unwrap_or_default();
        info!("Loaded live config: {live:?}");
        live
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct TeamConfig {
    pub logo: Option<String>,
    pub name: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct CasterConfig {
    pub name: Option<String>,
    pub vdo_link: Option<String>,
    pub show_camera: bool,
}