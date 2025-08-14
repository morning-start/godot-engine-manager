use std::path::PathBuf;

use gdem::core::config::ConfigTrait;
use gdem::core::tags;
use gdem::func::config;
use gdem::func::list;
use gdem::func::sync;

#[tokio::main]
async fn main() {
    let root = PathBuf::from("./debug/gdem");
    let cfg = config::Config::new(Some(root));
    // cfg.init_path();
    // cfg.save();
    // sync::sync_data(&cfg).await;
    let remote_engine_map = list::list_remote_engine_tags(&cfg.data, "4").unwrap();
    let names = &remote_engine_map
        .iter()
        .map(|v| v.as_object().unwrap()["name"].as_str().unwrap().to_string())
        .collect::<Vec<String>>();
    println!("{:?}", remote_engine_map);
}
