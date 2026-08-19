use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Destination {
    pub letter: char,
    pub label: String,
    pub file_system: String,
    pub gb: u64,
    pub free_gb: f64,
    pub guid: Option<String>,
    pub suggested: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WingetApp {
    pub id: String,
    pub name: String,
    pub version: String,
    pub keep: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WslDistro {
    pub name: String,
    pub version: u32,
    pub base_path: String,
    pub vhdx: Option<String>,
    pub gb: f64,
    pub keep: bool,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VhdCandidate {
    pub path: String,
    pub gb: f64,
    pub on_c: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevDrive {
    pub present: bool,
    pub letter: Option<char>,
    pub label: String,
    pub vhdx: Vec<VhdCandidate>,
    pub keep: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerData {
    pub present: bool,
    pub path: String,
    pub gb: f64,
    pub keep: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BraveExt {
    pub id: String,
    pub name: String,
    pub version: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BraveInfo {
    pub present: bool,
    pub bookmarks_path: Option<String>,
    pub bookmarks_bytes: u64,
    pub extensions: Vec<BraveExt>,
    pub keep: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dotfile {
    pub key: String,
    pub path: String,
    pub present: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinuxTool {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub pkg: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub layer: String,
    #[serde(default)]
    pub home: bool,
    #[serde(default)]
    pub work: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinuxToolsFile {
    #[serde(default)]
    pub schema_version: u32,
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    pub tools: Vec<LinuxTool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    pub computer: String,
    pub user: String,
    pub user_profile: String,
    pub destinations: Vec<Destination>,
    pub apps: Vec<WingetApp>,
    pub wsl: Vec<WslDistro>,
    pub dev_drive: DevDrive,
    pub docker: DockerData,
    pub brave: BraveInfo,
    pub dotfiles: Vec<Dotfile>,
    pub linux_tools: LinuxToolsFile,
}

pub fn default_keep_app(id: &str) -> bool {
    const SKIP: &[&str] = &[
        "Python.Python",
        "Python.Launcher",
        "Rustlang.",
        "Microsoft.OpenJDK",
        "Microsoft.DotNet.SDK",
        "Microsoft.Azure",
        "OpenJS.NodeJS",
        "Microsoft.VCRedist",
        "Microsoft.UI.Xaml",
        "Microsoft.WindowsAppRuntime",
        "Microsoft.VCLibs",
        "Microsoft.Edge",
        "Microsoft.OneDrive",
        "Microsoft.AppInstaller",
        "Microsoft.WSL",
        "Microsoft.Teams",
        "Microsoft.WindowsApp",
        "Microsoft.DotNet.Native",
    ];
    !SKIP.iter().any(|p| id.starts_with(p) || id == *p)
}

pub fn repo_root() -> Option<std::path::PathBuf> {
    if let Ok(p) = std::env::var("WSL_SETUP_ROOT") {
        let p = std::path::PathBuf::from(p);
        if p.join("profiles/linux.json").is_file() {
            return Some(p);
        }
    }
    let mut dirs = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(d) = exe.parent() {
            dirs.push(d.to_path_buf());
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        dirs.push(cwd);
    }
    for start in dirs {
        let mut cur = start;
        for _ in 0..8 {
            if cur.join("profiles/linux.json").is_file() {
                return Some(cur);
            }
            if !cur.pop() {
                break;
            }
        }
    }
    None
}
