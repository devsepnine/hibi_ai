fn main() {
    #[cfg(feature = "gui")]
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        println!("cargo:rerun-if-changed=assets/hibi.ico");
        winresource::WindowsResource::new()
            .set_icon("assets/hibi.ico")
            .compile()
            .expect("embed the Hibi application icon");
    }
}
