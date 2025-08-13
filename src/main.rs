use std::path::PathBuf;

use gdem::func::config;
use gdem::core::config::ConfigTrait;
use gdem::func::sync;

#[tokio::main]
async fn main() {
    let root = PathBuf::from("./debug/gdem");
    let cfg = config::Config::new(Some(root));
    cfg.init_path();
    cfg.save();
    sync::sync_data(&cfg).await;
}
