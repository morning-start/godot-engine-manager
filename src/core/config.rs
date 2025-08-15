use serde_json::Value;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};

pub trait ConfigTrait {
    // abstract
    fn get_root() -> PathBuf;
    fn new(root: Option<PathBuf>) -> Self;
    fn load() -> Self;
    fn save(&self);
    fn init_path(&self);
    // mut
    fn switch_version(&mut self, version: &str);

    // impl
    fn get_config_path() -> PathBuf {
        Self::get_root().join("config.json")
    }
    fn init_dir(dirs: &[&Path]) {
        for dir in dirs {
            if !dir.exists() {
                create_dir_all(dir).unwrap();
            }
        }
    }
    fn val2path(val: Option<&Value>) -> PathBuf {
        val.unwrap().as_str().unwrap_or_default().into()
    }
    fn val2str(val: Option<&Value>) -> String {
        val.unwrap().as_str().unwrap_or_default().to_string()
    }
    fn val2bool(val: Option<&Value>) -> bool {
        val.unwrap().as_bool().unwrap_or_default()
    }
    fn val2num(val: Option<&Value>) -> f64 {
        val.unwrap()
            .as_number()
            .unwrap()
            .as_f64()
            .unwrap_or_default()
    }
}
