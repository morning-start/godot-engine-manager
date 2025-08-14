use strum::IntoEnumIterator;
use strum_macros::EnumIter;

pub trait Tag {
    fn from_keyword(keyword: &str) -> Option<Self>
    where
        Self: Sized;
    fn to_keywords(&self) -> &'static [&'static str];
    fn get_labels(&self) -> &'static str;

    // 检查是否包含某个标签
    fn tag_in(&self, text: &str) -> bool
    where
        Self: Sized,
    {
        let keywords = self.to_keywords();
        if keywords.iter().any(|&k| text.contains(k)) {
            return true;
        }
        false
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
pub enum Language {
    CSharp,
    ExportTemplate,
    AARLib,
}
impl Tag for Language {
    fn from_keyword(keyword: &str) -> Option<Self> {
        match keyword {
            "mono" => Some(Self::CSharp),
            "export_templates" => Some(Self::ExportTemplate),
            "aar" => Some(Self::AARLib),
            _ => None,
        }
    }
    fn to_keywords(&self) -> &'static [&'static str] {
        match self {
            Self::CSharp => &["mono"],
            Self::ExportTemplate => &["export_templates"],
            Self::AARLib => &["aar"],
        }
    }
    fn get_labels(&self) -> &'static str {
        match self {
            Self::CSharp => "C#",
            Self::ExportTemplate => "导出模板",
            Self::AARLib => "AAR库",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
pub enum OS {
    Windows,
    Linux,
    MacOS,
}

impl Tag for OS {
    fn from_keyword(keyword: &str) -> Option<Self> {
        match keyword {
            "win" => Some(Self::Windows),
            "linux" | "x11" => Some(Self::Linux),
            "macos" | "osx" => Some(Self::MacOS),
            _ => None,
        }
    }
    fn to_keywords(&self) -> &'static [&'static str] {
        match self {
            Self::Windows => &["win"],
            Self::Linux => &["linux", "x11"],
            Self::MacOS => &["macos", "osx"],
        }
    }
    fn get_labels(&self) -> &'static str {
        match self {
            Self::Windows => "Windows",
            Self::Linux => "Linux",
            Self::MacOS => "macOS",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
pub enum Architecture {
    AMD64,
    AMD32,
    ARM64,
    ARM32,
    Universal,
}
impl Tag for Architecture {
    fn from_keyword(keyword: &str) -> Option<Self> {
        match keyword {
            ".64" | "_64" | "win64" | "osx64" => Some(Self::AMD64),
            ".32" | "_32" | "win32" | "osx32" => Some(Self::AMD32),
            "arm64" => Some(Self::ARM64),
            "arm32" => Some(Self::ARM32),
            "universal" => Some(Self::Universal),
            _ => None,
        }
    }
    fn to_keywords(&self) -> &'static [&'static str] {
        match self {
            Self::AMD64 => &[".64", "_64", "win64", "osx64"],
            Self::AMD32 => &[".32", "_32", "win32", "osx32"],
            Self::ARM64 => &["arm64"],
            Self::ARM32 => &["arm32"],
            Self::Universal => &["universal"],
        }
    }
    fn get_labels(&self) -> &'static str {
        match self {
            Self::AMD64 => "AMD64",
            Self::AMD32 => "AMD32",
            Self::ARM64 => "ARM64",
            Self::ARM32 => "ARM32",
            Self::Universal => "Universal",
        }
    }
}

pub fn get_tags(text: &str) -> Vec<String> {
    let mut tags: Vec<String> = Vec::new();
    for lang in Language::iter() {
        if lang.tag_in(text) {
            tags.push(lang.get_labels().to_string());
        }
    }
    for os in OS::iter() {
        if os.tag_in(text) {
            tags.push(os.get_labels().to_string());
        }
    }
    for arch in Architecture::iter() {
        if arch.tag_in(text) {
            tags.push(arch.get_labels().to_string());
        }
    }

    tags
}

const SUPPORT_EXTENSION: &[&str] = &[".aar", ".zip", ".tar.xz", ".sha256"];

pub fn is_support_extension(file_name: &str) -> bool {
    SUPPORT_EXTENSION.iter().any(|&ext| file_name.contains(ext))
}
