use tokio::fs::remove_file;

use crate::core::source::format_url;
use crate::core::style::new_spinner;
use crate::core::utils::{download_file, extract_zip, promote_if_single_subdir, sha512sum};
use crate::func::tool::{get_major_from_tag, load_remote_engine_assets};
use crate::func::{config::Config, tool::extract_version};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

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
fn get_cache_dir(root: &Path, file_name: &str) -> PathBuf {
    let version = extract_version(file_name).unwrap();
    let major = get_major_from_tag(version.as_str());
    let cache_dir = root.join(major).join(version);
    if !cache_dir.exists() {
        fs::create_dir_all(&cache_dir).unwrap();
    }
    cache_dir
}

async fn get_remote_sha512(
    file_name: &str,
    cfg: &Config,
    proxy_url: Option<&str>,
) -> Result<String, Box<dyn Error>> {
    let cache_dir = get_cache_dir(&cfg.cache, file_name);
    let sum_url = query_sum_file_url(file_name, &cfg.data).unwrap();
    let source = cfg.source.clone();
    let sum_url = format_url(sum_url.as_str(), Some(source));
    // sum_file_path
    let sum_file_path = cache_dir.join("SHA512-SUMS.txt");
    // 如果sum_file_path 不存在
    if !sum_file_path.exists() {
        if download_file(sum_url.as_str(), &sum_file_path.as_path(), proxy_url)
            .await
            .is_err()
        {
            return Err("Download failed".into());
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
    let cache_dir = get_cache_dir(&cfg.cache, file_name);
    let remote_sha512 = get_remote_sha512(file_name, cfg, proxy_url).await?;
    let local_sha512 = sha512sum(cache_dir.join(file_name))?;
    Ok(remote_sha512 == local_sha512)
}

async fn install_engine(file_name: &str, cfg: &Config) -> Result<String, Box<dyn Error>> {
    let cache_dir = get_cache_dir(&cfg.cache, file_name);
    let file_path = cache_dir.join(file_name);
    if file_path.exists() {
        return Ok(format!("{} exists", file_name));
    }

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
    if download_file(url.as_str(), &file_path.as_path(), proxy_url)
        .await
        .is_err()
    {
        return Err("Download failed".into());
    }

    Ok(format!("{} download success", file_name))
}

pub fn extract_engine(
    package_path: &Path,
    target_folder: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    // 确保目标文件夹存在
    std::fs::create_dir_all(target_folder)?;
    extract_zip(package_path, target_folder)?;
    // 智能解压，如果target_folder中只有一个文件夹，则优化文件结构
    //  target_folder/a/  =>  target_folder/
    promote_if_single_subdir(target_folder)?;
    Ok(())
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
///     let file_name = "Godot_v4.4.1-stable_win64.exe.zip";
///     let cfg = Config::load()?;
///     full_install_process(file_name, &cfg).await?;
///     Ok(())
/// }
/// ```
pub async fn full_install_process(file_name: &str, cfg: &Config) -> Result<(), Box<dyn Error>> {
    let proxy_url = if cfg.proxy.is_empty() {
        None
    } else {
        Some(cfg.proxy.as_str())
    };
    // 获取下载链接
    let pb = new_spinner();

    // 下载引擎
    pb.set_message("Downloading");
    let msg = install_engine(file_name, cfg).await?;
    pb.set_message(msg);

    // 检查sum
    pb.set_message("Checking sum");

    let check = check_sha512(file_name, cfg, proxy_url).await?;
    if !check {
        let cache_dir = get_cache_dir(&cfg.cache, file_name);
        let file_path = cache_dir.join(file_name);
        remove_file(file_path).await?;
        pb.finish_with_message("Checksum failed, file removed");
    }
    pb.finish_with_message("Checksum passed");

    let pd = new_spinner();
    pd.set_message("Extracting");
    let cache_dir = get_cache_dir(&cfg.cache, file_name);
    let file_path = cache_dir.join(file_name);
    let data_dir = get_cache_dir(&cfg.data, file_name);
    // filename 去除zip和exe
    let file_name: String = file_name.replace(".zip", "").replace(".exe", "");
    let data_path = data_dir.join(file_name);
    extract_engine(&file_path, &data_path)?;
    pd.finish_with_message("Extracting done");

    Ok(())
}
