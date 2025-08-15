use std::path::PathBuf;

use gdem::core::config::ConfigTrait;
use gdem::core::tags;
use gdem::func::config;
use gdem::func::install::full_install_process;
use gdem::func::list;
use gdem::func::sync;
use gdem::func::tool::extract_version;



#[tokio::main]
async fn main() {
    let root = PathBuf::from("./debug/gdem");
    let cfg = config::Config::new(Some(root));
    // cfg.init_path();
    // cfg.save();
    // sync::sync_data(&cfg).await;
    let local_engine_map = list::list_local_engines(&cfg.home).unwrap();
    println!("{:?}", local_engine_map);
    // let remote_engine_map = list::list_remote_engine_assets(&cfg.data,"4.4").unwrap();
    // let names = &remote_engine_map
    //     .iter()
    //     .map(|v| v.as_object().unwrap()["name"].as_str().unwrap().to_string())
    //     .collect::<Vec<String>>();
    // println!("{:?}", names);
    // full_install_process(names[4].as_str(), &cfg).await.unwrap();
    // full_install_process(names[5].as_str(), &cfg).await.unwrap();

}
