use serde_derive::Deserialize;
use std::{
    fs::{create_dir, File},
    io::{BufReader, Error, ErrorKind, Read, Write},
    path::PathBuf,
};
use toml::Value;

const APP_NAME: &'static str = "workstyle";
const DEFAULT_FALLBACK_ICON: &'static str = " ";

fn config_file() -> Result<PathBuf, Error> {
    let mut path_to_config =
        dirs::config_dir().ok_or(Error::new(ErrorKind::Other, "Missing default config dir"))?;
    path_to_config.push(APP_NAME);
    if !path_to_config.exists() {
        create_dir(path_to_config.clone())?;
    }
    path_to_config.push("config.toml");
    Ok(path_to_config)
}

pub(super) fn generate_config_file_if_absent() -> Result<PathBuf, Error> {
    let config_file = config_file()?;
    if !config_file.exists() {
        let mut config_file = File::create(&config_file)?;
        let content = include_bytes!("default_config.toml");
        config_file.write_all(content)?;
    }
    Ok(config_file)
}

// I had rather simply deserialize the map with serde, but I don't know what to deserialize into to
// preserve ordering.
fn try_from_toml_value(value: &Value) -> Result<Vec<(String, String)>, String> {
    match value {
        Value::Table(map) => map
            .into_iter()
            .map(|(k, v)| {
                Ok((
                    k.clone(),
                    v.as_str()
                        .ok_or("Expected for the map value to be a string")?
                        .into(),
                ))
            })
            .collect(),
        _ => Err("Expected a map".to_string()),
    }
}

fn get_icon_mappings_from_config(config: &PathBuf) -> Result<Vec<(String, String)>, Error> {
    let mut config_file = File::open(config)?;
    let mut content = String::new();
    config_file.read_to_string(&mut content)?;
    try_from_toml_value(&content.parse::<toml::Value>().map_err(|e| {
        log::error!(
            "Error parsing configuration file.\nInvalid syntax in {:#?}.\n{}",
            config,
            e
        );
        Error::new(ErrorKind::Other, "Invalid configuration file")
    })?)
    .map_err(|e| {
        log::error!("{}", e);
        Error::new(ErrorKind::Other, "Invalid configuration file")
    })
}

fn get_icon_mappings_from_default_config() -> Vec<(String, String)> {
    try_from_toml_value(&String::from_utf8(include_bytes!("default_config.toml").iter().cloned().collect()).expect("Expected utf-8 encoded string in default_config.toml").parse::<Value>()
        .expect("The default config isn't user generated, so we assumed it was correct. This will teach us not to trust programmers.")).expect("Bang!")
}

pub(super) fn get_icon_mappings(config: &Result<PathBuf, Error>) -> Vec<(String, String)> {
    if let Ok(config) = config {
        if let Ok(content) = get_icon_mappings_from_config(&config) {
            return content;
        }
    }
    get_icon_mappings_from_default_config()
}

#[derive(Debug, Deserialize)]
struct Config {
    other: Option<ExtraConfig>,
}

#[derive(Debug, Deserialize)]
struct ExtraConfig {
    #[serde(default = "ExtraConfig::default_fallback_icon")]
    fallback_icon: String,
}

impl ExtraConfig {
  fn default_fallback_icon() -> String {
      DEFAULT_FALLBACK_ICON.to_string()
  }
}

pub(super) fn get_fallback_icon(config: &Result<PathBuf, Error>) -> String {
    let config_path = config.as_ref().unwrap().to_str().unwrap();
    let mut config_file = BufReader::new(
        File::open(config_path).expect(&format!("Failed to open file: {}", config_path)),
    );
    let mut content = String::new();
    config_file
        .read_to_string(&mut content)
        .expect("Failed to read file");
    let config: Config = toml::from_str(&content).unwrap();
    if let Some(other) = config.other {
        other.fallback_icon
    } else {
        DEFAULT_FALLBACK_ICON.to_string()
    }
}
