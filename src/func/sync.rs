use crate::core::utils::{build_client, save_json};
use crate::func::config::Config;
use indicatif::{ProgressBar, ProgressStyle};
use serde_json::Value;

pub async fn sync_data(cfg: &Config) {
    const URL: &str = "https://godothub.atomgit.net/web/api/releases.json";
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner());
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    pb.set_message("Syncing data...");

    let file_path = cfg.data.join("releases.json");
    if file_path.exists() {
        return;
    }
    let client = build_client(Some(cfg.proxy.as_str())).unwrap();
    let resp = client.get(URL).send().await.unwrap();
    // json
    let res = resp.json::<Value>().await.unwrap();
    save_json(&res, &file_path).unwrap();
    pb.finish_with_message("Sync data done.");
}
