fn main() {
    println!("cargo:rerun-if-changed=assets/app.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut res = winresource::WindowsResource::new();
    res.set_icon("assets/app.ico");
    res.set("ProductName", "Windows WSL Setup");
    res.set(
        "FileDescription",
        "Collect, restore, New WSL, and named software profiles",
    );
    res.set("OriginalFilename", "windows-wsl-setup.exe");
    res.set("InternalName", "windows-wsl-setup");
    res.compile().expect("embed Windows resources");
}
