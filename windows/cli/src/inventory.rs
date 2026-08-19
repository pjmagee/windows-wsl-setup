use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::model::*;

pub fn collect() -> Result<Inventory, String> {
    let user_profile = std::env::var("USERPROFILE").unwrap_or_default();
    let local = std::env::var("LOCALAPPDATA").unwrap_or_default();
    let computer = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "PC".into());
    let user = std::env::var("USERNAME").unwrap_or_default();

    Ok(Inventory {
        computer,
        user,
        user_profile: user_profile.clone(),
        destinations: destinations()?,
        apps: winget_apps(),
        wsl: wsl_distros(),
        dev_drive: dev_drive(),
        docker: docker_data(&local),
        brave: brave(&local),
        dotfiles: dotfiles(&user_profile, &local),
        linux_tools: load_linux_tools(),
    })
}

pub fn winget_apps_pub() -> Vec<WingetApp> {
    winget_apps()
}

fn load_linux_tools() -> LinuxToolsFile {
    if let Some(root) = repo_root() {
        let p = root.join("profiles/linux.json");
        if let Ok(raw) = fs::read_to_string(&p) {
            if let Ok(v) = serde_json::from_str::<LinuxToolsFile>(&raw) {
                return v;
            }
        }
    }
    LinuxToolsFile {
        schema_version: 1,
        note: String::new(),
        tools: Vec::new(),
    }
}

fn file_gb(path: &Path) -> f64 {
    fs::metadata(path)
        .map(|m| m.len() as f64 / 1_073_741_824.0)
        .unwrap_or(0.0)
}

fn destinations() -> Result<Vec<Destination>, String> {
    let mut out = Vec::new();
    let today = chrono_today();
    let host = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "PC".into());
    for letter in 'D'..='Z' {
        if letter == 'C' {
            continue;
        }
        let root = format!("{letter}:\\");
        if !Path::new(&root).exists() {
            continue;
        }
        let (label, fs) = volume_info(&root);
        if fs.eq_ignore_ascii_case("ReFS") || label.eq_ignore_ascii_case("Dev Drive") {
            continue;
        }
        let (total, free) = disk_space(&root);
        if total == 0 {
            continue;
        }
        let gb = total / 1_073_741_824;
        let free_gb = free as f64 / 1_073_741_824.0;
        out.push(Destination {
            letter,
            label,
            file_system: fs,
            gb,
            free_gb: (free_gb * 10.0).round() / 10.0,
            guid: volume_guid(&root),
            suggested: format!("{letter}:\\Backups\\{host}-{today}"),
        });
    }
    out.sort_by(|a, b| b.free_gb.partial_cmp(&a.free_gb).unwrap());
    Ok(out)
}

fn chrono_today() -> String {
    // Local date YYYY-MM-DD without extra crates.
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    // Use PowerShell-less approximation: UTC date is fine for folder names.
    let days = secs / 86400;
    let (y, m, d) = civil_from_days(days);
    format!("{y:04}-{m:02}-{d:02}")
}

fn civil_from_days(z: i64) -> (i32, u32, u32) {
    // Howard Hinnant civil_from_days (UTC).
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m as u32, d as u32)
}

#[cfg(windows)]
fn volume_info(root: &str) -> (String, String) {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::GetVolumeInformationW;
    let wide: Vec<u16> = std::ffi::OsStr::new(root)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let mut name = [0u16; 64];
    let mut fs = [0u16; 32];
    let ok = unsafe {
        GetVolumeInformationW(
            wide.as_ptr(),
            name.as_mut_ptr(),
            name.len() as u32,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            fs.as_mut_ptr(),
            fs.len() as u32,
        )
    };
    if ok == 0 {
        return (String::new(), String::new());
    }
    (utf16_z(&name), utf16_z(&fs))
}

#[cfg(windows)]
fn disk_space(root: &str) -> (u64, u64) {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
    let wide: Vec<u16> = std::ffi::OsStr::new(root)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let mut free = 0u64;
    let mut total = 0u64;
    let ok =
        unsafe { GetDiskFreeSpaceExW(wide.as_ptr(), std::ptr::null_mut(), &mut total, &mut free) };
    if ok == 0 {
        (0, 0)
    } else {
        (total, free)
    }
}

#[cfg(windows)]
fn volume_guid(root: &str) -> Option<String> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::GetVolumeNameForVolumeMountPointW;
    let wide: Vec<u16> = std::ffi::OsStr::new(root)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let mut buf = [0u16; 80];
    let ok = unsafe { GetVolumeNameForVolumeMountPointW(wide.as_ptr(), buf.as_mut_ptr(), 80) };
    if ok == 0 {
        return None;
    }
    let s = utf16_z(&buf);
    s.split('{')
        .nth(1)
        .and_then(|r| r.split('}').next())
        .map(|g| g.to_string())
}

#[cfg(windows)]
fn utf16_z(buf: &[u16]) -> String {
    let n = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..n])
}

#[cfg(not(windows))]
fn volume_info(_: &str) -> (String, String) {
    (String::new(), String::new())
}
#[cfg(not(windows))]
fn disk_space(_: &str) -> (u64, u64) {
    (0, 0)
}
#[cfg(not(windows))]
fn volume_guid(_: &str) -> Option<String> {
    None
}

fn winget_apps() -> Vec<WingetApp> {
    let tmp = std::env::temp_dir().join(format!("wsl-setup-winget-{}.json", std::process::id()));
    let _ = Command::new("winget")
        .args([
            "export",
            "--output",
            tmp.to_str().unwrap_or("wsl-setup-winget.json"),
            "--accept-source-agreements",
            "--disable-interactivity",
        ])
        .output();
    let mut apps = Vec::new();
    if let Ok(raw) = fs::read_to_string(&tmp) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
            if let Some(sources) = v.get("Sources").and_then(|s| s.as_array()) {
                for src in sources {
                    if let Some(pkgs) = src.get("Packages").and_then(|p| p.as_array()) {
                        for p in pkgs {
                            let id = p
                                .get("PackageIdentifier")
                                .and_then(|x| x.as_str())
                                .unwrap_or("")
                                .to_string();
                            if id.is_empty() {
                                continue;
                            }
                            let version = p
                                .get("Version")
                                .and_then(|x| x.as_str())
                                .unwrap_or("")
                                .to_string();
                            let keep = default_keep_app(&id);
                            apps.push(WingetApp {
                                name: id.clone(),
                                id,
                                version,
                                keep,
                            });
                        }
                    }
                }
            }
        }
    }
    let _ = fs::remove_file(&tmp);
    apps.sort_by(|a, b| a.id.cmp(&b.id));
    apps
}

fn wsl_distros() -> Vec<WslDistro> {
    #[cfg(not(windows))]
    {
        return Vec::new();
    }
    #[cfg(windows)]
    {
        let mut out = Vec::new();
        let hk = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
        let Ok(lxss) = hk.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Lxss") else {
            return out;
        };
        for name in lxss.enum_keys().filter_map(|k| k.ok()) {
            if name.eq_ignore_ascii_case("DefaultDistribution") {
                continue;
            }
            let Ok(sub) = lxss.open_subkey(&name) else {
                continue;
            };
            let distro: String = sub.get_value("DistributionName").unwrap_or_default();
            if distro.is_empty() {
                continue;
            }
            let base: String = sub.get_value("BasePath").unwrap_or_default();
            let base = base.trim_start_matches(r"\\?\").to_string();
            let ver: u32 = sub.get_value("Version").unwrap_or(2u32);
            let vhdx = PathBuf::from(&base).join("ext4.vhdx");
            let exists = vhdx.is_file();
            let gb = if exists { file_gb(&vhdx) } else { 0.0 };
            let kind = if distro.contains("docker-desktop") {
                "docker"
            } else {
                "linux"
            };
            out.push(WslDistro {
                name: distro,
                version: ver,
                base_path: base,
                vhdx: exists.then(|| vhdx.display().to_string()),
                gb: (gb * 100.0).round() / 100.0,
                keep: kind == "linux",
                kind: kind.into(),
            });
        }
        out
    }
}

fn dev_drive() -> DevDrive {
    let mut vhdx = Vec::new();
    for p in [
        "E:\\DevDrive\\Dev Drive.vhdx",
        "C:\\DevDrive\\Dev Drive.vhdx",
    ] {
        let path = Path::new(p);
        if path.is_file() {
            vhdx.push(VhdCandidate {
                path: p.into(),
                gb: (file_gb(path) * 100.0).round() / 100.0,
                on_c: p.starts_with("C:"),
            });
        }
    }
    let mut letter = None;
    let mut label = String::new();
    if Path::new("D:\\").exists() {
        let (lab, fs) = volume_info("D:\\");
        if fs.eq_ignore_ascii_case("ReFS") || lab.eq_ignore_ascii_case("Dev Drive") {
            letter = Some('D');
            label = lab;
        }
    }
    let present = letter.is_some() || !vhdx.is_empty();
    DevDrive {
        present,
        letter,
        label,
        vhdx,
        keep: present,
    }
}

fn docker_data(local: &str) -> DockerData {
    let path = PathBuf::from(local).join(r"Docker\wsl\disk\docker_data.vhdx");
    let present = path.is_file();
    let gb = if present {
        (file_gb(&path) * 100.0).round() / 100.0
    } else {
        0.0
    };
    DockerData {
        present,
        path: path.display().to_string(),
        gb,
        keep: false,
    }
}

fn brave(local: &str) -> BraveInfo {
    let root = PathBuf::from(local).join(r"BraveSoftware\Brave-Browser\User Data\Default");
    let ext_dir = root.join("Extensions");
    let mut extensions = Vec::new();
    if ext_dir.is_dir() {
        if let Ok(rd) = fs::read_dir(&ext_dir) {
            for e in rd.flatten() {
                let id = e.file_name().to_string_lossy().into_owned();
                if id.len() != 32 || id == "Temp" {
                    continue;
                }
                let mut version = String::new();
                let mut name = id.clone();
                if let Ok(vers) = fs::read_dir(e.path()) {
                    if let Some(v) = vers.flatten().find(|x| x.path().is_dir()) {
                        version = v.file_name().to_string_lossy().into_owned();
                        let mf = v.path().join("manifest.json");
                        if let Ok(raw) = fs::read_to_string(mf) {
                            if let Ok(j) = serde_json::from_str::<serde_json::Value>(&raw) {
                                if let Some(n) = j.get("name").and_then(|x| x.as_str()) {
                                    if !n.starts_with("__MSG_") {
                                        name = n.to_string();
                                    }
                                }
                            }
                        }
                    }
                }
                extensions.push(BraveExt {
                    url: format!("https://chromewebstore.google.com/detail/{id}"),
                    id,
                    name,
                    version,
                });
            }
        }
    }
    let bm = root.join("Bookmarks");
    let bookmarks_bytes = fs::metadata(&bm).map(|m| m.len()).unwrap_or(0);
    BraveInfo {
        present: root.is_dir(),
        bookmarks_path: bm.is_file().then(|| bm.display().to_string()),
        bookmarks_bytes,
        extensions,
        keep: true,
    }
}

fn dotfiles(user_profile: &str, local: &str) -> Vec<Dotfile> {
    let docs =
        PathBuf::from(user_profile).join(r"Documents\PowerShell\Microsoft.PowerShell_profile.ps1");
    let items = [
        ("wslconfig", PathBuf::from(user_profile).join(".wslconfig")),
        ("gitconfig", PathBuf::from(user_profile).join(".gitconfig")),
        (
            "sshConfig",
            PathBuf::from(user_profile).join(r".ssh\config"),
        ),
        (
            "sshKnownHosts",
            PathBuf::from(user_profile).join(r".ssh\known_hosts"),
        ),
        (
            "grokConfig",
            PathBuf::from(user_profile).join(r".grok\config.toml"),
        ),
        (
            "terminal",
            PathBuf::from(local)
                .join(r"Packages\Microsoft.WindowsTerminal_8wekyb3d8bbwe\LocalState\settings.json"),
        ),
        ("psProfile", docs),
    ];
    items
        .into_iter()
        .map(|(key, path)| Dotfile {
            key: key.into(),
            present: path.is_file(),
            path: path.display().to_string(),
        })
        .collect()
}
