use colored::*;
use indicatif::{ProgressBar, ProgressStyle};

pub fn new_spinner() -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner());
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    pb
}

pub fn show_list(versions: &[String], title: &str) -> String {
     let title = title.truecolor(128, 128, 128).to_string();

    let mut output = format!("{}\n", title);
    let last_index = versions.len().saturating_sub(1); // 防止下溢

    for (index, version) in versions.iter().enumerate() {

        let text = format!("{}) {}", index + 1, version);
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
