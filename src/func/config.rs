use crate::core::config::ConfigTrait;
use crate::core::source::Source;
use crate::core::utils::{load_json, save_json};
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;

#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    pub root: PathBuf,
    pub home: PathBuf,
    pub cache: PathBuf,
    pub data: PathBuf,
    pub proxy: String,
    pub version: String,
    pub source: Source,
}
impl ConfigTrait for Config {
    fn new(root: Option<PathBuf>) -> Self {
        let root = root.unwrap_or_else(Config::get_root);
        let home = root.join("home");
        let cache = root.join("cache");
        let data = root.join("data");
        Self {
            root,
            home,
            cache,
            data,
            proxy: "".to_string(),
            version: "".to_string(),
            source: Source::GodotHub,
        }
    }

    fn get_root() -> PathBuf {
        let gdem_root = env::var("GDEM_ROOT");
        match gdem_root {
            Ok(root) => PathBuf::from(root),
            Err(_) => {
                // 默认目录
                let home = env::var("HOME").unwrap();
                PathBuf::from(home).join(".gdem")
            }
        }
    }

    fn load() -> Self {
        let root = Self::get_root();
        let config = load_json(&root.join("config.json")).unwrap();
        let root = Self::val2path(config.get("root"));
        let home = Self::val2path(config.get("home"));
        let cache = Self::val2path(config.get("cache"));
        let data = Self::val2path(config.get("data"));
        let proxy = Self::val2str(config.get("proxy"));
        let version = Self::val2str(config.get("version"));
        let source = Self::val2str(config.get("source"));
        let source = Source::from_str(source.as_str());
        Self {
            root,
            home,
            cache,
            data,
            proxy,
            version,
            source: source,
        }
    }
    fn init_path(&self) {
        Self::init_dir(&[&self.root, &self.home, &self.cache, &self.data]);
    }
    fn save(&self) {
        let config = serde_json::to_value(&self).unwrap();
        save_json(&config, &self.root.join("config.json")).unwrap();
    }
    fn switch_version(&mut self, version: &str) {
        self.version = version.to_string();
    }
}
