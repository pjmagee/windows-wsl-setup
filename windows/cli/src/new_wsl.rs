//! Create a supported WSL distro and install the default toolchain.
//!
//! Create targets: latest official image per family (Ubuntu-26.04 default).
//! System packages via apt, pacman, dnf, or zypper. CLIs via Homebrew.
//! Pengwin is manage-only (already installed). The human never clones this repo.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

pub const DISTRO: &str = "Ubuntu-26.04";

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Bootstrap {
    Apt,
    Pacman,
    Dnf,
    Zypper,
}

#[derive(Debug, Clone, Copy)]
pub struct DistroFamily {
    pub family: &'static str,
    pub label: &'static str,
    pub aliases: &'static [&'static str],
    /// Id used when `wsl -l -o` is empty (tests, WSL missing).
    pub fallback: &'static str,
    /// If set, names starting with this belong to the family (FedoraLinux-44).
    pub prefix: Option<&'static str>,
    /// Exact create id. When set, never pick an older sibling (Ubuntu-24.04).
    pub exact: Option<&'static str>,
    pub skip_contains: Option<&'static str>,
    pub bootstrap: Bootstrap,
}

/// Latest per family. Intersect with `wsl --list --online` at resolve time.
pub const FAMILIES: &[DistroFamily] = &[
    DistroFamily {
        family: "ubuntu",
        label: "Ubuntu 26.04 LTS",
        aliases: &["ubuntu", "ubuntu-26.04"],
        fallback: "Ubuntu-26.04",
        prefix: Some("Ubuntu-"),
        exact: Some("Ubuntu-26.04"),
        skip_contains: None,
        bootstrap: Bootstrap::Apt,
    },
    DistroFamily {
        family: "debian",
        label: "Debian",
        aliases: &["debian"],
        fallback: "Debian",
        prefix: None,
        exact: Some("Debian"),
        skip_contains: None,
        bootstrap: Bootstrap::Apt,
    },
    DistroFamily {
        family: "arch",
        label: "Arch Linux",
        aliases: &["arch", "archlinux"],
        fallback: "archlinux",
        prefix: None,
        exact: Some("archlinux"),
        skip_contains: None,
        bootstrap: Bootstrap::Pacman,
    },
    DistroFamily {
        family: "kali",
        label: "Kali Linux",
        aliases: &["kali", "kali-linux"],
        fallback: "kali-linux",
        prefix: None,
        exact: Some("kali-linux"),
        skip_contains: None,
        bootstrap: Bootstrap::Apt,
    },
    DistroFamily {
        family: "fedora",
        label: "Fedora Linux",
        aliases: &["fedora"],
        fallback: "FedoraLinux-44",
        prefix: Some("FedoraLinux-"),
        exact: None,
        skip_contains: None,
        bootstrap: Bootstrap::Dnf,
    },
    DistroFamily {
        family: "alma",
        label: "AlmaLinux OS",
        aliases: &["alma", "almalinux"],
        fallback: "AlmaLinux-10",
        prefix: Some("AlmaLinux-"),
        exact: None,
        skip_contains: Some("kitten"),
        bootstrap: Bootstrap::Dnf,
    },
    DistroFamily {
        family: "opensuse",
        label: "openSUSE Tumbleweed",
        aliases: &["opensuse", "tumbleweed", "opensuse-tumbleweed"],
        fallback: "openSUSE-Tumbleweed",
        prefix: Some("openSUSE-"),
        exact: Some("openSUSE-Tumbleweed"),
        skip_contains: None,
        bootstrap: Bootstrap::Zypper,
    },
    DistroFamily {
        family: "oracle",
        label: "Oracle Linux",
        aliases: &["oracle", "oraclelinux"],
        fallback: "OracleLinux_9_5",
        prefix: Some("OracleLinux_"),
        exact: Some("OracleLinux_9_5"),
        skip_contains: None,
        bootstrap: Bootstrap::Dnf,
    },
];

#[derive(Debug, Clone, serde::Serialize)]
pub struct DistroChoice {
    pub id: String,
    pub label: String,
    pub family: &'static str,
    pub bootstrap: Bootstrap,
    pub online: bool,
    pub installed: bool,
}

fn family_for(name: &str) -> Option<&'static DistroFamily> {
    let n = name.trim();
    for f in FAMILIES {
        if f.fallback.eq_ignore_ascii_case(n)
            || f.aliases.iter().any(|a| a.eq_ignore_ascii_case(n))
            || f.exact
                .is_some_and(|id| id.eq_ignore_ascii_case(n))
        {
            return Some(f);
        }
        if let Some(p) = f.prefix {
            if n.len() > p.len() && n[..p.len()].eq_ignore_ascii_case(p) {
                if f.skip_contains
                    .is_some_and(|s| n.to_ascii_lowercase().contains(s))
                {
                    continue;
                }
                return Some(f);
            }
        }
    }
    None
}

fn version_key(name: &str, prefix: &str) -> Vec<u32> {
    let rest = if name.len() >= prefix.len() && name[..prefix.len()].eq_ignore_ascii_case(prefix) {
        &name[prefix.len()..]
    } else {
        name
    };
    rest.split(|c: char| !c.is_ascii_digit())
        .filter(|p| !p.is_empty())
        .filter_map(|p| p.parse().ok())
        .collect()
}

fn matches_family_online(f: &DistroFamily, name: &str) -> bool {
    if f.skip_contains
        .is_some_and(|s| name.to_ascii_lowercase().contains(s))
    {
        return false;
    }
    if let Some(id) = f.exact {
        return name.eq_ignore_ascii_case(id);
    }
    if let Some(p) = f.prefix {
        return name.len() > p.len() && name[..p.len()].eq_ignore_ascii_case(p);
    }
    name.eq_ignore_ascii_case(f.fallback)
}

/// Pick the create id for a family from `wsl --list --online`.
/// Empty `online` → fallback (WSL missing / tests).
pub fn resolve_family(f: &DistroFamily, online: &[String]) -> Result<String, String> {
    if let Some(id) = f.exact {
        if online.is_empty() || online.iter().any(|n| n.eq_ignore_ascii_case(id)) {
            return Ok(id.to_string());
        }
        return Err(format!(
            "{id} is not in `wsl --list --online`. Update WSL (elevated: wsl --update) and retry."
        ));
    }
    let mut found: Vec<&String> = online
        .iter()
        .filter(|n| matches_family_online(f, n))
        .collect();
    if found.is_empty() {
        if online.is_empty() {
            return Ok(f.fallback.to_string());
        }
        return Err(format!(
            "{} is not in `wsl --list --online`. Update WSL and retry.",
            f.fallback
        ));
    }
    let prefix = f.prefix.unwrap_or("");
    found.sort_by(|a, b| version_key(a, prefix).cmp(&version_key(b, prefix)));
    Ok((*found.last().unwrap()).clone())
}

fn create_help() -> String {
    "Ubuntu-26.04, Debian, archlinux, kali-linux, fedora, alma, opensuse, oracle".into()
}

/// Canonical WSL create name, or an error. Empty string → Ubuntu-26.04.
/// Pengwin / old numbered releases do **not** silently become Ubuntu.
pub fn parse_distro(name: &str) -> Result<String, String> {
    parse_distro_against(name, &online_names())
}

pub fn parse_distro_against(name: &str, online: &[String]) -> Result<String, String> {
    let n = name.trim();
    if n.is_empty() {
        return Ok(DISTRO.to_string());
    }
    let Some(f) = family_for(n) else {
        if n.to_ascii_lowercase().contains("pengwin") {
            return Err(
                "Pengwin is not a create target (Store app, manage-only if already installed)"
                    .into(),
            );
        }
        return Err(format!(
            "unsupported distro {name:?}. Pick one of: {}",
            create_help()
        ));
    };
    let resolved = resolve_family(f, online)?;
    let alias = f.aliases.iter().any(|a| a.eq_ignore_ascii_case(n));
    if !alias && !n.eq_ignore_ascii_case(&resolved) && !n.eq_ignore_ascii_case(f.family) {
        return Err(format!(
            "{n} is not the current {} create target ({resolved}).",
            f.label
        ));
    }
    Ok(resolved)
}

/// Names from `wsl --list --online` (NAME column). Empty if WSL is missing or the table could not be parsed.
pub fn parse_online_names(text: &str) -> Vec<String> {
    let text = text.replace('\0', "");
    let mut names = Vec::new();
    let mut started = false;
    for line in text.lines() {
        let line = line.trim();
        if !started {
            let u = line.to_ascii_uppercase();
            if u.starts_with("NAME") && u.contains("FRIENDLY") {
                started = true;
            }
            continue;
        }
        if line.is_empty() {
            continue;
        }
        if let Some(name) = line.split_whitespace().next() {
            if name.eq_ignore_ascii_case("NAME") {
                continue;
            }
            if !names.iter().any(|n: &String| n.eq_ignore_ascii_case(name)) {
                names.push(name.to_string());
            }
        }
    }
    names
}

pub fn online_names() -> Vec<String> {
    let text = Command::new("wsl.exe")
        .args(["--list", "--online"])
        .output()
        .map(|o| decode(&o.stdout) + &decode(&o.stderr))
        .unwrap_or_default();
    parse_online_names(&text)
}

pub fn distro_choices() -> Vec<DistroChoice> {
    distro_choices_against(&online_names(), &distro_names())
}

pub fn distro_choices_against(online: &[String], listed: &[String]) -> Vec<DistroChoice> {
    let parsed = !online.is_empty();
    FAMILIES
        .iter()
        .map(|f| {
            let resolved = resolve_family(f, online).unwrap_or_else(|_| f.fallback.to_string());
            let online_ok = !parsed || online.iter().any(|n| n.eq_ignore_ascii_case(&resolved));
            DistroChoice {
                id: resolved.clone(),
                label: f.label.to_string(),
                family: f.family,
                bootstrap: f.bootstrap,
                online: online_ok,
                installed: listed.iter().any(|n| n.eq_ignore_ascii_case(&resolved)),
            }
        })
        .collect()
}
const LINUX_REPO: &str = "$HOME/code/wwm";
const GIT_REMOTE: &str = "https://github.com/pjmagee/wwm.git";

const ENSURE_USER: &str = include_str!("../../ensure-user.sh");

fn decode(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).replace('\0', "")
}

fn wsl(args: &[&str]) -> Result<String, String> {
    let out = Command::new("wsl.exe")
        .args(args)
        .output()
        .map_err(|e| format!("wsl.exe: {e}"))?;
    let stdout = decode(&out.stdout);
    let stderr = decode(&out.stderr);
    if out.status.success() {
        Ok(stdout)
    } else {
        Err(format!("{stdout}{stderr}").trim().to_string())
    }
}

pub fn distro_names() -> Vec<String> {
    wsl(&["-l", "-q"])
        .unwrap_or_default()
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn is_helper_distro(name: &str) -> bool {
    name.trim().to_ascii_lowercase().starts_with("docker-desktop")
}

/// WSL default (`*` in `wsl -l -v`).
pub fn default_distro_name() -> Option<String> {
    let text = Command::new("wsl.exe")
        .args(["-l", "-v"])
        .output()
        .ok()
        .map(|o| decode(&o.stdout) + &decode(&o.stderr))?;
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix('*') {
            return rest.split_whitespace().next().map(|s| s.to_string());
        }
    }
    None
}

/// Set this distro as default only when there isn't already a real default.
pub fn maybe_set_default_distro(distro: &str) -> Result<(bool, String), String> {
    let distro = parse_distro(distro)?;
    let current = default_distro_name();
    let others: Vec<String> = distro_names()
        .into_iter()
        .filter(|n| !is_helper_distro(n) && !n.eq_ignore_ascii_case(&distro))
        .collect();
    match current.as_deref() {
        None => set_default_distro(&distro).map(|s| (true, s)),
        Some(c) if is_helper_distro(c) => set_default_distro(&distro).map(|s| (true, s)),
        Some(_) if others.is_empty() => set_default_distro(&distro).map(|s| (true, s)),
        Some(c) if c.eq_ignore_ascii_case(&distro) => {
            Ok((false, format!("default distro already {distro}")))
        }
        Some(c) => Ok((
            false,
            format!("left default as {c} (added {distro} beside it)"),
        )),
    }
}

fn windows_checkout() -> Option<PathBuf> {
    let mut starts = Vec::new();
    // Cargo build dir: .../windows/cli → repo root. Exists on this PC after
    // copying wwm.exe to ~/.wwm; missing on other machines (GitHub clone).
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if let Some(root) = manifest.parent().and_then(|p| p.parent()) {
        starts.push(root.to_path_buf());
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(p) = exe.parent() {
            starts.push(p.to_path_buf());
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        starts.push(cwd);
    }
    for mut dir in starts {
        for _ in 0..12 {
            if dir.join("install.sh").is_file() {
                return Some(dir);
            }
            if !dir.pop() {
                break;
            }
        }
    }
    None
}

#[allow(dead_code)]
pub fn has_distro() -> bool {
    has_named(DISTRO)
}

pub fn has_named(distro: &str) -> bool {
    let Ok(d) = parse_distro(distro) else {
        return false;
    };
    distro_names().iter().any(|n| n.eq_ignore_ascii_case(&d))
}

fn refuse_system_drive(path: &str) -> Result<(), String> {
    let p = path.trim();
    if p.chars()
        .next()
        .map(|c| c.eq_ignore_ascii_case(&'c'))
        .unwrap_or(false)
        && p.chars().nth(1) == Some(':')
    {
        return Err("refusing to put a WSL disk on C:".into());
    }
    Ok(())
}

fn installed_canonical(name: &str) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("distro name required".into());
    }
    if is_helper_distro(name) {
        return Err("will not touch docker-desktop".into());
    }
    distro_names()
        .into_iter()
        .find(|n| n.eq_ignore_ascii_case(name))
        .ok_or_else(|| format!("{name} is not installed"))
}

fn looks_like_reboot(msg: &str) -> bool {
    let m = msg.to_ascii_lowercase();
    m.contains("restart") || m.contains("reboot")
}

/// Best-effort: enable WSLg if the user has no `.wslconfig` yet.
pub fn ensure_wslconfig() -> String {
    let Some(profile) = std::env::var_os("USERPROFILE") else {
        return "no USERPROFILE — skipped .wslconfig".into();
    };
    let path = PathBuf::from(profile).join(".wslconfig");
    if path.exists() {
        return format!("{} already exists", path.display());
    }
    match std::fs::write(&path, "[wsl2]\nguiApplications=true\n") {
        Ok(()) => format!("wrote {}", path.display()),
        Err(e) => format!("could not write .wslconfig: {e}"),
    }
}

pub fn ensure_wsl() -> Result<String, String> {
    let ver = match Command::new("wsl.exe").arg("--version").output() {
        Ok(o) => decode(&o.stdout) + &decode(&o.stderr),
        Err(e) => {
            return Err(format!(
                "wsl.exe is missing ({e}). Enable Virtualization in firmware, then run this app again (Windows may ask for a reboot)."
            ));
        }
    };
    let _ = Command::new("wsl.exe")
        .args(["--set-default-version", "2"])
        .output();
    let _ = Command::new("wsl.exe").arg("--update").output();
    Ok(ver.lines().take(4).collect::<Vec<_>>().join(" | "))
}

fn online_has(distro: &str) -> bool {
    let names = online_names();
    if names.is_empty() {
        return true;
    }
    names.iter().any(|n| n.eq_ignore_ascii_case(distro))
}

pub fn install_distro(distro: &str) -> Result<String, String> {
    install_distro_at(distro, None)
}

pub fn install_distro_at(distro: &str, location: Option<&str>) -> Result<String, String> {
    let distro = parse_distro(distro)?;
    if has_named(&distro) {
        return Ok(format!("{distro} already installed"));
    }
    if !online_has(&distro) {
        return Err(format!(
            "{distro} is not in `wsl --list --online`. Update WSL (elevated: wsl --update) and retry."
        ));
    }
    if let Some(loc) = location {
        refuse_system_drive(loc)?;
    }
    let loc = location.map(str::trim).filter(|s| !s.is_empty());
    let mut last = String::new();
    let mut argsets: Vec<Vec<String>> = Vec::new();
    for prefix in [
        vec!["--install".into(), "-d".into(), distro.clone(), "--no-launch".into()],
        vec!["--install".into(), distro.clone(), "--no-launch".into()],
    ] {
        let mut a = prefix;
        if let Some(p) = loc {
            a.push("--location".into());
            a.push(p.to_string());
        }
        argsets.push(a);
    }
    for args in argsets {
        let out = Command::new("wsl.exe")
            .args(&args)
            .output()
            .map_err(|e| e.to_string())?;
        last = format!("{}{}", decode(&out.stdout), decode(&out.stderr));
        if has_named(&distro) {
            let extra = if looks_like_reboot(&last) {
                " Windows may still want a reboot before the first launch."
            } else {
                ""
            };
            return Ok(format!("{distro} installed.{extra}").trim().to_string());
        }
        if looks_like_reboot(&last) {
            return Err(format!(
                "Windows enabled WSL and wants a reboot. Restart, then run New WSL again.\n{last}"
            ));
        }
    }
    Err(format!(
        "could not install {distro}. Run this app elevated if Windows asked for admin.\n{last}"
    ))
}

pub fn wait_for_root(distro: &str) -> Result<String, String> {
    let distro = parse_distro(distro)?;
    let mut last = String::new();
    for _ in 0..45 {
        match wsl(&["-d", &distro, "-u", "root", "--", "echo", "ok"]) {
            Ok(s) if s.to_ascii_lowercase().contains("ok") => {
                return Ok(format!("{distro} is up (root)"));
            }
            Ok(s) => last = s,
            Err(e) => last = e,
        }
        thread::sleep(Duration::from_secs(2));
    }
    Err(format!(
        "{distro} did not start after install. Reboot if Windows asked, then retry.\n{last}"
    ))
}

#[allow(dead_code)]
pub fn linux_user() -> Option<String> {
    linux_user_on(DISTRO)
}

pub fn linux_user_on(distro: &str) -> Option<String> {
    let distro = parse_distro(distro).ok()?;
    wsl(&[
        "-d",
        &distro,
        "-u",
        "root",
        "--",
        "bash",
        "-lc",
        "getent passwd 1000 | cut -d: -f1",
    ])
    .ok()
    .map(|s| s.trim().to_string())
    .filter(|s| !s.is_empty())
}

fn linux_name_from_windows() -> String {
    let raw = std::env::var("USERNAME").unwrap_or_else(|_| "ubuntu".into());
    let mut n: String = raw
        .to_ascii_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .collect();
    if n.is_empty() {
        n = "ubuntu".into();
    }
    if !n.starts_with(|c: char| c.is_ascii_lowercase() || c == '_') {
        n.insert(0, 'u');
    }
    n.truncate(32);
    n
}

/// Create uid 1000 from the Windows username when the distro has no user yet.
pub fn create_user_if_needed(distro: &str) -> Result<String, String> {
    let distro = parse_distro(distro)?;
    if let Some(u) = linux_user_on(&distro) {
        return Ok(format!("linux user already exists: {u}"));
    }
    let name = linux_name_from_windows();
    let script = format!(
        r#"set -euo pipefail
if getent passwd 1000 >/dev/null; then
  echo "uid 1000 exists: $(getent passwd 1000 | cut -d: -f1)"
  exit 0
fi
if id -u {name} >/dev/null 2>&1; then
  echo "user {name} exists"
  exit 0
fi
useradd -m -s /bin/bash -u 1000 {name}
usermod -aG sudo,adm,wheel {name} 2>/dev/null || usermod -aG wheel {name} 2>/dev/null || usermod -aG sudo,adm {name} 2>/dev/null || true
passwd -d {name} >/dev/null || true
echo "created {name} (uid 1000, empty password)"
"#
    );
    wsl(&[
        "-d",
        &distro,
        "-u",
        "root",
        "--",
        "bash",
        "-lc",
        &script,
    ])
    .map(|s| s.trim().to_string())
}

pub fn ensure_passwordless_sudo(distro: &str) -> Result<String, String> {
    let distro = parse_distro(distro)?;
    if linux_user_on(&distro).is_none() {
        return Err(
            "no Linux user yet — New WSL should have created one. Retry, or open the distro once."
                .into(),
        );
    }
    let mut child = Command::new("wsl.exe")
        .args(["-d", &distro, "-u", "root", "--", "bash", "-s"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(ENSURE_USER.replace("\r\n", "\n").as_bytes())
            .map_err(|e| e.to_string())?;
    }
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    let text = format!("{}{}", decode(&out.stdout), decode(&out.stderr));
    if !out.status.success() {
        return Err(text.trim().to_string());
    }
    let _ = Command::new("wsl.exe")
        .args(["--terminate", &distro])
        .output();
    Ok(text.trim().to_string())
}

pub fn is_blank_profile(id: &str) -> bool {
    id.eq_ignore_ascii_case("blank")
}

/// Save ~/.wwm/profile=blank. No Homebrew, no clone.
pub fn mark_blank_profile(distro: &str) -> Result<String, String> {
    let distro = parse_distro(distro)?;
    let script = r#"
set -euo pipefail
mkdir -p "$(getent passwd 1000 | cut -d: -f6)/.wwm"
printf 'blank\n' >"$(getent passwd 1000 | cut -d: -f6)/.wwm/profile"
echo "blank: distro + passwordless sudo only"
"#;
    wsl(&["-d", &distro, "--", "bash", "-lc", script]).map(|s| s.trim().to_string())
}

pub fn set_default_distro(distro: &str) -> Result<String, String> {
    let distro = parse_distro(distro)?;
    wsl(&["--set-default", &distro]).map(|_| format!("default distro = {distro}"))
}

/// Deletes the distro disk. Caller must have confirmed (`--yes`).
pub fn unregister_distro(name: &str) -> Result<String, String> {
    let canonical = installed_canonical(name)?;
    wsl(&["--unregister", &canonical]).map(|_| format!("unregistered {canonical}"))
}

/// `wsl --manage <name> --move <dir>`. Any installed distro, including Pengwin.
pub fn move_distro(name: &str, dir: &str) -> Result<String, String> {
    let canonical = installed_canonical(name)?;
    refuse_system_drive(dir)?;
    let dir = dir.trim();
    let _ = Command::new("wsl.exe")
        .args(["--terminate", &canonical])
        .output();
    wsl(&["--manage", &canonical, "--move", dir])
        .map(|_| format!("moved {canonical} to {dir}"))
}

fn valid_clone_name(name: &str) -> Result<(), String> {
    let n = name.trim();
    if n.is_empty() {
        return Err("clone name required".into());
    }
    if is_helper_distro(n) {
        return Err("will not name a distro docker-desktop".into());
    }
    if !n
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic())
        || !n
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(format!(
            "invalid distro name {n:?} (start with a letter; letters, digits, - _)"
        ));
    }
    Ok(())
}

/// Export src as VHD and import as `new_name`. Any installed distro.
pub fn clone_distro(src: &str, new_name: &str, location: Option<&str>) -> Result<String, String> {
    let src = installed_canonical(src)?;
    valid_clone_name(new_name)?;
    let new_name = new_name.trim();
    if distro_names()
        .iter()
        .any(|n| n.eq_ignore_ascii_case(new_name))
    {
        return Err(format!("{new_name} already exists"));
    }
    let dest = match location {
        Some(p) => {
            refuse_system_drive(p)?;
            PathBuf::from(p.trim())
        }
        None => {
            let base = lxss_base_path(&src).ok_or_else(|| {
                format!("could not find the VHDX for {src}; pass --location on another drive")
            })?;
            let parent = Path::new(&base)
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| PathBuf::from(&base));
            let dest = parent.join(new_name);
            refuse_system_drive(&dest.to_string_lossy())?;
            dest
        }
    };
    std::fs::create_dir_all(&dest).map_err(|e| format!("create {}: {e}", dest.display()))?;
    let export = dest.join("clone-export.vhdx");
    let export_s = export.to_string_lossy().into_owned();
    let dest_s = dest.to_string_lossy().into_owned();
    let _ = Command::new("wsl.exe")
        .args(["--terminate", &src])
        .output();
    wsl(&["--export", &src, &export_s, "--format", "vhd"])
        .map_err(|e| format!("export {src}: {e}"))?;
    let import = wsl(&["--import", new_name, &dest_s, &export_s, "--vhd"]);
    let _ = std::fs::remove_file(&export);
    import.map_err(|e| format!("import {new_name}: {e}"))?;
    if let Some(uid) = lxss_default_uid(&src) {
        let _ = lxss_set_default_uid(new_name, uid);
    }
    Ok(format!("cloned {src} → {new_name} at {}", dest.display()))
}

#[cfg(windows)]
fn lxss_base_path(name: &str) -> Option<String> {
    let hk = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    let lxss = hk
        .open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Lxss")
        .ok()?;
    for key in lxss.enum_keys().filter_map(|k| k.ok()) {
        if key.eq_ignore_ascii_case("DefaultDistribution") {
            continue;
        }
        let Ok(sub) = lxss.open_subkey(&key) else {
            continue;
        };
        let distro: String = sub.get_value("DistributionName").unwrap_or_default();
        if distro.eq_ignore_ascii_case(name) {
            let base: String = sub.get_value("BasePath").unwrap_or_default();
            if base.is_empty() {
                return None;
            }
            return Some(base.trim_start_matches(r"\\?\").to_string());
        }
    }
    None
}

#[cfg(not(windows))]
fn lxss_base_path(_name: &str) -> Option<String> {
    None
}

#[cfg(windows)]
fn lxss_default_uid(name: &str) -> Option<u32> {
    let hk = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    let lxss = hk
        .open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Lxss")
        .ok()?;
    for key in lxss.enum_keys().filter_map(|k| k.ok()) {
        let Ok(sub) = lxss.open_subkey(&key) else {
            continue;
        };
        let distro: String = sub.get_value("DistributionName").unwrap_or_default();
        if distro.eq_ignore_ascii_case(name) {
            return sub.get_value("DefaultUid").ok();
        }
    }
    None
}

#[cfg(not(windows))]
fn lxss_default_uid(_name: &str) -> Option<u32> {
    None
}

#[cfg(windows)]
fn lxss_set_default_uid(name: &str, uid: u32) -> Result<(), String> {
    let hk = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    let lxss = hk
        .open_subkey_with_flags(
            "Software\\Microsoft\\Windows\\CurrentVersion\\Lxss",
            winreg::enums::KEY_READ | winreg::enums::KEY_WRITE,
        )
        .map_err(|e| e.to_string())?;
    for key in lxss.enum_keys().filter_map(|k| k.ok()) {
        if key.eq_ignore_ascii_case("DefaultDistribution") {
            continue;
        }
        let sub = lxss
            .open_subkey_with_flags(&key, winreg::enums::KEY_READ | winreg::enums::KEY_WRITE)
            .map_err(|e| e.to_string())?;
        let distro: String = sub.get_value("DistributionName").unwrap_or_default();
        if distro.eq_ignore_ascii_case(name) {
            sub.set_value("DefaultUid", &uid).map_err(|e| e.to_string())?;
            return Ok(());
        }
    }
    Err(format!("no Lxss key for {name}"))
}

#[cfg(not(windows))]
fn lxss_set_default_uid(_name: &str, _uid: u32) -> Result<(), String> {
    Ok(())
}

/// Clone this project's Linux installer inside the distro and run it.
pub fn install_toolchain(
    distro: &str,
    profile: &str,
    profile_json: Option<&str>,
) -> Result<String, String> {
    let distro = parse_distro(distro)?;
    let profile = crate::catalog::sanitize_id(profile)?;
    let mut copy = String::new();
    if let Some(json) = profile_json {
        let tmp = std::env::temp_dir().join(format!("wwm-linux-{profile}.json"));
        std::fs::write(&tmp, json).map_err(|e| format!("write linux profile: {e}"))?;
        let win = tmp.to_string_lossy().replace('\'', "");
        copy = format!(
            r#"
mkdir -p "$HOME/.wwm/profiles"
src="$(wslpath -a '{win}' 2>/dev/null || true)"
if [ -n "$src" ] && [ -f "$src" ]; then
  cp "$src" "$HOME/.wwm/profiles/{profile}.json"
fi
"#
        );
    }
    let checkout = windows_checkout();
    let win_src = checkout
        .as_ref()
        .map(|p| p.display().to_string().replace('\\', "/").replace('\'', ""))
        .unwrap_or_default();
    let linux_src = if win_src.is_empty() {
        String::new()
    } else {
        wsl(&["-d", &distro, "--", "wslpath", "-a", &win_src])
            .ok()
            .map(|s| s.trim().replace('\'', ""))
            .filter(|s| !s.is_empty() && !s.contains('\n'))
            .unwrap_or_default()
    };
    let fetch = if linux_src.is_empty() {
        format!(
            r#"
if [ ! -d {repo}/.git ]; then
  echo "wwm installer git clone {remote}"
  git clone {remote} {repo}
else
  echo "wwm installer git pull"
  git -C {repo} pull --ff-only || true
fi
"#,
            repo = LINUX_REPO,
            remote = GIT_REMOTE,
        )
    } else {
        format!(
            r#"
echo "wwm installer from {src}"
mkdir -p {repo}
tar -C '{src}' \
  --exclude='windows/cli/target' \
  --exclude='site/node_modules' \
  --exclude='site/dist' \
  --exclude='site/.astro' \
  --exclude='.git' \
  -cf - . | tar -C {repo} -xf -
"#,
            src = linux_src,
            repo = LINUX_REPO,
        )
    };
    let script = format!(
        r#"
set -euo pipefail
sudo -n true
if command -v apt-get >/dev/null; then
  sudo apt-get update -y
  sudo DEBIAN_FRONTEND=noninteractive apt-get install -y git curl
elif command -v pacman >/dev/null; then
  sudo pacman -Sy --noconfirm --needed git curl
elif command -v dnf >/dev/null; then
  sudo dnf install -y git curl
elif command -v zypper >/dev/null; then
  sudo zypper --non-interactive install git curl
fi
{copy}
mkdir -p "$HOME/code"
{fetch}
git -C {repo} remote set-url origin {remote} 2>/dev/null || true
if [ -f "$HOME/.wwm/profiles/{profile}.json" ]; then
  mkdir -p {repo}/profiles/linux
  cp "$HOME/.wwm/profiles/{profile}.json" {repo}/profiles/linux/{profile}.json
fi
chmod +x {repo}/install.sh {repo}/scripts/wsl-open {repo}/scripts/pbcopy {repo}/scripts/pbpaste {repo}/windows/ensure-user.sh
cd {repo}
./install.sh {profile}
"#,
        repo = LINUX_REPO,
        remote = GIT_REMOTE,
        profile = profile,
        copy = copy,
        fetch = fetch,
    );
    let out = Command::new("wsl.exe")
        .args(["-d", &distro, "--", "bash", "-lc", &script])
        .output()
        .map_err(|e| e.to_string())?;
    let text = format!("{}{}", decode(&out.stdout), decode(&out.stderr));
    if out.status.success() {
        Ok(text)
    } else {
        Err(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_profile_id() {
        assert!(is_blank_profile("blank"));
        assert!(is_blank_profile("Blank"));
        assert!(!is_blank_profile("home"));
    }

    fn sample_online() -> Vec<String> {
        parse_online_names(
            "\
The following is a list of valid distributions that can be installed.\n\
Install using 'wsl.exe --install <Distro>'.\n\
\n\
NAME                            FRIENDLY NAME\n\
Ubuntu                          Ubuntu\n\
Debian                          Debian GNU/Linux\n\
kali-linux                      Kali Linux Rolling\n\
Ubuntu-24.04                    Ubuntu 24.04 LTS\n\
Ubuntu-26.04                    Ubuntu 26.04 LTS\n\
archlinux                       Arch Linux\n\
FedoraLinux-43                  Fedora Linux 43\n\
FedoraLinux-44                  Fedora Linux 44\n\
AlmaLinux-8                     AlmaLinux OS 8\n\
AlmaLinux-10                    AlmaLinux OS 10\n\
AlmaLinux-Kitten-10             AlmaLinux OS Kitten 10\n\
openSUSE-Tumbleweed             openSUSE Tumbleweed\n\
openSUSE-Leap-16.0              openSUSE Leap 16.0\n\
OracleLinux_9_5                 Oracle Linux 9.5\n\
OracleLinux_7_9                 Oracle Linux 7.9\n",
        )
    }

    #[test]
    fn aliases_resolve() {
        let online = sample_online();
        assert_eq!(parse_distro_against("ubuntu", &online).unwrap(), "Ubuntu-26.04");
        assert_eq!(parse_distro_against("Debian", &online).unwrap(), "Debian");
        assert_eq!(parse_distro_against("arch", &online).unwrap(), "archlinux");
        assert_eq!(parse_distro_against("", &online).unwrap(), "Ubuntu-26.04");
        assert_eq!(parse_distro_against("kali", &online).unwrap(), "kali-linux");
        assert_eq!(parse_distro_against("fedora", &online).unwrap(), "FedoraLinux-44");
        assert_eq!(parse_distro_against("alma", &online).unwrap(), "AlmaLinux-10");
        assert_eq!(
            parse_distro_against("opensuse", &online).unwrap(),
            "openSUSE-Tumbleweed"
        );
        assert_eq!(
            parse_distro_against("oracle", &online).unwrap(),
            "OracleLinux_9_5"
        );
    }

    #[test]
    fn checkout_has_install_sh() {
        let root = windows_checkout().expect("repo root with install.sh");
        assert!(root.join("install.sh").is_file());
        assert!(root.join("profiles").is_dir());
    }

    #[test]
    fn helper_distros() {
        assert!(is_helper_distro("docker-desktop"));
        assert!(is_helper_distro("docker-desktop-data"));
        assert!(!is_helper_distro("Debian"));
    }

    #[test]
    fn rejects_old_and_unofficial() {
        let online = sample_online();
        let e = parse_distro_against("FedoraLinux-43", &online).unwrap_err();
        assert!(e.contains("FedoraLinux-44"), "{e}");
        let e = parse_distro_against("Ubuntu-24.04", &online).unwrap_err();
        assert!(e.contains("Ubuntu-26.04"), "{e}");
        let e = parse_distro_against("Pengwin", &online).unwrap_err();
        assert!(e.to_ascii_lowercase().contains("pengwin"), "{e}");
        let e = parse_distro_against("FedoraRemix", &online).unwrap_err();
        assert!(e.contains("unsupported"), "{e}");
        let e = parse_distro_against("AlmaLinux-Kitten-10", &online).unwrap_err();
        assert!(e.contains("unsupported") || e.contains("AlmaLinux-10"), "{e}");
    }

    #[test]
    fn latest_per_family_from_online() {
        let online = sample_online();
        let listed = vec!["Debian".into(), "Pengwin".into()];
        let choices = distro_choices_against(&online, &listed);
        let id = |fam: &str| {
            choices
                .iter()
                .find(|c| c.family == fam)
                .map(|c| c.id.as_str())
                .unwrap()
        };
        assert_eq!(id("ubuntu"), "Ubuntu-26.04");
        assert_eq!(id("fedora"), "FedoraLinux-44");
        assert_eq!(id("alma"), "AlmaLinux-10");
        assert_eq!(id("opensuse"), "openSUSE-Tumbleweed");
        assert_eq!(id("oracle"), "OracleLinux_9_5");
        assert_eq!(id("kali"), "kali-linux");
        let debian = choices.iter().find(|c| c.family == "debian").unwrap();
        assert!(debian.installed);
        assert!(debian.online);
        assert!(!choices.iter().any(|c| c.id.contains("Kitten")));
        assert!(!choices.iter().any(|c| c.id.contains("Pengwin")));
    }

    #[test]
    fn parses_wsl_list_online() {
        let names = sample_online();
        assert!(names.iter().any(|n| n == "Ubuntu-26.04"));
        assert!(names.iter().any(|n| n == "Debian"));
        assert!(names.iter().any(|n| n == "archlinux"));
        assert!(names.iter().any(|n| n == "FedoraLinux-44"));
        assert!(!names.iter().any(|n| n == "NAME"));
    }

    #[test]
    fn clone_name_rules() {
        assert!(valid_clone_name("Debian-copy").is_ok());
        assert!(valid_clone_name("docker-desktop").is_err());
        assert!(valid_clone_name("1bad").is_err());
        assert!(valid_clone_name("has space").is_err());
    }

    #[test]
    fn refuse_c_drive() {
        assert!(refuse_system_drive(r"C:\WSL\Debian").is_err());
        assert!(refuse_system_drive(r"c:\foo").is_err());
        assert!(refuse_system_drive(r"D:\WSL\Debian").is_ok());
    }
}
