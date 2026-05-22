fn main() {

    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/Logo.ico");
        

        match res.compile() {
            Ok(_) => println!("cargo:warning=Icon compiled successfully!"),
            Err(_) => {

                println!("cargo:warning=Windres not found. Executable built without icon.");
            }
        }
    }
}