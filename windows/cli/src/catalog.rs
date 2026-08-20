//! Shipped catalogs + profiles, merged with `%USERPROFILE%\.windows-wsl-setup\`.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

const LINUX_CATALOG_JSON: &str = include_str!("../../../profiles/linux.json");
const WINDOWS_CATALOG_JSON: &str = include_str!("../../../profiles/windows.json");
const LINUX_HOME: &str = include_str!("../../../profiles/linux/home.json");
const LINUX_WORK: &str = include_str!("../../../profiles/linux/work.json");
const WINDOWS_DEFAULT: &str = include_str!("../../../profiles/windows/default.json");
const WINDOWS_HOME: &str = include_str!("../../../profiles/windows/home.json");
const WINDOWS_WORK: &str = include_str!("../../../profiles/windows/work.json");
const BUNDLE_DEFAULT: &str = include_str!("../../../profiles/bundles/default.json");
const BUNDLE_HOME: &str = include_str!("../../../profiles/bundles/home.json");
const BUNDLE_WORK: &str = include_str!("../../../profiles/bundles/work.json");

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinuxCatalogFile {
    #[serde(default)]
    pub schema_version: u32,
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    pub tools: Vec<LinuxTool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowsPackage {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub linux: Option<String>,
    #[serde(default)]
    pub prefer_linux: bool,
    /// Lower installs first. 10 = password manager, 20 = browser, 30 = desktop, …
    #[serde(default = "default_priority")]
    pub priority: u32,
}

fn default_priority() -> u32 {
    100
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowsCatalogFile {
    #[serde(default)]
    pub schema_version: u32,
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    pub packages: Vec<WindowsPackage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinuxProfileDoc {
    #[serde(default)]
    pub schema_version: u32,
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub tools: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowsProfileDoc {
    #[serde(default)]
    pub schema_version: u32,
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub packages: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WslSpec {
    #[serde(default = "default_distro")]
    pub distro: String,
    #[serde(default)]
    pub create_if_missing: bool,
}

fn default_distro() -> String {
    "Ubuntu-26.04".into()
}

impl Default for WslSpec {
    fn default() -> Self {
        Self {
            distro: default_distro(),
            create_if_missing: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleDoc {
    #[serde(default)]
    pub schema_version: u32,
    pub id: String,
    #[serde(default)]
    pub name: String,
    pub windows: String,
    pub linux: String,
    #[serde(default)]
    pub wsl: WslSpec,
}

#[derive(Debug, Clone)]
pub struct Store {
    pub linux_catalog: LinuxCatalogFile,
    pub windows_catalog: WindowsCatalogFile,
    pub linux: BTreeMap<String, LinuxProfileDoc>,
    pub windows: BTreeMap<String, WindowsProfileDoc>,
    pub bundles: BTreeMap<String, BundleDoc>,
    pub linux_source: BTreeMap<String, &'static str>,
    pub windows_source: BTreeMap<String, &'static str>,
    pub bundle_source: BTreeMap<String, &'static str>,
}

pub fn user_root() -> PathBuf {
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .unwrap_or_default();
    PathBuf::from(home).join(".windows-wsl-setup")
}

pub fn user_profiles() -> PathBuf {
    user_root().join("profiles")
}

/// File / `apply` id. `"Media PC"` → `media-pc`. Must start with a letter.
pub fn sanitize_id(raw: &str) -> Result<String, String> {
    let mut n = String::new();
    let mut dash = false;
    for c in raw.trim().chars() {
        let c = c.to_ascii_lowercase();
        if c.is_ascii_alphanumeric() || c == '_' {
            n.push(c);
            dash = false;
        } else if c == '-' || c.is_ascii_whitespace() {
            if !n.is_empty() && !dash {
                n.push('-');
                dash = true;
            }
        }
    }
    if n.ends_with('-') {
        n.pop();
    }
    if n.is_empty() || !n.starts_with(|c: char| c.is_ascii_lowercase()) {
        return Err("name must start with a letter (a-z)".into());
    }
    if n.len() > 40 {
        return Err("profile id is too long".into());
    }
    Ok(n)
}

impl Store {
    pub fn shipped() -> Result<Self, String> {
        let linux_catalog: LinuxCatalogFile =
            serde_json::from_str(LINUX_CATALOG_JSON).map_err(|e| format!("linux.json: {e}"))?;
        let windows_catalog: WindowsCatalogFile =
            serde_json::from_str(WINDOWS_CATALOG_JSON).map_err(|e| format!("windows.json: {e}"))?;
        let mut s = Self {
            linux_catalog,
            windows_catalog,
            linux: BTreeMap::new(),
            windows: BTreeMap::new(),
            bundles: BTreeMap::new(),
            linux_source: BTreeMap::new(),
            windows_source: BTreeMap::new(),
            bundle_source: BTreeMap::new(),
        };
        s.insert_linux(LINUX_HOME, "shipped")?;
        s.insert_linux(LINUX_WORK, "shipped")?;
        s.insert_windows(WINDOWS_DEFAULT, "shipped")?;
        s.insert_windows(WINDOWS_HOME, "shipped")?;
        s.insert_windows(WINDOWS_WORK, "shipped")?;
        s.insert_bundle(BUNDLE_DEFAULT, "shipped")?;
        s.insert_bundle(BUNDLE_HOME, "shipped")?;
        s.insert_bundle(BUNDLE_WORK, "shipped")?;
        Ok(s)
    }

    pub fn load() -> Result<Self, String> {
        let mut s = Self::shipped()?;
        s.merge_user_dir(&user_profiles());
        Ok(s)
    }

    fn insert_linux(&mut self, json: &str, source: &'static str) -> Result<(), String> {
        let p: LinuxProfileDoc = serde_json::from_str(json).map_err(|e| e.to_string())?;
        self.linux_source.insert(p.id.clone(), source);
        self.linux.insert(p.id.clone(), p);
        Ok(())
    }

    fn insert_windows(&mut self, json: &str, source: &'static str) -> Result<(), String> {
        let p: WindowsProfileDoc = serde_json::from_str(json).map_err(|e| e.to_string())?;
        self.windows_source.insert(p.id.clone(), source);
        self.windows.insert(p.id.clone(), p);
        Ok(())
    }

    fn insert_bundle(&mut self, json: &str, source: &'static str) -> Result<(), String> {
        let p: BundleDoc = serde_json::from_str(json).map_err(|e| e.to_string())?;
        self.bundle_source.insert(p.id.clone(), source);
        self.bundles.insert(p.id.clone(), p);
        Ok(())
    }

    fn merge_user_dir(&mut self, root: &Path) {
        self.merge_json_dir(&root.join("linux"), |s, raw| {
            if let Ok(p) = serde_json::from_str::<LinuxProfileDoc>(raw) {
                s.linux_source.insert(p.id.clone(), "user");
                s.linux.insert(p.id.clone(), p);
            }
        });
        self.merge_json_dir(&root.join("windows"), |s, raw| {
            if let Ok(p) = serde_json::from_str::<WindowsProfileDoc>(raw) {
                s.windows_source.insert(p.id.clone(), "user");
                s.windows.insert(p.id.clone(), p);
            }
        });
        self.merge_json_dir(&root.join("bundles"), |s, raw| {
            if let Ok(p) = serde_json::from_str::<BundleDoc>(raw) {
                s.bundle_source.insert(p.id.clone(), "user");
                s.bundles.insert(p.id.clone(), p);
            }
        });
    }

    fn merge_json_dir(&mut self, dir: &Path, add: impl Fn(&mut Self, &str)) {
        let Ok(rd) = fs::read_dir(dir) else {
            return;
        };
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().and_then(|x| x.to_str()) != Some("json") {
                continue;
            }
            if let Ok(raw) = fs::read_to_string(&p) {
                add(self, &raw);
            }
        }
    }

    pub fn windows_pkg(&self, id: &str) -> Option<&WindowsPackage> {
        self.windows_catalog
            .packages
            .iter()
            .find(|p| p.id.eq_ignore_ascii_case(id))
    }

    pub fn linux_tool(&self, id: &str) -> Option<&LinuxTool> {
        self.linux_catalog
            .tools
            .iter()
            .find(|t| t.id.eq_ignore_ascii_case(id))
    }

    #[allow(dead_code)]
    pub fn linux_category(&self, id: &str) -> &str {
        self.linux_tool(id)
            .map(|t| t.category.as_str())
            .filter(|c| !c.is_empty())
            .unwrap_or("other")
    }

    pub fn windows_category(&self, id: &str) -> String {
        if let Some(p) = self.windows_pkg(id) {
            if !p.category.is_empty() {
                return p.category.clone();
            }
        }
        crate::classify::windows_id(id).category.to_string()
    }

    pub fn prefer_linux(&self, id: &str) -> bool {
        if let Some(p) = self.windows_pkg(id) {
            return p.prefer_linux;
        }
        crate::classify::windows_id(id).prefer_linux
    }

    pub fn linux_equivalent(&self, id: &str) -> Option<String> {
        self.windows_pkg(id).and_then(|p| p.linux.clone())
    }
}

pub fn write_json(path: &Path, v: &impl Serialize) -> Result<(), String> {
    if let Some(d) = path.parent() {
        fs::create_dir_all(d).map_err(|e| e.to_string())?;
    }
    let raw = serde_json::to_string_pretty(v).map_err(|e| e.to_string())?;
    fs::write(path, raw + "\n").map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shipped_parses() {
        let s = Store::shipped().expect("shipped json");
        assert!(s.linux.contains_key("home"));
        assert!(s.linux.contains_key("work"));
        assert!(s.windows.contains_key("default"));
        assert!(s.bundles.contains_key("home"));
        assert!(s.linux_tool("uv").is_some());
        assert_eq!(
            s.windows_pkg("Microsoft.AzureCLI")
                .unwrap()
                .linux
                .as_deref(),
            Some("azure-cli")
        );
        let home = &s.linux["home"];
        assert!(home.tools.contains(&"grok-build".into()));
        assert!(!home.tools.contains(&"copilot-cli".into()));
        let work = &s.linux["work"];
        assert!(work.tools.contains(&"copilot-cli".into()));
        assert!(!work.tools.contains(&"grok-build".into()));
        let one = s.windows_pkg("AgileBits.1Password").unwrap().priority;
        let brave = s.windows_pkg("Brave.Brave").unwrap().priority;
        let steam = s.windows_pkg("Valve.Steam").unwrap().priority;
        assert!(one < brave && brave < steam);
        assert_eq!(s.windows_category("Microsoft.Office"), "general");
        assert_eq!(s.windows_category("Microsoft.Outlook"), "general");
        assert_eq!(s.windows_category("Microsoft.VisualStudioCode"), "editors");
        assert_eq!(s.linux_tool("uv").unwrap().category, "build");
        assert_eq!(s.linux_tool("changie").unwrap().category, "environment");
        assert_eq!(s.linux_tool("devtunnel").unwrap().category, "environment");
        assert_eq!(s.linux_tool("hugo").unwrap().category, "web");
        assert_eq!(s.linux_tool("astro").unwrap().category, "web");
        assert_eq!(s.linux_tool("astro").unwrap().kind, "npm");
        assert!(home.tools.contains(&"astro".into()));
        assert!(!work.tools.contains(&"astro".into()));
    }

    #[test]
    fn profile_id_from_display_name() {
        assert_eq!(sanitize_id("Media PC").unwrap(), "media-pc");
        assert_eq!(sanitize_id("my-dev").unwrap(), "my-dev");
        assert_eq!(sanitize_id("  Home_lab  ").unwrap(), "home_lab");
        assert_eq!(sanitize_id("media--pc").unwrap(), "media-pc");
        assert_eq!(sanitize_id("a - b").unwrap(), "a-b");
        assert!(sanitize_id("").is_err());
        assert!(sanitize_id("123").is_err());
        assert!(sanitize_id("2nd PC").is_err());
    }
}
