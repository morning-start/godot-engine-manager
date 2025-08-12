pub enum Source {
    GodotHub,
    GodotEngine,
}

impl Source {
    pub fn get_domain(&self) -> &str {
        match self {
            Source::GodotHub => "gitcode.com/godothub",
            Source::GodotEngine => "github.com/godotengine",
        }
    }
}

pub fn format_url(url: &str, source: Option<Source>) -> String {
    let source = source.unwrap_or(Source::GodotHub);
    let domain = source.get_domain();
    url.replace(Source::GodotEngine.get_domain(), domain)
}
