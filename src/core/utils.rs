use flate2::read::GzDecoder;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use reqwest::ClientBuilder;
use ring::digest::{Context, SHA256, SHA512};
use serde_json::Value;
use std::fs::{
    self, File, read_dir, read_to_string, remove_dir, remove_dir_all, remove_file, rename,
};
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
    std::fs::create_dir_all(&target_folder)?;
    let file = File::open(zip_file)?;
    let mut archive = ZipArchive::new(file)?;
    archive.extract(target_folder)?;
    promote_if_single_subdir(&target_folder)?;
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
/// 异步下载文件到指定路径，并显示下载进度
///
/// # Arguments
///
/// * `uri` - 要下载的文件的URL
/// * `file_path` - 保存文件的本地路径
/// * `proxy_url` - 可选的代理URL
///
/// # Returns
///
/// * `Result<(), Box<dyn std::error::Error + Send + Sync>>` - 下载结果，成功返回Ok(())，失败返回错误信息
///
/// # Example
///
/// ```
/// use godot_engine_manager::core::utils::download_file;
/// use std::path::Path;
///
/// let uri = "https://downloads.tuxfamily.org/godotengine/4.0/Godot_v4.0-stable_win64.zip";
/// let file_path = Path::new("Godot_v4.0-stable_win64.zip");
/// let proxy_url = Some("http://127.0.0.1:7890");
///
/// download_file(uri, file_path, proxy_url).await.unwrap();
/// ```

///

/// 获取远程文件的总大小
async fn get_remote_file_size(
    client: &reqwest::Client,
    uri: &str,
    start_pos: u64,
) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    // 构建请求，添加Range头以支持断点续传
    let request = client.get(uri);
    let request = if start_pos > 0 {
        request.header("Range", format!("bytes={}-", start_pos))
    } else {
        request
    };

    let response = request.send().await?;
    let total_size = response
        .headers()
        .get("content-length")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.parse::<u64>().ok())
        .unwrap_or(0);

    // 如果是断点续传，需要获取实际的总文件大小
    let total_size = if start_pos > 0 {
        // 从Content-Range头获取总大小
        if let Some(content_range) = response.headers().get("content-range") {
            if let Ok(content_range_str) = content_range.to_str() {
                // 格式: bytes 100-199/200
                if let Some(total) = content_range_str.split('/').nth(1) {
                    total.parse::<u64>().unwrap_or(total_size + start_pos)
                } else {
                    total_size + start_pos
                }
            } else {
                total_size + start_pos
            }
        } else {
            total_size + start_pos
        }
    } else {
        total_size
    };

    Ok(total_size)
}

pub async fn download_file(
    uri: &str,
    file_path: &Path,
    proxy_url: Option<&str>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let client = build_client(proxy_url)?;

    // 检查本地已存在的文件大小
    let start_pos = if file_path.exists() {
        let metadata = tokio::fs::metadata(file_path).await?;
        metadata.len()
    } else {
        0
    };

    // 获取远程文件总大小
    let total_size = get_remote_file_size(&client, uri, start_pos).await?;

    // 如果本地文件已完全下载，则直接返回
    if start_pos == total_size && total_size > 0 {
        return Ok("File already downloaded".to_string());
    }

    // 重新发送请求以获取响应流
    let request = client.get(uri);
    let request = if start_pos > 0 {
        request.header("Range", format!("bytes={}-", start_pos))
    } else {
        request
    };
    let response = request.send().await?;

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
    pb.set_position(start_pos);

    // 以追加模式打开文件
    let mut file = if start_pos > 0 {
        TokioFile::options().append(true).open(file_path).await?
    } else {
        TokioFile::create(file_path).await?
    };

    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = start_pos;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
        let chunk_len = chunk.len() as u64;
        downloaded += chunk_len;
        pb.set_position(downloaded);
    }

    file.flush().await?;
    pb.finish_with_message("✓");

    Ok("Download completed".to_string())
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

pub fn sha512sum<P: AsRef<Path>>(file_path: P) -> io::Result<String> {
    let path = file_path.as_ref();
    let mut file = File::open(path)?;
    let mut context = Context::new(&SHA512);
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
pub fn symlink(original: &Path, link: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // println!("Create link: {} -> {}", original.display(), link.display());
    // // 清理现有的 链接或目录
    if link.exists() {
        if link.is_dir() {
            fs::remove_dir_all(link)?;
        } else {
            fs::remove_file(link)?;
        }
    }

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

/// 将字节大小格式化为人类可读的字符串表示
///
/// # Arguments
///
/// * `size` - 以字节为单位的大小值
///
/// # Returns
///
/// * `String` - 格式化后的大小字符串，包含适当的单位（B, KB, MB, GB）
pub fn format_size(size: f64) -> String {
    if size > 1024.0 * 1024.0 * 1024.0 {
        format!("{:.2}GB", size / 1024.0 / 1024.0 / 1024.0)
    } else if size > 1024.0 * 1024.0 {
        format!("{:.2}MB", size / 1024.0 / 1024.0)
    } else if size > 1024.0 {
        format!("{:.2}KB", size / 1024.0)
    } else {
        format!("{}B", size)
    }
}

pub fn promote_if_single_subdir(target_folder: &Path) -> io::Result<()> {
    if !target_folder.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotADirectory,
            "目标路径不是目录",
        ));
    }

    let entries: Vec<fs::DirEntry> = fs::read_dir(target_folder)?
        .filter_map(|res| res.ok())
        .filter(|entry| {
            // 可选：忽略隐藏文件（以 . 开头）
            let name = entry.file_name();
            if let Some(name_str) = name.to_str() {
                !name_str.starts_with('.')
            } else {
                true // 无法转成字符串的也保留
            }
        })
        .collect();

    // 检查可见条目数量
    if entries.len() == 1 {
        let entry = &entries[0];
        let inner_path = entry.path();

        if inner_path.is_dir() {
            let parent_dir = target_folder;

            // 将 inner_path 中的所有内容移动到 parent_dir
            for inner_entry in fs::read_dir(&inner_path)? {
                let inner_entry = inner_entry?;
                let src_path = inner_entry.path();
                let dst_path = parent_dir.join(inner_entry.file_name());

                fs::rename(src_path, dst_path)?;
            }

            // 删除空的原目录
            fs::remove_dir(&inner_path)?;
        }
    }

    Ok(())
}
