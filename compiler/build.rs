fn main() {
    // نتحقق من نظام التشغيل
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set_icon("Logo.ico");
        
        // بدلاً من unwrap() التي تسبب الانهيار، سنستخدم match
        match res.compile() {
            Ok(_) => println!("cargo:warning=Icon compiled successfully!"),
            Err(_) => {
                // إذا فشل، اطبع تحذير فقط وأكمل البناء
                println!("cargo:warning=Windres not found. Executable built without icon.");
            }
        }
    }
}