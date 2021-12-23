fn main() -> std::io::Result<()> {
    let os = std::env::var("CARGO_CFG_TARGET_OS").ok().unwrap();
    if os == "windows" {
        winres::WindowsResource::new()
            // This path can be absolute, or relative to your crate root.
            .set_icon("../assets/newspaper.ico")
            .compile()?;
    }
    Ok(())
}