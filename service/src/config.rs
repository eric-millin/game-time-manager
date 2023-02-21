use serde::Deserialize;
use std::fs;
use std::str::FromStr;

#[derive(Deserialize)]
pub struct Config {
    overlay: OverlayConfig,
    watcher: WatcherConfig,
}

#[derive(Deserialize)]
struct OverlayConfig {
    width: u32,
    height: u32,
    font: String,
    font_size: u32,
    font_rgb: Vec<u8>,
    background_rgb: Vec<u8>,
}

#[derive(Deserialize)]
struct WatcherConfig {
    poll_frequency: u32,
    show_frequency: u32,
    show_duration: u32,
}

pub fn load() -> Result<Config, toml::de::Error> {
    let cfg = fs::read_to_string("config.toml").unwrap();

    return toml::from_str(cfg.as_str());
}
