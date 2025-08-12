fn main() {
    // https://godothub.atomgit.net/web/api/releases.json
    // browser_download_url.replace("github.com/godotengine", "gitcode.com/godothub"),
    let url = "https://github.com/godotengine/godot/releases/download/4.4.1-stable/Godot_v4.4.1-stable_mono_win64.zip";
    let url = url.replace("github.com/godotengine", "gitcode.com/godothub");
    println!("{}", url);
}
