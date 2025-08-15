use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use serde_json::Value;
use tabled::{builder::Builder, settings::style::Style};

pub fn new_spinner() -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner());
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    pb
}

pub fn show_table(data: &Vec<Value>, title: &str) -> String {
    if data.is_empty() {
        return "Empty data".to_string();
    }

    // 确保所有元素都是 Object
    for item in data {
        if !item.is_object() {
            return "Error: All items in data must be JSON objects".to_string();
        }
    }

    // 使用 Vec 去重，保留首次出现顺序
    let mut columns = Vec::new();

    for item in data {
        if let Value::Object(map) = item {
            for key in map.keys() {
                if !columns.contains(key) {
                    columns.push(key.clone());
                }
            }
        }
    }

    let mut builder = Builder::new();
    // 添加表头
    builder.push_record(columns.iter().map(|s| s.as_str()));

    // 添加每一行数据
    for item in data {
        if let Value::Object(map) = item {
            let row = columns.iter().map(|col| {
                match map.get(col) {
                    Some(Value::String(s)) => s.clone(),
                    Some(Value::Null) => "".to_string(),
                    Some(v) => v.to_string(), // 其他类型转字符串（数字、bool 等）
                    None => "".to_string(),   // 缺失字段留空
                }
            });
            builder.push_record(row);
        }
    }

    let mut table = builder.build();

    // 使用现代风格样式
    table.with(Style::modern());

    // title 斜体
    // "\x1B[1;3m{}\x1B[0m", text
    let format_title = format!("\x1B[1;3m{}\x1B[0m", title);
    let table_str = table.to_string();
    // 标题居中
    let table_width = table.total_width();
    let title_width = title.len();
    let padding = (table_width - title_width) / 2;
    let centered_title = " ".repeat(padding) + format_title.as_str();

    // 添加标题
    format!("{}\n{}", centered_title, table_str)
}

pub fn show_tree(versions: &[String], current: &str, title: &str) -> String {
    // title 变灰色
    let title = title.truecolor(128, 128, 128).to_string();

    let mut output = format!("{}\n", title);
    let last_index = versions.len().saturating_sub(1); // 防止下溢

    for (index, version) in versions.iter().enumerate() {
        let mark = if version == current { "*" } else { " " };

        let text = format!("{}{}) {}", mark, index + 1, version);
        // 根据是否为当前版本来决定文本颜色
        let text = if version == current {
            text.green().to_string()
        } else {
            text.normal().to_string()
        };

        let prefix = if index == last_index {
            "└──"
        } else {
            "├──"
        };

        let line = format!("{} {}\n", prefix, text);
        output.push_str(&line);
    }

    output.trim_end().to_string()
}
