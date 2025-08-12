pub struct Tag {
    keywords: &'static [&'static str],
    labels: &'static str,
}
// 输入一个字符串，根据keywords判断是否包含
impl Tag {
    pub const fn new(keywords: &'static [&'static str], labels: &'static str) -> Self {
        Self { keywords, labels }
    }
    pub fn is_match(&self, input: &str) -> bool {
        self.keywords.iter().any(|keyword| input.contains(keyword))
    }
    pub fn get_labels(&self) -> &'static str {
        self.labels
    }
    pub fn get_keywords(&self) -> &'static [&'static str] {
        self.keywords
    }
}
// 标签列表
pub struct TagList {
    tags: &'static [Tag],
}
impl TagList {
    pub const fn new(tags: &'static [Tag]) -> Self {
        Self { tags }
    }
    pub fn get_tags(&self) -> &'static [Tag] {
        self.tags
    }
    pub fn get_tag_by_labels(&self, labels: &str) -> Option<&Tag> {
        self.tags.iter().find(|tag| tag.get_labels() == labels)
    }
    pub fn get_tags_by_input(&self, input: &str) -> Vec<&Tag> {
        self.tags.iter().filter(|tag| tag.is_match(input)).collect()
    }
}

pub const TAGS: TagList = TagList::new(&[
    Tag::new(&["windows"], "Windows"),
    Tag::new(&["win64"], "Windows X86 64位"),
    Tag::new(&["win32"], "Windows X86 32位"),
    Tag::new(&["linux"], "Linux"),
    Tag::new(&["x11_64", "x11.64"], "Linux X86 64位"),
    Tag::new(&["x11_32", "x11.32"], "Linux X86 32位"),
    Tag::new(&["headless.64", "headless_64"], "X86 64位 无头版"),
    Tag::new(&["server.64", "server_64"], "X86 64位"),
    Tag::new(&["macos", "osx"], "macOS"),
    Tag::new(&["osx32"], "32位"),
    Tag::new(&["osx64"], "64位"),
    Tag::new(&["android_editor"], "Android"),
    Tag::new(&["horizonos"], "Horizon"),
    Tag::new(&["picoos"], "Pico"),
    Tag::new(&["web_editor"], "Web编辑器"),
    Tag::new(&["arm32"], "ARM 32位"),
    Tag::new(&["arm64"], "ARM 64位"),
    Tag::new(&["x86_32"], "X86 32位"),
    Tag::new(&["x86_64"], "X86 64位"),
    Tag::new(&["universal"], "Universal"),
    Tag::new(&["server"], "服务器"),
    Tag::new(&[".aar"], "AAR库"),
    Tag::new(&[".aab"], "aab"),
    Tag::new(&[".apk"], "apk"),
    Tag::new(&["export_templates"], "导出模板"),
    Tag::new(&["mono"], "C#"),
    Tag::new(&[".tar.xz"], "源代码"),
    Tag::new(&[".sha256"], "校验文件"),
]);
