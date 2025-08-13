use crate::core::handler::DocumentHandler;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;

pub fn list_local_engines(home: &Path) -> Result<HashMap<String, Vec<String>>, Box<dyn Error>> {
    // {'4.x':['4.0','4.1']}
    let mut engine_map: HashMap<String, Vec<String>> = HashMap::new();
    // 如果 home 目录是空的，返回空的Vec
    if home.iter().next().is_none() {
        return Ok(engine_map);
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
                engine_map.insert(major_name, major_list);
            }
        }
        return Ok(engine_map);
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]

struct EngineInfo {
    tag_name: String,
    published_at: String,
}
impl EngineInfo {
    pub fn get_major(&self) -> String {
        let tag_name: String = self.tag_name.clone();
        let major = tag_name.split(".");
        let major = major.collect::<Vec<&str>>()[0];
        // 4.x
        format!("{}.x", major)
    }
}

pub fn list_remote_engines(data: &Path) -> Result<HashMap<String, Vec<String>>, Box<dyn Error>> {
    let mut engine_map: HashMap<String, Vec<String>> = HashMap::new();
    let file_path = data.join("releases.json");
    let handler = DocumentHandler::load_data(&file_path)?;
    let fields = ["tag_name", "published_at"];
    let handler = handler.get_specific_fields(fields.as_slice())?;
    let engine_info_list: Vec<EngineInfo> = serde_json::from_value(handler.document().clone())?;
    // 每一个major 都有一个Vec<String>
    for engine_info in engine_info_list {
        let major = engine_info.get_major();
        let version = engine_info.tag_name;
        engine_map.entry(major).or_insert(Vec::new()).push(version);
    }
    // 对每个Vec<String> 进行倒叙排序
    for (_, version_list) in engine_map.iter_mut() {
        version_list.sort_by(|a, b| b.cmp(a));
    }


    return Ok(engine_map);
}
