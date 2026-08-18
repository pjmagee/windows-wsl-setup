use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::model::WingetApp;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct KitSelections {
    #[serde(default)]
    pub apps: Vec<String>,
    #[serde(default)]
    pub wsl: Vec<String>,
    #[serde(default)]
    pub dev_drive: bool,
    #[serde(default)]
    pub docker_data: bool,
    #[serde(default)]
    pub browser: bool,
    #[serde(default)]
    pub dotfiles: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct KitWsl {
    pub name: String,
    #[serde(default)]
    pub vhdx: Option<String>,
    #[serde(default)]
    pub on_dev_drive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct KitDevDrive {
    #[serde(default)]
    pub keep: bool,
    #[serde(default)]
    pub live_path: Option<String>,
    #[serde(default)]
    pub copied_to: Option<String>,
    #[serde(default)]
    pub letter: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct KitDocument {
    #[serde(default)]
    pub schema_version: u32,
    #[serde(default)]
    pub computer: String,
    #[serde(default)]
    pub user: String,
    #[serde(default)]
    pub linux_profile: String,
    #[serde(default)]
    pub kit_root: String,
    #[serde(default)]
    pub selections: KitSelections,
    #[serde(default)]
    pub wsl_disks: Vec<KitWsl>,
    #[serde(default)]
    pub dev_drive_disk: Option<KitDevDrive>,
}

#[derive(Debug, Clone)]
pub struct LoadedKit {
    pub dir: PathBuf,
    pub doc: KitDocument,
    pub apps: Vec<WingetApp>,
}

pub fn find_kits() -> Vec<PathBuf> {
    let mut out = Vec::new();
    for letter in 'D'..='Z' {
        if letter == 'C' {
            continue;
        }
        let backups = PathBuf::from(format!("{letter}:\\Backups"));
        if backups.is_dir() {
            if let Ok(rd) = fs::read_dir(&backups) {
                for e in rd.flatten() {
                    let kit = e.path().join("KIT.json");
                    if kit.is_file() {
                        out.push(e.path());
                    }
                }
            }
        }
        let root_kit = PathBuf::from(format!("{letter}:\\KIT.json"));
        if root_kit.is_file() {
            out.push(PathBuf::from(format!("{letter}:\\")));
        }
    }
    out.sort();
    out
}

pub fn load_kit(dir: &Path) -> Result<LoadedKit, String> {
    let doc_path = dir.join("KIT.json");
    let raw = fs::read_to_string(&doc_path).map_err(|e| format!("read {}: {e}", doc_path.display()))?;
    let doc: KitDocument =
        serde_json::from_str(&raw).map_err(|e| format!("KIT.json: {e}"))?;
    let mut apps = Vec::new();
    let apps_json = dir.join("inventory/apps.json");
    if apps_json.is_file() {
        if let Ok(v) = serde_json::from_str::<Vec<WingetApp>>(&fs::read_to_string(apps_json).unwrap_or_default())
        {
            apps = v;
        }
    }
    if apps.is_empty() {
        let sel = dir.join("apps/winget-selected.json");
        if let Ok(raw) = fs::read_to_string(sel) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
                if let Some(sources) = v.get("Sources").and_then(|s| s.as_array()) {
                    for src in sources {
                        if let Some(pkgs) = src.get("Packages").and_then(|p| p.as_array()) {
                            for p in pkgs {
                                if let Some(id) = p.get("PackageIdentifier").and_then(|x| x.as_str()) {
                                    apps.push(WingetApp {
                                        id: id.to_string(),
                                        name: id.to_string(),
                                        version: String::new(),
                                        keep: true,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    for a in &mut apps {
        a.keep = true;
    }
    Ok(LoadedKit {
        dir: dir.to_path_buf(),
        doc,
        apps,
    })
}
