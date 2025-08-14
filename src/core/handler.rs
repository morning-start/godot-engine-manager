use super::utils::load_json; // 引用项目中已有的JSON加载函数
use serde_json::Value;
use std::{error::Error, option::Option, path::Path};

/// JSON文档处理器，提供加载、查询和字段重命名功能
#[derive(Debug, Clone)]
pub struct DocumentHandler {
    pub document: Vec<Value>,
}

impl DocumentHandler {
    /// 创建新的文档处理器实例
    pub fn new(document: Vec<Value>) -> Self {
        Self { document }
    }

    /// 从JSON文件加载数据并创建处理器实例
    pub fn load_data(file_path: &Path) -> Result<Self, Box<dyn Error>> {
        let document = load_json(file_path)?;
        Ok(Self::new(document.as_array().unwrap().to_vec()))
    }
}

impl DocumentHandler {
    /// 根据键值对查询数据，返回新的处理器实例
    pub fn query(&self, key: Option<&str>, value: Option<&Value>) -> Result<Self, Box<dyn Error>> {
        // 如果未提供键或值，返回当前实例的克隆
        if key.is_none() || value.is_none() {
            return Ok(self.clone());
        }
        let key = key.unwrap();
        let value = value.unwrap();
        // 筛选符合条件的元素
        let filtered: Vec<Value> = self
            .document
            .iter()
            .filter(|item| item.as_object().and_then(|obj| obj.get(key)) == Some(value))
            .cloned()
            .collect();

        Ok(Self::new(filtered))
    }

    /// 根据名称映射重命名字段
    pub fn rename(
        &mut self,
        name_map: &std::collections::HashMap<String, String>,
    ) -> Result<(), Box<dyn Error>> {
        // 遍历每个对象并重命名字段
        for item in &mut self.document {
            if let Some(obj) = item.as_object_mut() {
                for (old_name, new_name) in name_map {
                    if let Some(value) = obj.remove(old_name) {
                        obj.insert(new_name.clone(), value);
                    }
                }
            }
        }

        Ok(())
    }
    /// 根据指定字段顺序重排文档中的对象字段
    pub fn orderby(&mut self, levels: &[&str]) -> Result<Vec<Value>, Box<dyn Error>> {
        let array = &mut self.document;
        let mut new_doc = Vec::with_capacity(array.len());

        for item in array.drain(..) {
            let mut new_item = serde_json::Map::new();
            if let Some(obj) = item.as_object() {
                for &level in levels {
                    if let Some(value) = obj.get(level) {
                        new_item.insert(level.to_string(), value.clone());
                    }
                }
            }
            new_doc.push(Value::Object(new_item));
        }
        Ok(new_doc)
    }

    /// 根据指定键对文档进行排序
    pub fn sort(&mut self, key: &str, reverse: bool) -> Result<(), Box<dyn Error>> {
        let array = &mut self.document;

        array.sort_by(|a, b| {
            let a_val = a.as_object().and_then(|obj| obj.get(key));
            let b_val = b.as_object().and_then(|obj| obj.get(key));
            // TODO 处理不同类型的字段
            match (a_val, b_val) {
                (Some(a), Some(b)) => a.as_str().cmp(&b.as_str()),
                (Some(_), None) => std::cmp::Ordering::Greater,
                (None, Some(_)) => std::cmp::Ordering::Less,
                (None, None) => std::cmp::Ordering::Equal,
            }
        });

        if reverse {
            array.reverse();
        }

        Ok(())
    }

    /// 对文档中的每个元素应用转换函数
    pub fn map<F>(&mut self, f: F) -> Result<(), Box<dyn Error>>
    where
        F: Fn(Value) -> Value,
    {
        let array = &mut self.document;

        for item in array {
            *item = f(item.take());
        }

        Ok(())
    }

    /// 对文档中指定键的值应用转换函数
    pub fn apply<F>(&mut self, key: &str, f: F) -> Result<(), Box<dyn Error>>
    where
        F: Fn(Value) -> Value,
    {
        let array = &mut self.document;

        for item in array {
            if let Some(obj) = item.as_object_mut() {
                if let Some(value) = obj.remove(key) {
                    obj.insert(key.to_string(), f(value));
                }
            }
        }

        Ok(())
    }

    /// 根据条件筛选文档元素，返回新的处理器实例
    pub fn flitter<F>(&self, condition: F) -> Result<Self, Box<dyn Error>>
    where
        F: Fn(&Value) -> bool,
    {
        let array = &self.document;

        let filtered: Vec<Value> = array
            .iter()
            .filter(|&item| condition(item))
            .cloned()
            .collect();

        Ok(Self::new(filtered))
    }

    /// 获取文档中指定字段的信息，返回新的处理器实例
    pub fn get_specific_fields(&self, fields: &[&str]) -> Result<Self, Box<dyn Error>> {
        let array = &self.document;

        let mut new_doc = Vec::with_capacity(array.len());

        for item in array {
            let obj = item.as_object().ok_or("Array element is not an object")?;
            let mut new_item = serde_json::Map::new();

            for &field in fields {
                if let Some(value) = obj.get(field) {
                    new_item.insert(field.to_string(), value.clone());
                }
            }

            new_doc.push(Value::Object(new_item));
        }

        Ok(Self::new(new_doc))
    }

    pub fn group_by<F>(&self, key: &str, agg_map: Option<F>) -> Result<Self, Box<dyn Error>>
    where
        F: Fn(Vec<Value>) -> Vec<Value>,
    {
        // 确保文档是数组类型
        let array = &self.document;

        // 创建一个 HashMap 来存储分组后的数据
        let mut grouped_data: std::collections::HashMap<String, serde_json::Map<String, Value>> =
            std::collections::HashMap::new();

        // 遍历数组中的每个元素
        for item in array {
            // 确保元素是对象类型
            let obj = item.as_object().ok_or("Array element is not an object")?;

            // 获取用于分组的键值
            let key_value = obj
                .get(key)
                .ok_or(format!("Key '{}' not found in object", key))?;
            let key_str = key_value
                .as_str()
                .ok_or(format!("Key '{}' is not a string", key))?
                .to_string();

            // 如果键值不在分组数据中，则初始化
            if !grouped_data.contains_key(&key_str) {
                let mut new_group = serde_json::Map::new();
                new_group.insert(key.to_string(), key_value.clone());
                grouped_data.insert(key_str.clone(), new_group);
            }

            // 遍历对象中的每个键值对
            for (k, v) in obj {
                // 跳过分组键
                if k != key {
                    // 如果键不在分组数据中，则初始化为空数组
                    if !grouped_data.get(&key_str).unwrap().contains_key(k) {
                        grouped_data
                            .get_mut(&key_str)
                            .unwrap()
                            .insert(k.clone(), Value::Array(vec![]));
                    }

                    // 将值添加到数组中
                    if let Some(arr) = grouped_data.get_mut(&key_str).unwrap().get_mut(k) {
                        if let Some(arr) = arr.as_array_mut() {
                            arr.push(v.clone());
                        }
                    }
                }
            }
        }

        // 如果提供了聚合函数，则应用它
        if let Some(agg_fn) = agg_map {
            for group in grouped_data.values_mut() {
                for (k, v) in group.clone() {
                    if k != key {
                        if let Some(arr) = v.as_array() {
                            let new_arr = agg_fn(arr.clone());
                            group.insert(k, Value::Array(new_arr));
                        }
                    }
                }
            }
        }

        // 将分组数据转换为 Vec<Value>
        let result: Vec<Value> = grouped_data.into_values().map(Value::Object).collect();

        Ok(Self::new(result))
    }
}
