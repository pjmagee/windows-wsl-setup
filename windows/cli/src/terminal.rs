//! Windows Terminal is the host console WWM supports.
//!
//! Official distro tabs come from **Microsoft.WSL** (icons, names, `--cd`).
//! WWM must not overwrite their `commandline` — that is what broke Debian
//! (`/home/patri: Is a directory`). We only:
//! - disable the *legacy* `Windows.Terminal.Wsl` generator
//! - add a **wsl** launcher that follows the WSL default
//!
//! Adding/removing a distro is enough for Microsoft.WSL to add/drop its tab.

use std::fs;
use std::path::PathBuf;

use serde::Serialize;
use serde_json::{json, Value};

use crate::new_wsl;

/// Legacy Terminal generator. Microsoft.WSL is the current, proper source.
const LEGACY_WSL_SOURCE: &str = "Windows.Terminal.Wsl";
const OFFICIAL_WSL_SOURCE: &str = "Microsoft.WSL";
const WWM_SOURCE: &str = "wwm";

/// Bare `wsl` tab — follows the WSL default distro (`wsl.exe ~`).
pub const WSL_GUID: &str = "{8f3e1c2a-9b74-4d6e-a1f0-2c8e4e6b90bb}";

const PENGUIN: &str = "ms-appx:///ProfileIcons/{9acb9455-ca41-5af7-950f-6bca1bc9722f}.png";

#[derive(Debug, Clone, Serialize)]
pub struct SyncReport {
    pub ok: bool,
    pub profiles: Vec<String>,
    pub fragment: String,
    pub default_profile: Option<String>,
    pub detail: String,
}

pub fn skip_distro(name: &str) -> bool {
    name.trim().is_empty() || new_wsl::is_helper_distro(name)
}

pub fn fragment_dir() -> Option<PathBuf> {
    let local = std::env::var_os("LOCALAPPDATA")?;
    Some(PathBuf::from(local).join(r"Microsoft\Windows Terminal\Fragments\wwm"))
}

fn settings_paths() -> Vec<PathBuf> {
    let Some(local) = std::env::var_os("LOCALAPPDATA") else {
        return Vec::new();
    };
    let local = PathBuf::from(local);
    [
        local.join(r"Packages\Microsoft.WindowsTerminal_8wekyb3d8bbwe\LocalState\settings.json"),
        local.join(
            r"Packages\Microsoft.WindowsTerminalPreview_8wekyb3d8bbwe\LocalState\settings.json",
        ),
        local.join(r"Microsoft\Windows Terminal\settings.json"),
    ]
    .into_iter()
    .collect()
}

fn existing_settings() -> Vec<PathBuf> {
    settings_paths()
        .into_iter()
        .filter(|p| p.is_file())
        .collect()
}

fn wsl_exe() -> String {
    let root = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".into());
    format!(r"{root}\System32\wsl.exe")
}

fn launcher_profile() -> Value {
    json!({
        "guid": WSL_GUID,
        "name": "wsl",
        "commandline": wsl_exe(),
        "startingDirectory": "~",
        "hidden": false,
        "icon": PENGUIN,
    })
}

/// Keep official Microsoft.WSL tabs. Do not overwrite their command line.
pub fn sync(set_default: Option<&str>) -> Result<SyncReport, String> {
    let mut names: Vec<String> = new_wsl::distro_names()
        .into_iter()
        .filter(|n| !skip_distro(n))
        .collect();
    names.sort_by_key(|a| a.to_ascii_lowercase());

    let profiles = vec![launcher_profile()];

    let dir = fragment_dir().ok_or_else(|| "LOCALAPPDATA missing".to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let old = dir.parent().map(|p| p.join("wsl-setup"));
    if let Some(old) = old {
        if old.is_dir() {
            let _ = fs::remove_dir_all(&old);
        }
    }
    let path = dir.join("profiles.json");
    let body = json!({ "profiles": profiles });
    let raw = serde_json::to_string_pretty(&body).map_err(|e| e.to_string())? + "\n";
    fs::write(&path, raw).map_err(|e| format!("write {}: {e}", path.display()))?;

    let mut default_profile = None;
    let default_guid = set_default.and_then(official_guid);
    patch_settings(default_guid.as_deref())?;
    if let Some(d) = set_default {
        if names.iter().any(|n| n.eq_ignore_ascii_case(d)) {
            default_profile = Some(d.to_string());
        }
    }

    Ok(SyncReport {
        ok: true,
        profiles: names,
        fragment: path.display().to_string(),
        default_profile,
        detail: "open a new Windows Terminal window to load profiles".into(),
    })
}

fn official_guid(name: &str) -> Option<String> {
    if let Some(g) = guid_from_settings(name) {
        return Some(g);
    }
    lxss_guid(name)
}

fn guid_from_settings(name: &str) -> Option<String> {
    for path in existing_settings() {
        let Ok(raw) = fs::read_to_string(&path) else {
            continue;
        };
        let Ok(v) = serde_json::from_str::<Value>(&raw) else {
            continue;
        };
        let Some(list) = v.pointer("/profiles/list").and_then(|x| x.as_array()) else {
            continue;
        };
        for p in list {
            let source = p.get("source").and_then(|s| s.as_str()).unwrap_or("");
            if source != OFFICIAL_WSL_SOURCE {
                continue;
            }
            let n = p.get("name").and_then(|s| s.as_str()).unwrap_or("");
            if n.eq_ignore_ascii_case(name) {
                if let Some(g) = p.get("guid").and_then(|s| s.as_str()) {
                    return Some(g.to_string());
                }
            }
        }
    }
    None
}

#[cfg(windows)]
fn lxss_guid(name: &str) -> Option<String> {
    use winreg::enums::*;
    use winreg::RegKey;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let lxss = hkcu
        .open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Lxss")
        .ok()?;
    for sub in lxss.enum_keys().filter_map(|k| k.ok()) {
        if !sub.starts_with('{') {
            continue;
        }
        let Ok(key) = lxss.open_subkey(&sub) else {
            continue;
        };
        let Ok(dist) = key.get_value::<String, _>("DistributionName") else {
            continue;
        };
        if dist.eq_ignore_ascii_case(name) {
            return Some(sub);
        }
    }
    None
}

#[cfg(not(windows))]
fn lxss_guid(_: &str) -> Option<String> {
    None
}

fn patch_settings(default_guid: Option<&str>) -> Result<(), String> {
    let paths = existing_settings();
    if paths.is_empty() {
        return Ok(());
    }
    for path in paths {
        let Ok(raw) = fs::read_to_string(&path) else {
            continue;
        };
        let next = if let Ok(mut v) = serde_json::from_str::<Value>(&raw) {
            keep_official_wsl_tabs(&mut v);
            if let Some(guid) = default_guid {
                if let Some(obj) = v.as_object_mut() {
                    obj.insert("defaultProfile".into(), json!(guid));
                }
            }
            serde_json::to_string_pretty(&v).map_err(|e| e.to_string())? + "\n"
        } else {
            let mut next = ensure_legacy_wsl_disabled(&raw);
            next = strip_microsoft_wsl_disabled(&next);
            if let Some(guid) = default_guid {
                next = set_default_profile(&next, guid);
            }
            next
        };
        if next != raw {
            fs::write(&path, next).map_err(|e| format!("{}: {e}", path.display()))?;
        }
    }
    Ok(())
}

/// Official Microsoft.WSL tabs stay. Hide leftover WWM clones. Disable only the old generator.
fn keep_official_wsl_tabs(v: &mut Value) {
    let mut sources: Vec<String> = Vec::new();
    if let Some(arr) = v.get("disabledProfileSources").and_then(|x| x.as_array()) {
        for x in arr {
            if let Some(s) = x.as_str() {
                if s != OFFICIAL_WSL_SOURCE && !sources.iter().any(|e| e == s) {
                    sources.push(s.to_string());
                }
            }
        }
    }
    if !sources.iter().any(|e| e == LEGACY_WSL_SOURCE) {
        sources.push(LEGACY_WSL_SOURCE.into());
    }
    if let Some(obj) = v.as_object_mut() {
        obj.insert("disabledProfileSources".into(), json!(sources));
    }
    let Some(list) = v
        .pointer_mut("/profiles/list")
        .and_then(|x| x.as_array_mut())
    else {
        return;
    };
    for p in list {
        let source = p
            .get("source")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();
        let guid = p
            .get("guid")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();
        let Some(obj) = p.as_object_mut() else {
            continue;
        };
        if source == OFFICIAL_WSL_SOURCE {
            obj.insert("hidden".into(), json!(false));
        } else if source == WWM_SOURCE && guid != WSL_GUID {
            obj.insert("hidden".into(), json!(true));
        }
    }
}

fn ensure_legacy_wsl_disabled(raw: &str) -> String {
    if raw.contains(LEGACY_WSL_SOURCE) {
        return raw.to_string();
    }
    if raw.contains("disabledProfileSources") {
        return raw.to_string();
    }
    if let Some(i) = raw.find('{') {
        let mut s = raw.to_string();
        s.insert_str(
            i + 1,
            "\n    \"disabledProfileSources\": [ \"Windows.Terminal.Wsl\" ],",
        );
        return s;
    }
    raw.to_string()
}

fn strip_microsoft_wsl_disabled(raw: &str) -> String {
    raw.replace(", \"Microsoft.WSL\"", "")
        .replace("\"Microsoft.WSL\", ", "")
        .replace("\"Microsoft.WSL\",", "")
}

fn set_default_profile(raw: &str, guid: &str) -> String {
    let repl = format!("\"defaultProfile\": \"{guid}\"");
    if raw.contains("\"defaultProfile\"") {
        return regex_replace_default(raw, &repl);
    }
    if let Some(i) = raw.find('{') {
        let mut s = raw.to_string();
        s.insert_str(i + 1, &format!("\n    {repl},"));
        return s;
    }
    raw.to_string()
}

fn regex_replace_default(raw: &str, repl: &str) -> String {
    if let Some(at) = raw.find("\"defaultProfile\"") {
        let mut out = String::with_capacity(raw.len());
        out.push_str(&raw[..at]);
        let rest = &raw[at + "\"defaultProfile\"".len()..];
        let s = rest.trim_start_matches(|c: char| c == ' ' || c == '\t');
        let s = s.strip_prefix(':').unwrap_or(s).trim_start();
        if s.starts_with('"') {
            if let Some(end) = s[1..].find('"') {
                out.push_str(repl);
                out.push_str(&s[end + 2..]);
                return out;
            }
        }
    }
    raw.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn docker_is_skipped() {
        assert!(skip_distro("docker-desktop"));
        assert!(skip_distro("docker-desktop-data"));
        assert!(!skip_distro("Debian"));
        assert!(!skip_distro("Ubuntu-26.04"));
    }

    #[test]
    fn default_profile_replace() {
        let src = "{\n  \"defaultProfile\": \"{old}\",\n  \"profiles\": {}\n}";
        let got = set_default_profile(src, WSL_GUID);
        assert!(got.contains(&format!("\"defaultProfile\": \"{WSL_GUID}\"")));
        assert!(!got.contains("{old}"));
    }

    #[test]
    fn keeps_microsoft_wsl_hides_wwm_clones() {
        let mut v: Value = serde_json::from_str(
            r#"{
            "disabledProfileSources": ["Windows.Terminal.Wsl", "Microsoft.WSL"],
            "profiles": { "list": [
                { "guid": "{aaa}", "name": "Debian", "source": "Microsoft.WSL", "hidden": true },
                { "guid": "{5a12a748-0df3-5211-b86e-667fd155e1c1}", "name": "Debian", "source": "wwm", "hidden": false },
                { "guid": "{8f3e1c2a-9b74-4d6e-a1f0-2c8e4e6b90bb}", "name": "wsl", "source": "wwm", "hidden": false },
                { "name": "PowerShell", "source": "Windows.Terminal.PowershellCore", "hidden": false }
            ] }
        }"#,
        )
        .unwrap();
        keep_official_wsl_tabs(&mut v);
        let sources = v["disabledProfileSources"].as_array().unwrap();
        assert!(sources
            .iter()
            .any(|s| s.as_str() == Some("Windows.Terminal.Wsl")));
        assert!(!sources.iter().any(|s| s.as_str() == Some("Microsoft.WSL")));
        let list = v["profiles"]["list"].as_array().unwrap();
        assert_eq!(list[0]["hidden"], json!(false));
        assert_eq!(list[1]["hidden"], json!(true));
        assert_eq!(list[2]["hidden"], json!(false));
        assert_eq!(list[3]["hidden"], json!(false));
    }
}
