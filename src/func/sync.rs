use crate::core::utils::{build_client, save_json};
use crate::func::config::Config;
use serde_json::Value;

pub async fn sync_data(cfg: &Config) {
    const URL: &str = "https://godothub.atomgit.net/web/api/releases.json";
    let file_path = cfg.data.join("releases.json");
    if file_path.exists() {
        return;
    }
    let client = build_client(Some(cfg.proxy.as_str())).unwrap();
    let resp = client.get(URL).send().await.unwrap();
    // json
    let res = resp.json::<Value>().await.unwrap();
    save_json(&res, &file_path).unwrap();
}
