use crate::core::source::format_url;
use crate::core::style::new_spinner;
use crate::core::utils::{download_file, extract_zip, sha512sum};
use crate::func::config::Config;
use crate::func::tool::{
    format_engine_name, get_asset_name, get_levels_dir, load_remote_engine_assets,
};
use std::error::Error;
use std::fs;
use std::path::Path;
use tokio::fs::remove_file;

fn query_url(file_name: &str, data: &Path) -> Result<String, Box<dyn Error>> {
    let assets = load_remote_engine_assets(file_name, data)?;

    // 查找文件名
    let asset = assets.flitter(|v| {
        let name = v["name"].as_str().unwrap();
        if name.contains(file_name) {
            return true;
        }
        false
    })?;
    let asset = asset.document.first().unwrap();

    let url = asset["browser_download_url"].as_str().unwrap().to_string();

    Ok(url)
}
fn query_sum_file_url(file_name: &str, data: &Path) -> Result<String, Box<dyn Error>> {
    let assets = load_remote_engine_assets(file_name, data)?;
    let asset = assets.flitter(|v| {
        let name = v["name"].as_str().unwrap();
        if name.contains("SHA512-SUMS") {
            return true;
        }
        false
    })?;
    let asset = asset.document.first().unwrap();
    let url = asset["browser_download_url"].as_str().unwrap().to_string();

    Ok(url)
}

async fn get_remote_sha512(
    file_name: &str,
    cfg: &Config,
    proxy_url: Option<&str>,
) -> Result<String, Box<dyn Error>> {
    let cache_dir = get_levels_dir(&cfg.cache, file_name);
    let sum_url = query_sum_file_url(file_name, &cfg.data).unwrap();
    let source = cfg.source.clone();
    let sum_url = format_url(sum_url.as_str(), Some(source));
    // sum_file_path
    let sum_file_path = cache_dir.join("SHA512-SUMS.txt");
    // 如果sum_file_path 不存在
    if !sum_file_path.exists() {
        match download_file(sum_url.as_str(), &sum_file_path.as_path(), proxy_url).await {
            Ok(_) => {}
            Err(e) => {
                return Err(e);
            }
        }
    }
    // 读取sum_file_path
    let sum_text = fs::read_to_string(&sum_file_path)?;

    // 从sum_text中查找对应文件名的sha512值
    for line in sum_text.lines() {
        // 每行格式为: <sha512_hash>  <file_name>
        if let Some(pos) = line.find(file_name) {
            // 提取sha512值
            let sha512 = line[..pos].trim();
            return Ok(sha512.to_string());
        }
    }

    Err(format!("SHA512 for {} not found", file_name).into())
}

async fn check_sha512(
    file_name: &str,
    cfg: &Config,
    proxy_url: Option<&str>,
) -> Result<bool, Box<dyn Error>> {
    let cache_dir = get_levels_dir(&cfg.cache, file_name);
    let remote_sha512 = get_remote_sha512(file_name, cfg, proxy_url).await?;
    let local_sha512 = sha512sum(cache_dir.join(file_name))?;
    Ok(remote_sha512 == local_sha512)
}

async fn install_engine(file_name: &str, cfg: &Config) -> Result<String, Box<dyn Error>> {
    let cache_dir = get_levels_dir(&cfg.cache, file_name);
    let file_path = cache_dir.join(file_name);

    // 获取下载链接
    let url = query_url(file_name, &cfg.data).unwrap();
    let source = cfg.source.clone();
    let url = format_url(url.as_str(), Some(source));
    // 下载路径
    let proxy_url = if cfg.proxy.is_empty() {
        None
    } else {
        Some(cfg.proxy.as_str())
    };
    match download_file(url.as_str(), &file_path.as_path(), proxy_url).await {
        Ok(msg) => Ok(format!("{} {}", file_name, msg)),
        Err(e) => Err(e),
    }
}

/// 完整的引擎安装流程，包括下载和校验
///
/// 该函数执行完整的引擎安装流程，包括下载指定的引擎文件并校验其完整性。
/// 如果校验失败，会自动删除已下载的文件。
///
/// # Arguments
///
/// * `file_name` - 要安装的引擎文件名
/// * `cfg` - 配置对象，包含安装所需的配置信息
///
/// # Returns
///
/// * `Result<(), Box<dyn Error>>` - 成功时返回Ok(())，失败时返回错误信息
///
/// # Examples
///
/// ```
/// use gdem::func::install::full_install_process;
/// use gdem::func::config::Config;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let engine = "Godot_v4.4.1-stable_win64.exe.zip";
///     let cfg = Config::load()?;
///     full_install_process(engine, &cfg).await?;
///     Ok(())
/// }
/// ```
pub async fn full_install_process(
    engine: &str,
    cfg: &Config,
    force: bool,
    skip_check: bool,
) -> Result<String, Box<dyn Error>> {
    let proxy_url = if cfg.proxy.is_empty() {
        None
    } else {
        Some(cfg.proxy.as_str())
    };
    let engine = &get_asset_name(&engine, &cfg.data);

    let cache_dir = get_levels_dir(&cfg.cache, engine);
    let file_path = cache_dir.join(engine);
    let file_name = format_engine_name(engine);
    if file_path.exists() {
        if force {
            remove_file(&file_path).await?;
        }
    }

    // 获取下载链接
    let pb = new_spinner();
    // 下载引擎
    pb.set_message("Downloading");
    let msg = install_engine(engine, cfg).await.unwrap();
    pb.finish_with_message(msg);

    // 检查sum
    if !skip_check {
        let pb = new_spinner();
        pb.set_message("Checking sum");
        let check = check_sha512(engine, cfg, proxy_url).await?;
        if !check {
            let cache_dir = get_levels_dir(&cfg.cache, engine);
            let file_path = cache_dir.join(engine);
            remove_file(file_path).await?;
            pb.finish_with_message("Checksum failed, file removed");
        }
        pb.finish_with_message("Checksum passed");
    }

    // zip 或 tpz 解压，不同的处理方式。如果要解压，要提供 spinner
    if file_path.to_string_lossy().ends_with(".zip") {
        let pd = new_spinner();
        pd.set_message("Extracting");
        extract_engine(&file_path, engine, &cfg.home)?;
        pd.finish_with_message("Extracting done");
    } else if file_path.to_string_lossy().ends_with(".tpz") {
        let pd = new_spinner();
        pd.set_message("Extracting");
        extract_template(&file_path, &cfg.data, engine)?;
        pd.finish_with_message("Extracting done");
    }
    Ok(file_name)
}

fn extract_engine(file_path: &Path, engine: &str, home: &Path) -> Result<(), Box<dyn Error>> {
    let file_name = format_engine_name(engine);
    let home_dir = get_levels_dir(home, engine);
    let target_folder = home_dir.join(&file_name);

    // 确保目标文件夹存在
    extract_zip(file_path, &target_folder)?;

    // if self_contained {
    //     // 在data_path中创建一个 _sc_ 空文件
    //     let sc_file_path = data_path.join("_sc_");
    //     fs::File::create(sc_file_path)?;
    // } else {
    //     // 如果存在_sc_文件，删除
    //     let sc_file_path = data_path.join("_sc_");
    //     if sc_file_path.exists() {
    //         fs::remove_file(sc_file_path)?;
    //     }
    // }
    Ok(())
}

// TODO 解压导出模板 .tpz  使用 zip 解压。
// FIXME 问题难点，需要通过 导出模板名 获取对应的 引擎名 来确定 self contained 模式 的路径

fn extract_template(file_path: &Path, data: &Path, file: &str) -> Result<(), Box<dyn Error>> {
    // file_path  xxx.tpz
    let target_path = data.join("Godot").join("export_templates");

    // Godot_v3.6.1-stable_mono_export_templates.tpz -> 3.6.1.stable.mono
    // Godot_v3.6.1-stable_export_templates.tpz -> 3.6.1.stable
    let template_dir = file
        .replace("_export_templates.tpz", "")
        .replace("Godot_v", "")
        .replace("_", ".")
        .replace("-", ".");
    let target_folder = target_path.join(&template_dir);

    extract_zip(file_path, &target_folder)?;
    Ok(())
}
