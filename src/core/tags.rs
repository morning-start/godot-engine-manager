pub trait Tags {
    fn from_keyword(keyword: &str) -> Option<Self>
    where
        Self: Sized;
    fn to_keywords(&self) -> &'static [&'static str];
    fn get_labels(&self) -> &'static str;
}

pub enum Language {
    CSharp,
    GdScript,
    ExportTemplate,
    AARLib,
}
impl Tags for Language {
    fn from_keyword(keyword: &str) -> Option<Self> {
        match keyword {
            "mono" => Some(Self::CSharp),
            "gdscript" => Some(Self::GdScript),
            "export_templates" => Some(Self::ExportTemplate),
            "aar" => Some(Self::AARLib),
            _ => None,
        }
    }
    fn to_keywords(&self) -> &'static [&'static str] {
        match self {
            Self::CSharp => &["mono"],
            Self::GdScript => &["gdscript"],
            Self::ExportTemplate => &["export_templates"],
            Self::AARLib => &["aar"],
        }
    }
    fn get_labels(&self) -> &'static str {
        match self {
            Self::CSharp => "C#",
            Self::GdScript => "GdScript",
            Self::ExportTemplate => "导出模板",
            Self::AARLib => "AAR库",
        }
    }
}

pub enum OS {
    Windows,
    Linux,
    MacOS,
}

impl Tags for OS {
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

pub enum Architecture {
    AMD64,
    AMD32,
    ARM64,
    ARM32,
    Universal,
}
impl Tags for Architecture {
    fn from_keyword(keyword: &str) -> Option<Self> {
        match keyword {
            "64" => Some(Self::AMD64),
            "32" => Some(Self::AMD32),
            "arm64" => Some(Self::ARM64),
            "arm32" => Some(Self::ARM32),
            "universal" => Some(Self::Universal),
            _ => None,
        }
    }
    fn to_keywords(&self) -> &'static [&'static str] {
        match self {
            Self::AMD64 => &["64"],
            Self::AMD32 => &["32"],
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
