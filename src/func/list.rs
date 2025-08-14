use crate::core::handler::{self, DocumentHandler};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct Engine {
    pub major: String,
    pub versions: Vec<String>,
}

pub fn list_local_engines(home: &Path) -> Result<Vec<Engine>, Box<dyn Error>> {
    // {'4.x':['4.0','4.1']} --> [{ 'major':'4.x','versions':['4.0','4.1']}]
    let mut engine_list: Vec<Engine> = Vec::new();
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
                engine_list.push(Engine {
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

pub fn list_remote_engines(data: &Path) -> Result<Vec<Engine>, Box<dyn Error>> {
    let mut engine_list: Vec<Engine> = Vec::new();
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
        engine_list.push(Engine {
            major: major.to_string(),
            versions,
        });
    }

    // 遍历handler 中的每个元素
    Ok(engine_list)
}

// pub fn list_remote_engine_tags(
//     data: &Path,
//     version: &str,
// ) -> Result<HashMap<String, Vec<String>>, Box<dyn Error>> {
//     let mut tags_map: HashMap<String, Vec<String>> = HashMap::new();
//     let handler = load_remote_engines_handler(data, &["tag_name", "assets"])?;
//     //handler flitter tag_name 中 name 包含 version 的
//     let handler = handler.flitter(|v| v["tag_name"].as_str().unwrap().contains(version))?;
//     // 从中找到最新版本
//     let latest_version = handler.document[0]["tag_name"]
//         .as_str()
//         .unwrap();
//     // 从最新版本中找到assets
//     let assets = handler.document[0]["assets"]
//         .as_array()
//         .unwrap();
//     let assets = DocumentHandler::new(assets.clone());
// }

fn load_remote_engines_handler(
    data: &Path,
    fields: &[&str],
) -> Result<DocumentHandler, Box<dyn Error>> {
    let file_path = data.join("releases.json");
    let handler = DocumentHandler::load_data(&file_path)?;
    let handler = handler.get_specific_fields(fields)?;
    Ok(handler)
}

fn get_major(tag_name: &str) -> String {
    let major = tag_name.split(".");
    let major = major.collect::<Vec<&str>>()[0];
    format!("{}.x", major)
}
