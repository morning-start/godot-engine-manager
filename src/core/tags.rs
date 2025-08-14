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
        let keyword = keyword.to_lowercase();
        if keyword.contains("mono") {
            Some(Self::CSharp)
        } else if keyword.contains("export_templates") {
            Some(Self::ExportTemplate)
        } else if keyword.contains("aar") {
            Some(Self::AARLib)
        } else {
            None
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

impl OS {
    pub fn get_local_os() -> Self {
        let sys_os = std::env::consts::OS;
        let sys_os = sys_os.to_lowercase();
        let os = Self::from_keyword(&sys_os).unwrap_or(Self::Windows); // 默认为Windows
        os
    }
}

impl Tag for OS {
    fn from_keyword(keyword: &str) -> Option<Self> {
        let keyword = keyword.to_lowercase();
        if keyword.contains("win") {
            Some(Self::Windows)
        } else if keyword.contains("linux") || keyword.contains("x11") {
            Some(Self::Linux)
        } else if keyword.contains("macos") || keyword.contains("osx") {
            Some(Self::MacOS)
        } else {
            None
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

impl Architecture {
    pub fn get_local_arch() -> Self {
        let arch = std::env::consts::ARCH;
        let arch = arch.to_lowercase();
        let arch = Self::from_keyword(&arch).unwrap_or(Self::AMD64); // 默认为AMD64
        arch
    }
}

impl Tag for Architecture {
    fn from_keyword(keyword: &str) -> Option<Self> {
        let keyword = keyword.to_lowercase();
        if keyword.contains(".64")
            || keyword.contains("_64")
            || keyword.contains("win64")
            || keyword.contains("osx64")
        {
            Some(Self::AMD64)
        } else if keyword.contains(".32")
            || keyword.contains("_32")
            || keyword.contains("win32")
            || keyword.contains("osx32")
        {
            Some(Self::AMD32)
        } else if keyword.contains("arm64") {
            Some(Self::ARM64)
        } else if keyword.contains("arm32") {
            Some(Self::ARM32)
        } else if keyword.contains("universal") {
            Some(Self::Universal)
        } else {
            None
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

const SUPPORT_FILE: &[&str] = &[".aar", ".zip", ".txt", ".tpz"];

pub fn is_support_file(file_name: &str) -> bool {
    SUPPORT_FILE.iter().any(|&ext| file_name.contains(ext))
}
