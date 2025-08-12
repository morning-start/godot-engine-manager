use flate2::read::GzDecoder;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use reqwest::ClientBuilder;
use ring::digest::{Context, SHA256};
use serde_json::Value;
use std::fs::{File, read_dir, read_to_string, remove_dir, remove_dir_all, remove_file, rename};
use std::io::{self, Read, Write};
use std::path::Path;
use tar::Archive;
use tokio::fs::File as TokioFile;
use tokio::io::AsyncWriteExt;
use tokio_stream::StreamExt;
use zip::ZipArchive;

/// 解压缩zip文件到指定文件夹
///
/// # Arguments
/// * `zip_file` - zip文件的路径
/// * `target_folder` - 目标文件夹的路径
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - 解压缩结果
pub fn extract_zip(
    zip_file: &Path,
    target_folder: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(zip_file)?;
    let mut archive = ZipArchive::new(file)?;
    archive.extract(target_folder)?;
    Ok(())
}

/// 解压缩tar.gz文件到指定文件夹
///
/// # Arguments
/// * `tar_gz_file` - tar.gz文件的路径
/// * `target_folder` - 目标文件夹的路径
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - 解压缩结果
pub fn extract_tar_gz(
    tar_gz_file: &Path,
    target_folder: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(tar_gz_file)?;
    let gz_decoder = GzDecoder::new(file);
    let mut archive = Archive::new(gz_decoder);
    archive.unpack(target_folder)?;
    Ok(())
}

// 构建带有可选代理的客户端
pub fn build_client(proxy_url: Option<&str>) -> Result<reqwest::Client, reqwest::Error> {
    let mut builder = ClientBuilder::new();

    if let Some(proxy_str) = proxy_url {
        if let Ok(proxy) = reqwest::Proxy::all(proxy_str) {
            builder = builder.proxy(proxy);
        }
    }

    builder.build()
}
pub async fn download_file(
    uri: &str,
    file_path: &Path,
    proxy_url: Option<&str>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = build_client(proxy_url)?;

    let response = client.get(uri).send().await?;
    // let response= response.error_for_status()?;

    let total_size = response
        .headers()
        .get("content-length")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.parse::<u64>().ok())
        .unwrap_or(0);

    let m = MultiProgress::new();
    let pb = m.add(ProgressBar::new(total_size));
    pb.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {bytes:>7}/{total_bytes:7} ({percent:>3}%) {bytes_per_sec:9} {msg}",
        )?
        .progress_chars("##-"),
    );
    let msg = file_path
        .file_name()
        .and_then(|os_str| os_str.to_str()) // &OsStr -> Option<&str>
        .unwrap_or("unknown"); // 失败时提供默认值

    pb.set_message(msg.to_string());

    let mut file = TokioFile::create(file_path).await?;
    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
        let chunk_len = chunk.len() as u64;
        downloaded += chunk_len;
        pb.set_position(downloaded);
    }

    file.flush().await?;
    pb.finish_with_message("✓");

    Ok(())
}

/// 计算文件的 SHA-256 哈希，返回十六进制字符串
pub fn sha256sum<P: AsRef<Path>>(file_path: P) -> io::Result<String> {
    let path = file_path.as_ref();
    let mut file = File::open(path)?;
    let mut context = Context::new(&SHA256);
    let mut buffer = [0u8; 4096];

    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        context.update(&buffer[..n]);
    }

    let digest = context.finish();
    Ok(digest
        .as_ref()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect())
}

pub fn save_json(json: &Value, file_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(file_path)?;
    file.write_all(serde_json::to_string_pretty(json)?.as_bytes())?;
    Ok(())
}

pub fn load_json(file_path: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let json = serde_json::from_str(&read_to_string(file_path)?)?;
    Ok(json)
}

// 创建系统链接，适配多个系统
pub fn link(original: &Path, link: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // println!("Create link: {} -> {}", original.display(), link.display());
    if link.exists() {
        remove_file(link)?;
    }

    #[cfg(unix)]
    std::os::unix::fs::symlink(original, link)?;
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(original, link)?;
    Ok(())
}

#[derive(Debug, Default, Clone)]
pub struct UrlParams {
    pub params: Vec<(String, String)>,
}

impl UrlParams {
    /// 创建一个新的空的 UrlParams 实例
    pub fn new() -> Self {
        Self { params: Vec::new() }
    }

    /// 添加一个键值对到查询参数中
    /// 允许同一个键出现多次
    pub fn add(&mut self, key: &str, value: &dyn ToString) {
        let value_str = value.to_string();
        self.params.push((key.to_string(), value_str));
    }

    /// 添加一个可迭代集合的所有元素作为值，使用指定的键
    /// 集合中的每个元素都会被转换为字符串并作为独立的键值对添加
    pub fn add_iterable<I, V: ToString>(&mut self, iterable: I, key: &str)
    where
        I: IntoIterator<Item = V>,
    {
        for item in iterable {
            self.add(key, &item);
        }
    }

    /// 有条件地添加一个值到查询参数中
    /// 只有当提供的 `Option<T>` 是 `Some(value)` 时才会添加
    pub fn add_optional<T: ToString>(&mut self, key: &str, value: Option<T>) {
        if let Some(val) = value {
            self.add(key, &val);
        }
    }
    /// 返回参数的数量
    pub fn len(&self) -> usize {
        self.params.len()
    }

    /// 检查是否没有任何参数
    pub fn is_empty(&self) -> bool {
        self.params.is_empty()
    }
}

/// 检测指定文件夹下是否只有一个文件夹，如果是，则将子文件夹的所有内容移动到指定文件夹，然后删除子文件夹。
///
/// # Arguments
/// * `target_folder` - 目标文件夹的路径
///
/// # Returns
/// * `Result<bool, Box<dyn std::error::Error>>` - 如果只有一个子文件夹并成功移动和删除则返回true，否则返回false
pub fn move_and_clean_subfolder(target_folder: &Path) -> Result<bool, Box<dyn std::error::Error>> {
    let mut folders = Vec::new();

    // 遍历目标文件夹中的所有项目
    for entry in read_dir(target_folder)? {
        let entry = entry?;
        let path = entry.path();

        // 检查是否是文件夹
        if path.is_dir() {
            folders.push(path);
        }
    }

    // 如果只有一个子文件夹
    if folders.len() == 1 {
        let subfolder_path = &folders[0];

        // 遍历子文件夹中的所有项目
        for entry in read_dir(subfolder_path)? {
            let entry = entry?;
            let item = entry.path();
            let dst = target_folder.join(entry.file_name());

            // 如果目标位置已存在文件或文件夹，先删除
            if dst.exists() {
                if dst.is_dir() {
                    remove_dir_all(&dst)?;
                } else {
                    remove_file(&dst)?;
                }
            }

            // 移动文件或文件夹
            rename(&item, &dst)?;
        }

        // 删除空的子文件夹
        remove_dir(subfolder_path)?;

        Ok(true)
    } else {
        Ok(false)
    }
}
