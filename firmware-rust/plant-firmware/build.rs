fn main() {
    embuild::espidf::sysenv::output();

    // Ensure src/secrets.rs exists for fresh clones & CI builds (ignored in git)
    let secrets_path = std::path::Path::new("src/secrets.rs");
    if !secrets_path.exists() {
        let _ = std::fs::write(
            secrets_path,
            "pub const WIFI_SSID: &str = \"\";\npub const WIFI_PASSWORD: &str = \"\";\n",
        );
    }
}
