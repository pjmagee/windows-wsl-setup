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
pub const SUPPORTED: &[&str] = &["Ubuntu-26.04", "Debian", "archlinux"];

pub fn normalize_distro(name: &str) -> &str {
    let n = name.trim();
    if n.is_empty() {
        return DISTRO;
    }
    if n.eq_ignore_ascii_case("ubuntu") || n.eq_ignore_ascii_case("ubuntu-26.04") {
        return "Ubuntu-26.04";
    }
    if n.eq_ignore_ascii_case("debian") {
        return "Debian";
    }
    if n.eq_ignore_ascii_case("arch") || n.eq_ignore_ascii_case("archlinux") {
        return "archlinux";
    }
    DISTRO
}
const LINUX_REPO: &str = "$HOME/code/windows-wsl-setup";
const GIT_REMOTE: &str = "https://github.com/pjmagee/windows-wsl-setup.git";

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

#[allow(dead_code)]
pub fn has_distro() -> bool {
    has_named(DISTRO)
}

pub fn has_named(distro: &str) -> bool {
    let d = normalize_distro(distro);
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
    Command::new("wsl.exe")
        .args(["--list", "--online"])
        .output()
        .map(|o| {
            let text = decode(&o.stdout) + &decode(&o.stderr);
            text.to_ascii_lowercase()
                .contains(&distro.to_ascii_lowercase())
        })
        .unwrap_or(true)
}

pub fn install_distro(distro: &str) -> Result<String, String> {
    let distro = normalize_distro(distro);
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
    let distro = normalize_distro(distro);
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
    let distro = normalize_distro(distro);
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
    let distro = normalize_distro(distro);
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
    let distro = normalize_distro(distro);
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

pub fn set_default_distro(distro: &str) -> Result<String, String> {
    let distro = normalize_distro(distro);
    wsl(&["--set-default", distro]).map(|_| format!("default distro = {distro}"))
}

/// Clone this project's Linux installer inside the distro and run it.
pub fn install_toolchain(
    distro: &str,
    profile: &str,
    profile_json: Option<&str>,
) -> Result<String, String> {
    let distro = normalize_distro(distro);
    let profile = crate::catalog::sanitize_id(profile)?;
    let mut copy = String::new();
    if let Some(json) = profile_json {
        let tmp = std::env::temp_dir().join(format!("wsl-setup-linux-{profile}.json"));
        std::fs::write(&tmp, json).map_err(|e| format!("write linux profile: {e}"))?;
        let win = tmp.to_string_lossy().replace('\'', "");
        copy = format!(
            r#"
mkdir -p "$HOME/.config/wsl-setup/profiles"
src="$(wslpath -a '{win}' 2>/dev/null || true)"
if [ -n "$src" ] && [ -f "$src" ]; then
  cp "$src" "$HOME/.config/wsl-setup/profiles/{profile}.json"
fi
"#
        );
    }
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
if [ ! -d {repo}/.git ]; then
  git clone {remote} {repo}
else
  git -C {repo} pull --ff-only || true
fi
if [ -f "$HOME/.config/wsl-setup/profiles/{profile}.json" ]; then
  mkdir -p {repo}/profiles/linux
  cp "$HOME/.config/wsl-setup/profiles/{profile}.json" {repo}/profiles/linux/{profile}.json
fi
chmod +x {repo}/install.sh {repo}/scripts/wsl-open {repo}/windows/ensure-user.sh
cd {repo}
./install.sh {profile}
"#,
        repo = LINUX_REPO,
        remote = GIT_REMOTE,
        profile = profile,
        copy = copy,
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
