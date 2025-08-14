use crate::core::handler::DocumentHandler;
use crate::core::tags::is_support_file;
use crate::core::tags::{Architecture, OS, Tag};
use crate::core::utils::format_size;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct EngineInfo {
    pub major: String,
    pub versions: Vec<String>,
}

pub fn list_local_engines(home: &Path) -> Result<Vec<EngineInfo>, Box<dyn Error>> {
    // {'4.x':['4.0','4.1']} --> [{ 'major':'4.x','versions':['4.0','4.1']}]
    let mut engine_list: Vec<EngineInfo> = Vec::new();
    // 如果 home 目录是空的，返回空的Vec
    if home.iter().next().is_none() {
        return Ok(engine_list);
    } else {
        // 遍历 home 目录下的所有文件
        // home/major/version
        for major in home.read_dir()? {
            let major = major?.path();
            let mut major_list: Vec<String> = Vec::new();
            if major.is_dir() {
                // 迭代子目录
                for version in major.read_dir()? {
                    let version = version?.path();
                    if version.is_dir() {
                        let dir_name = version.file_name().unwrap().to_string_lossy().to_string();
                        major_list.push(dir_name);
                    }
                }
                let major_name = major.file_name().unwrap().to_string_lossy().to_string();
                engine_list.push(EngineInfo {
                    major: major_name,
                    versions: major_list,
                });
            }
        }
        // 如果version 为空，删除该元素
        engine_list.retain(|v| !v.versions.is_empty());
        // 对每个Vec<String> 进行倒叙排序
        for v in engine_list.iter_mut() {
            v.versions.sort_by(|a, b| b.cmp(a));
        }
        // 对每个major 进行倒叙排序
        engine_list.sort_by(|a, b| b.major.cmp(&a.major));
        return Ok(engine_list);
    }
}

pub fn list_remote_engines(data: &Path) -> Result<Vec<EngineInfo>, Box<dyn Error>> {
    let mut engine_list: Vec<EngineInfo> = Vec::new();
    let mut handler = load_remote_engines_handler(data, &["tag_name"])?;

    // 在document 中添加major字段
    for item in handler.document.iter_mut() {
        let tag_name = item["tag_name"].as_str().unwrap();
        let major = get_major(tag_name);
        item["major"] = major.into();
    }
    // group by major
    let mut major_handler = handler
        .group_by("major", Some(|val_list| val_list))
        .unwrap();
    let name_map = HashMap::from([("tag_name".to_string(), "versions".to_string())]);
    major_handler.rename(&name_map)?;

    // 遍历major_handler 中的每个元素
    for obj in major_handler.document.iter() {
        let major = obj["major"].as_str().unwrap();
        let val_list = obj["versions"].as_array().unwrap();
        let versions = val_list
            .iter()
            .filter_map(|item| item.as_str())
            .map(ToString::to_string)
            .collect();
        engine_list.push(EngineInfo {
            major: major.to_string(),
            versions,
        });
    }

    // 遍历handler 中的每个元素
    Ok(engine_list)
}

pub fn list_remote_engine_tags(data: &Path, version: &str) -> Result<Vec<Value>, Box<dyn Error>> {
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

    Ok(assets.document)
}

fn load_remote_engines_handler(
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

fn get_major(tag_name: &str) -> String {
    let major = tag_name.split(".");
    let major = major.collect::<Vec<&str>>()[0];
    format!("{}.x", major)
}

