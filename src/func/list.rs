use crate::core::handler::DocumentHandler;
use crate::core::utils::format_size;
use crate::func::tool::{get_major_from_tag, load_remote_engines_handler};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct EngineInfo {
    pub major: String,
    pub versions: Vec<String>,
}

pub fn list_local_engines(home: &Path) -> Result<Vec<Value>, Box<dyn Error>> {
    // 如果 home 目录不存在或无法读取，返回空的Vec
    let mut engine_list: Vec<Value> = Vec::new();

    // 尝试读取主目录，如果失败则返回空列表
    let major_entries = match home.read_dir() {
        Ok(entries) => entries,
        Err(_) => return Ok(Vec::new()),
    };

    // 遍历主版本目录 (如 4.x, 3.x)
    for major_entry in major_entries {
        let major_path = major_entry?.path();

        // 确保是目录
        if !major_path.is_dir() {
            continue;
        }

        // 读取版本目录
        let version_entries = match major_path.read_dir() {
            Ok(entries) => entries,
            Err(_) => continue, // 如果无法读取版本目录，跳过这个主版本
        };

        // 遍历版本目录 (如 4.0, 4.1)
        for version_entry in version_entries {
            let version_path = version_entry?.path();

            // 确保是目录
            if !version_path.is_dir() {
                continue;
            }

            // 获取引擎目录名称
            let engine_dir_names: Vec<String> = version_path
                .read_dir()?
                .filter_map(|entry| {
                    entry.ok().and_then(|e| {
                        let path = e.path();
                        if path.is_dir() {
                            Some(e.file_name().to_string_lossy().to_string())
                        } else {
                            None
                        }
                    })
                })
                .collect();

            // 将引擎目录名称添加到结果列表
            engine_list.extend_from_slice(json!(engine_dir_names).as_array().unwrap());
        }
    }

    Ok(engine_list)
}

pub fn list_remote_engines(data: &Path) -> Result<Vec<Value>, Box<dyn Error>> {
    let mut handler = load_remote_engines_handler(data, &["tag_name"])?;

    // 在document 中添加major字段
    for item in handler.document.iter_mut() {
        let tag_name = item["tag_name"].as_str().unwrap();
        let major = get_major_from_tag(tag_name);
        item["major"] = major.into();
    }
    // group by major
    let mut major_handler = handler
        .group_by("major", Some(|val_list| val_list))
        .unwrap();
    let name_map = HashMap::from([("tag_name".to_string(), "versions".to_string())]);
    major_handler.rename(&name_map)?;

    // 遍历major_handler 中的每个元素

    // 遍历handler 中的每个元素
    Ok(major_handler.document)
}

/// 列出远程引擎的资产信息
///
/// 该函数从远程数据源加载指定版本的引擎资产信息，并对数据进行处理和格式化。
/// 处理过程包括：筛选指定版本的资产、提取特定字段、格式化文件大小和更新时间、过滤掉文本文件。
///
/// # Arguments
///
/// * `data` - 包含releases.json文件的目录路径
/// * `version` - 要查询的引擎版本号，例如"4.4-stable" 或 "4.4.1"
///
/// # Returns
///
/// * `Result<Vec<Value>, Box<dyn Error>>` - 成功时返回处理后的资产信息列表，失败时返回错误信息
///
/// # Process
///
/// 1. 使用load_remote_engines_handler加载包含tag_name和assets字段的远程数据
/// 2. 筛选出tag_name以指定version开头的记录
/// 3. 提取第一条记录的assets数组
/// 4. 从assets中提取name、size、updated_at三个字段
/// 5. 格式化size字段为人类可读的KB/MB/GB格式
/// 6. 格式化updated_at字段为YYYY-MM-DD格式
/// 7. 过滤掉以.txt结尾的文件项
pub fn list_remote_engine_assets(data: &Path, version: &str) -> Result<Vec<Value>, Box<dyn Error>> {
    let handler = load_remote_engines_handler(data, &["tag_name", "assets"])?;
    //handler flitter tag_name 中 name 包含 version 的
    let handler = handler.flitter(|v| {
        let name = v["tag_name"].as_str().unwrap();
        // 检查major
        if name.starts_with(version) {
            return true;
        }
        false
    })?;
    // 从handler 中提取assets
    let latest_assets = handler.document.first().unwrap();
    let latest_assets = latest_assets["assets"].as_array().unwrap().clone();

    let latest_assets = DocumentHandler::new(latest_assets);
    let mut assets = latest_assets.get_specific_fields(&["name", "size", "updated_at"])?;

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
    // 过滤掉 name 字段 以txt结尾
    let assets = assets.flitter(|v| {
        let name = v["name"].as_str().unwrap();
        if name.ends_with(".txt") {
            return false;
        }
        true
    })?;

    Ok(assets.document)
}
