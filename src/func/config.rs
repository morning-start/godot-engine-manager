use crate::core::config::ConfigTrait;
use crate::core::source::Source;
use crate::core::utils::{load_json, save_json, symlink};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io;
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
    fn init() -> Self {
        let root = Self::get_root();
        let config_file = root.join("config.json");
        // 如果已存在则load，否则创建
        let cfg = if config_file.exists() {
            Self::load(root)
        } else {
            Self::new(root)
        };
        cfg.init_path();
        cfg
    }
    fn new(root: PathBuf) -> Self {
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

    fn load(root: PathBuf) -> Self {
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

/// 递归复制目录
///
/// # Arguments
/// * `src` - 源目录路径
/// * `dst` - 目标目录路径
///
/// # Returns
/// * `Result<(), io::Error>` - 复制结果
fn copy_dir_recursively(src: &PathBuf, dst: &PathBuf) -> Result<(), io::Error> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursively(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

pub fn link_appdata(data: &PathBuf) {
    let appdata = env::var("APPDATA").unwrap();
    let appdata = PathBuf::from(appdata);
    let appdata = appdata.join("Godot");
    let data_path = data.join("Godot");

    if appdata.exists() {
        // 使用复制和删除替代重命名，以支持跨磁盘移动
        if !appdata.is_symlink() {
            copy_dir_recursively(&appdata, &data_path).unwrap();
            std::fs::remove_dir_all(&appdata).unwrap();
        } else {
            return;
        }
    }
    if !data_path.exists() {
        std::fs::create_dir_all(&data_path).unwrap();
    }
    symlink(&data_path, &appdata).unwrap();
}
