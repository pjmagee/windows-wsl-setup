fn main() {
    println!("cargo:rerun-if-changed=assets/app.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut res = winresource::WindowsResource::new();
    res.set_icon("assets/app.ico");
    res.set("ProductName", "Windows WSL Manager");
    res.set(
        "FileDescription",
        "Collect, restore, New WSL, and named software profiles",
    );
    res.set("OriginalFilename", "wwm.exe");
    res.set("InternalName", "wwm");
    res.compile().expect("embed Windows resources");
}
