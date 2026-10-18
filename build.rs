fn main() {
    println!("cargo:rerun-if-changed=assets/branding/icon.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        winresource::WindowsResource::new()
            .set_icon("assets/branding/icon.ico")
            .set("ProductName", "rusty_dlp")
            .set("FileDescription", "rusty_dlp download manager")
            .set("OriginalFilename", "rusty-dlp.exe")
            .compile()
            .expect("Windows icon and version resources must compile");
    }
}
