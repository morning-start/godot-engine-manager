use crate::core::handler::DocumentHandler;
use crate::core::tags::is_support_file;
use crate::core::tags::{Architecture, OS, Tag};
use crate::core::utils::format_size;
use regex::Regex;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

pub fn format_engine_name(engine: &str) -> String {
    let file_name: String = engine.replace(".zip", "").replace(".exe", "");
    file_name
}

pub fn get_levels_dir(root: &Path, engine: &str) -> PathBuf {
    let version = extract_version(engine).unwrap();
    let major = get_major_from_tag(version.as_str());
    let l_dir = root.join(major).join(version);
    if !l_dir.exists() {
        fs::create_dir_all(&l_dir).unwrap();
    }
    l_dir
}

/// 从文件名中提取版本号
///
/// # Arguments
///
/// * `name` - 文件名字符串
///
/// # Returns
///
/// * `Option<String>` - 提取到的版本号，如果未找到则返回 None
pub fn extract_version(engine: &str) -> Option<String> {
    // 匹配两种版本格式：4.4.1 和 4.4
    let re = Regex::new(r#"(\d+\.\d+(?:\.\d+)?)"#).unwrap();
    re.captures(engine)
        .map(|captures| captures.get(1).unwrap().as_str().to_string())
}

pub fn load_remote_engines_handler(
    data: &Path,
    fields: &[&str],
) -> Result<DocumentHandler, Box<dyn Error>> {
    let file_path = data.join("releases.json");
    let mut handler = DocumentHandler::load_data(&file_path)?;
    handler.apply("assets", |v| {
        let assets = v.as_array().unwrap();
        let assets = DocumentHandler::new(assets.clone());
        // 对assets 字段进行删除，只保留 name size updated_at browser_download_url
        let fields = ["name", "size", "updated_at", "browser_download_url"];
        let assets = assets.get_specific_fields(&fields).unwrap();
        // is_support_extension 过滤assets 中不支持的文件
        // 只保留本机系统和架构
        let local_os = OS::get_local_os();
        let local_arch = Architecture::get_local_arch();
        let assets = assets
            .flitter(|v| {
                let name = v["name"].as_str().unwrap();
                let is_supported = is_support_file(name);
                // 如果是zip，判断系统和架构
                let mut flag = true;
                if name.ends_with(".zip") {
                    let is_os = local_os.tag_in(name);
                    let is_arch = local_arch.tag_in(name);
                    flag = is_os && is_arch;
                }
                flag && is_supported
            })
            .unwrap();
        assets.document.into()
    })?;
    let handler = handler.get_specific_fields(fields)?;
    Ok(handler)
}

/// 从tag_name 中提取major 版本号
///
/// # Arguments
///
/// * `tag_name` - tag_name 字符串
///
/// # Returns
///
/// * `String` - 提取到的major 版本号
///
/// # Examples
///
/// ```
/// use gdem::func::tool::get_major_from_tag;
/// let tag_name = "3.5.1-stable";
/// let major = get_major_from_tag(tag_name);
/// assert_eq!(major, "3.x");
/// ```
pub fn get_major_from_tag(tag_name: &str) -> String {
    let major = tag_name.split(".");
    let major = major.collect::<Vec<&str>>()[0];
    format!("{}.x", major)
}

/// 加载远程引擎资源并处理数据格式
///
/// 该函数从指定路径加载远程引擎的发布数据，提取与指定版本匹配的资源，
/// 并对资源数据进行处理，包括文件大小格式化和更新时间格式化。
///
/// # Arguments
///
/// * `file_name` - 要匹配的文件名字符串，用于提取版本号
/// * `data` - 包含 releases.json 文件的目录路径
///
/// # Returns
///
/// * `Result<DocumentHandler, Box<dyn Error>>` - 成功时返回处理后的资源数据，失败时返回错误信息
///
/// # Process
///
/// 1. 从文件名中提取版本号
/// 2. 加载远程引擎处理句柄
/// 3. 过滤出与版本号匹配的资源
/// 4. 提取匹配资源的资产信息
/// 5. 格式化资产中的文件大小（转换为 KB、MB 或 GB）
/// 6. 格式化更新时间（转换为 YYYY-MM-DD 格式）
pub fn load_remote_engine_assets(
    file_name: &str,
    data: &Path,
) -> Result<DocumentHandler, Box<dyn Error>> {
    let version = extract_version(file_name).unwrap();

    let handler = load_remote_engines_handler(data, &["tag_name", "assets"])?;
    let handler = handler.flitter(|v| {
        let name = v["tag_name"].as_str().unwrap();
        // 检查major
        if name.starts_with(version.as_str()) {
            return true;
        }
        false
    })?;
    // 从handler 中提取assets
    let latest_assets = handler.document.first().unwrap();
    let latest_assets = latest_assets["assets"].as_array().unwrap().clone();

    let mut assets = DocumentHandler::new(latest_assets);
    // 对size进行处理，计算为字符串 KB、MB 或 GB
    for item in assets.document.iter_mut() {
        let size = item["size"].as_u64().unwrap() as f64;
        let size = format_size(size);
        item["size"] = size.into();
    }
    // updated_at 格式化为 2023-01-01 的格式
    for item in assets.document.iter_mut() {
        let updated_at = item["updated_at"].as_str().unwrap();
        let updated_at = updated_at.split("T").collect::<Vec<&str>>()[0];
        item["updated_at"] = updated_at.into();
    }
    Ok(assets)
}
