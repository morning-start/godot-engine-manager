use super::config::Config;
use crate::core::config::ConfigTrait;
use crate::core::utils::symlink;
use crate::func::tool::{format_engine_name, get_levels_dir};
use std::error::Error;

pub fn switch_engine(engine: &str, cfg: &mut Config) -> Result<bool, Box<dyn Error>> {
    let link_path = cfg.root.join("default");
    let home_dir = get_levels_dir(&cfg.home, engine);
    // filename 去除zip和exe
    let engine = format_engine_name(engine);
    let engine_path = home_dir.join(&engine);

    if let Err(e) = symlink(&engine_path, &link_path) {
        eprintln!("Create link failed: {}", e);
        return Err(e);
    }
    cfg.switch_version(&engine);
    cfg.save();
    Ok(true)
}
