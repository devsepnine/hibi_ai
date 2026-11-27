fn main() {
    #[cfg(feature = "gui")]
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        println!("cargo:rerun-if-changed=assets/hibi.ico");
        println!("cargo:rerun-if-changed=assets/hibi.manifest");
        winresource::WindowsResource::new()
            .set_icon("assets/hibi.ico")
            .set_manifest_file("assets/hibi.manifest")
            .compile()
            .expect("embed the Hibi application icon");
    }
}
