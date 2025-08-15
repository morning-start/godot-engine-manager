use std::error::Error;

use crate::func::{
    config::Config,
    tool::{format_engine_name, get_levels_dir},
};
use std::fs::remove_dir_all;

pub fn uninstall_engine(engine: &str, cfg: &mut Config) -> Result<bool, Box<dyn Error>> {
    let engine_name = format_engine_name(engine);
    let home_dir = get_levels_dir(&cfg.home, engine);
    let engine_path = home_dir.join(engine_name);
    if engine_path.exists() {
        remove_dir_all(engine_path)?;
        return Ok(true);
    }
    Ok(false)
}
