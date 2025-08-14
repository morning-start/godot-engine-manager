use std::path::PathBuf;

use gdem::core::tags;
use gdem::func::config;
use gdem::core::config::ConfigTrait;
use gdem::func::sync;
use gdem::func::list;

#[tokio::main]
async fn main() {
    let root = PathBuf::from("./debug/gdem");
    let cfg = config::Config::new(Some(root));
    // cfg.init_path();
    // cfg.save();
    // sync::sync_data(&cfg).await;
    let remote_engine_map = list::list_remote_engines(&cfg.data).unwrap();
    // println!("{:?}", remote_engine_map);
}
