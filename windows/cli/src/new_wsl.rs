//! Create a supported WSL distro and install the default toolchain.
//!
//! Official: Ubuntu-26.04 (default), Debian. Also: archlinux.
//! System packages via apt or pacman. CLIs via Homebrew. The human never clones.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

pub const DISTRO: &str = "Ubuntu-26.04";

#[derive(Debug, Clone, Copy)]
pub struct DistroKind {
    pub id: &'static str,
    pub label: &'static str,
    pub aliases: &'static [&'static str],
}

/// Distros this product can bootstrap (apt or pacman + Homebrew).
/// Fedora / Kali / openSUSE are restore-only, not Create targets.
pub const SUPPORTED: &[DistroKind] = &[
    DistroKind {
        id: "Ubuntu-26.04",
        label: "Ubuntu 26.04 LTS",
        aliases: &["ubuntu", "ubuntu-26.04"],
    },
    DistroKind {
        id: "Debian",
        label: "Debian",
        aliases: &["debian"],
    },
    DistroKind {
        id: "archlinux",
        label: "Arch Linux",
        aliases: &["arch", "archlinux"],
    },
];

#[derive(Debug, Clone, serde::Serialize)]
pub struct DistroChoice {
    pub id: &'static str,
    pub label: &'static str,
    pub online: bool,
    pub installed: bool,
}

/// Canonical WSL name, or an error. Empty string → Ubuntu-26.04.
/// Unknown names (Fedora, Kali, …) do **not** silently become Ubuntu.
pub fn parse_distro(name: &str) -> Result<&'static str, String> {
    let n = name.trim();
    if n.is_empty() {
        return Ok(DISTRO);
    }
    for d in SUPPORTED {
        if d.id.eq_ignore_ascii_case(n) || d.aliases.iter().any(|a| a.eq_ignore_ascii_case(n)) {
            return Ok(d.id);
        }
    }
    let ids: Vec<&str> = SUPPORTED.iter().map(|d| d.id).collect();
    Err(format!(
        "unsupported distro {name:?}. Pick one of: {}",
        ids.join(", ")
    ))
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
    let online = online_names();
    let listed = distro_names();
    let parsed = !online.is_empty();
    SUPPORTED
        .iter()
        .map(|d| DistroChoice {
            id: d.id,
            label: d.label,
            online: !parsed || online.iter().any(|n| n.eq_ignore_ascii_case(d.id)),
            installed: listed.iter().any(|n| n.eq_ignore_ascii_case(d.id)),
        })
        .collect()
}
const LINUX_REPO: &str = "$HOME/code/wwm";
const GIT_REMOTE: &str = "https://github.com/pjmagee/wwm.git";
const LEGACY_LINUX_REPO: &str = "$HOME/code/windows-wsl-manager";

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
        .filter(|n| !is_helper_distro(n) && !n.eq_ignore_ascii_case(distro))
        .collect();
    match current.as_deref() {
        None => set_default_distro(distro).map(|s| (true, s)),
        Some(c) if is_helper_distro(c) => set_default_distro(distro).map(|s| (true, s)),
        Some(_) if others.is_empty() => set_default_distro(distro).map(|s| (true, s)),
        Some(c) if c.eq_ignore_ascii_case(distro) => {
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
    distro_names().iter().any(|n| n.eq_ignore_ascii_case(d))
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
    let distro = parse_distro(distro)?;
    if has_named(distro) {
        return Ok(format!("{distro} already installed"));
    }
    if !online_has(distro) {
        return Err(format!(
            "{distro} is not in `wsl --list --online`. Update WSL (elevated: wsl --update) and retry."
        ));
    }
    let attempts: [&[&str]; 2] = [
        &["--install", "-d", distro, "--no-launch"],
        &["--install", distro, "--no-launch"],
    ];
    let mut last = String::new();
    for args in attempts {
        let out = Command::new("wsl.exe")
            .args(args)
            .output()
            .map_err(|e| e.to_string())?;
        last = format!("{}{}", decode(&out.stdout), decode(&out.stderr));
        if has_named(distro) {
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
        match wsl(&["-d", distro, "-u", "root", "--", "echo", "ok"]) {
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
        distro,
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
    if let Some(u) = linux_user_on(distro) {
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
    wsl(&["-d", distro, "-u", "root", "--", "bash", "-lc", &script]).map(|s| s.trim().to_string())
}

pub fn ensure_passwordless_sudo(distro: &str) -> Result<String, String> {
    let distro = parse_distro(distro)?;
    if linux_user_on(distro).is_none() {
        return Err(
            "no Linux user yet — New WSL should have created one. Retry, or open the distro once."
                .into(),
        );
    }
    let mut child = Command::new("wsl.exe")
        .args(["-d", distro, "-u", "root", "--", "bash", "-s"])
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
        .args(["--terminate", distro])
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
    wsl(&["-d", distro, "--", "bash", "-lc", script]).map(|s| s.trim().to_string())
}

pub fn set_default_distro(distro: &str) -> Result<String, String> {
    let distro = parse_distro(distro)?;
    wsl(&["--set-default", distro]).map(|_| format!("default distro = {distro}"))
}

/// Deletes the distro disk. Caller must have confirmed (`--yes`).
pub fn unregister_distro(name: &str) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("distro name required".into());
    }
    if is_helper_distro(name) {
        return Err("will not unregister docker-desktop".into());
    }
    let names = distro_names();
    let canonical = names
        .iter()
        .find(|n| n.eq_ignore_ascii_case(name))
        .cloned()
        .ok_or_else(|| format!("{name} is not installed"))?;
    wsl(&["--unregister", &canonical]).map(|_| format!("unregistered {canonical}"))
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
        wsl(&["-d", distro, "--", "wslpath", "-a", &win_src])
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
  if [ -d {legacy}/.git ]; then
    git clone {legacy} {repo}
  else
    git clone {remote} {repo}
  fi
else
  echo "wwm installer git pull"
  git -C {repo} pull --ff-only || true
fi
"#,
            repo = LINUX_REPO,
            legacy = LEGACY_LINUX_REPO,
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
fi
{copy}
mkdir -p "$HOME/code"
{fetch}
git -C {repo} remote set-url origin {remote} 2>/dev/null || true
if [ -f "$HOME/.wwm/profiles/{profile}.json" ]; then
  mkdir -p {repo}/profiles/linux
  cp "$HOME/.wwm/profiles/{profile}.json" {repo}/profiles/linux/{profile}.json
fi
chmod +x {repo}/install.sh {repo}/scripts/wsl-open {repo}/windows/ensure-user.sh
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
        .args(["-d", distro, "--", "bash", "-lc", &script])
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

    #[test]
    fn aliases_resolve() {
        assert_eq!(parse_distro("ubuntu").unwrap(), "Ubuntu-26.04");
        assert_eq!(parse_distro("Debian").unwrap(), "Debian");
        assert_eq!(parse_distro("arch").unwrap(), "archlinux");
        assert_eq!(parse_distro("").unwrap(), "Ubuntu-26.04");
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
    fn fedora_is_not_ubuntu() {
        let e = parse_distro("FedoraLinux-42").unwrap_err();
        assert!(e.contains("unsupported"));
        assert!(!e.to_ascii_lowercase().contains("became"));
    }

    #[test]
    fn parses_wsl_list_online() {
        let sample = "\
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
FedoraLinux-42                  Fedora Linux 42\n";
        let names = parse_online_names(sample);
        assert!(names.iter().any(|n| n == "Ubuntu-26.04"));
        assert!(names.iter().any(|n| n == "Debian"));
        assert!(names.iter().any(|n| n == "archlinux"));
        assert!(names.iter().any(|n| n == "FedoraLinux-42"));
        assert!(!names.iter().any(|n| n == "NAME"));
    }
}
