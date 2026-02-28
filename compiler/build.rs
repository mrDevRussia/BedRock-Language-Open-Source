fn main() {
    // Only attempt embedding on Windows
    if std::env::var("CARGO_CFG_TARGET_FAMILY").unwrap_or_default() != "windows" {
        return;
    }

    // Detect target env (msvc or gnu)
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();

    if target_env == "msvc" {
        // MSVC uses rc.exe from the Windows SDK or llvm-rc
        // embed-resource handles rc.exe automatically
        embed_resource::compile("bedrockc.rc", embed_resource::NONE);
        return;
    }

    // GNU toolchain: require windres (MinGW). If not present, skip icon.
    if std::process::Command::new("windres").arg("--version").output().is_ok() {
        embed_resource::compile("bedrockc.rc", embed_resource::NONE);
    } else {
        println!("cargo:warning=Icon embedding skipped on GNU toolchain: windres not found. Use MSVC toolchain for reliable icon embedding.");
    }
}
