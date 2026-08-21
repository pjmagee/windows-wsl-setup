use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::kit::LoadedKit;
use crate::model::BraveExt;

pub struct InstallResult {
    pub id: String,
    pub ok: bool,
    pub detail: String,
}

pub fn install_id(id: &str) -> InstallResult {
    let out = Command::new("winget")
        .args([
            "install",
            "--id",
            id,
            "-e",
            "--accept-package-agreements",
            "--accept-source-agreements",
            "--disable-interactivity",
        ])
        .output();
    match out {
        Ok(o) => {
            let code = o.status.code().unwrap_or(-1);
            // -1978335189 = already installed
            let ok = o.status.success() || code == -1978335189;
            let tail = String::from_utf8_lossy(&o.stdout);
            let err = String::from_utf8_lossy(&o.stderr);
            let detail = if ok {
                if code == -1978335189 {
                    "already installed".into()
                } else {
                    "installed".into()
                }
            } else {
                format!("exit {code} {err} {tail}")
                    .chars()
                    .take(200)
                    .collect()
            };
            InstallResult {
                id: id.into(),
                ok,
                detail,
            }
        }
        Err(e) => InstallResult {
            id: id.into(),
            ok: false,
            detail: format!("winget missing: {e}"),
        },
    }
}

pub fn remount_dev_drive(kit: &LoadedKit) -> Result<String, String> {
    let Some(disk) = &kit.doc.dev_drive_disk else {
        return Ok("no Dev Drive in kit".into());
    };
    if !disk.keep {
        return Ok("Dev Drive not selected".into());
    }
    let path = disk
        .copied_to
        .as_ref()
        .or(disk.live_path.as_ref())
        .cloned()
        .ok_or_else(|| "kit has no Dev Drive VHDX path".to_string())?;
    let path = if Path::new(&path).is_file() {
        path
    } else {
        let alt = kit.dir.join("vhdx").join("Dev Drive.vhdx");
        if alt.is_file() {
            alt.display().to_string()
        } else {
            return Err(format!("VHDX not found: {path}"));
        }
    };
    let ps = format!(
        r#"
$ErrorActionPreference = 'Stop'
$path = '{}'
Mount-VHD -Path $path -ErrorAction SilentlyContinue
$vhd = Get-VHD -Path $path
if (-not $vhd.Attached) {{ Mount-VHD -Path $path; $vhd = Get-VHD -Path $path }}
$part = Get-Partition -DiskNumber $vhd.DiskNumber | Where-Object {{ $_.Size -gt 1GB }} | Select-Object -First 1
if (-not $part) {{ throw 'no data partition on Dev Drive VHDX' }}
if ($part.DriveLetter -ne 'D') {{
  if ($part.DriveLetter) {{
    Remove-PartitionAccessPath -DiskNumber $part.DiskNumber -PartitionNumber $part.PartitionNumber -AccessPath "$($part.DriveLetter):\"
  }}
  Set-Partition -DiskNumber $part.DiskNumber -PartitionNumber $part.PartitionNumber -NewDriveLetter D
}}
'ok D:'
"#,
        path.replace('\'', "''")
    );
    let tmp = std::env::temp_dir().join("wwm-mount-devdrive.ps1");
    fs::write(&tmp, ps).map_err(|e| e.to_string())?;
    let out = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            tmp.to_str().unwrap_or(""),
        ])
        .output()
        .map_err(|e| e.to_string())?;
    let _ = fs::remove_file(&tmp);
    if out.status.success() {
        Ok(format!("mounted Dev Drive as D: from {path}"))
    } else {
        Err(format!(
            "Mount-VHD failed (needs admin / Hyper-V PowerShell): {}",
            String::from_utf8_lossy(&out.stderr)
        ))
    }
}

pub fn restore_wsl(kit: &LoadedKit) -> Result<String, String> {
    let mut notes = Vec::new();
    let _ = Command::new("wsl.exe").arg("--shutdown").status();
    for d in &kit.doc.wsl_disks {
        let vhdx = resolve_wsl_vhdx(kit, d);
        let Some(vhdx) = vhdx else {
            notes.push(format!("{}: no ext4.vhdx in kit", d.name));
            continue;
        };
        let _ = Command::new("icacls.exe")
            .args([&vhdx, "/grant", "*S-1-5-83-0:(F)"])
            .status();
        let listed8 = Command::new("wsl.exe")
            .args(["-l", "-q"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).replace('\0', ""))
            .unwrap_or_default();
        if listed8.contains(&d.name) {
            notes.push(format!("{} already registered", d.name));
            continue;
        }
        let st = Command::new("wsl.exe")
            .args(["--import-in-place", &d.name, &vhdx])
            .output()
            .map_err(|e| e.to_string())?;
        if st.status.success() {
            notes.push(format!("{} imported", d.name));
        } else {
            notes.push(format!(
                "{} import failed: {}",
                d.name,
                String::from_utf8_lossy(&st.stderr).replace('\0', "")
            ));
        }
    }
    match crate::terminal::sync(None) {
        Ok(r) => notes.push(format!("terminal: {}", r.profiles.join(", "))),
        Err(e) => notes.push(format!("terminal: {e}")),
    }
    if notes.is_empty() {
        Ok("no WSL disks in kit".into())
    } else {
        Ok(notes.join("; "))
    }
}

fn resolve_wsl_vhdx(kit: &LoadedKit, d: &crate::kit::KitWsl) -> Option<String> {
    if let Some(p) = &d.vhdx {
        if Path::new(p).is_file() {
            return Some(p.clone());
        }
        let rel = kit.dir.join(p);
        if rel.is_file() {
            return Some(rel.display().to_string());
        }
    }
    let in_kit = kit.dir.join("vhdx").join(&d.name).join("ext4.vhdx");
    if in_kit.is_file() {
        return Some(in_kit.display().to_string());
    }
    let on_d = PathBuf::from(r"D:\WSL").join(&d.name).join("ext4.vhdx");
    if on_d.is_file() {
        return Some(on_d.display().to_string());
    }
    None
}

pub fn restore_browser(kit: &LoadedKit) -> Result<String, String> {
    if !kit.doc.selections.browser {
        return Ok("browser restore not selected".into());
    }
    let local = std::env::var("LOCALAPPDATA").unwrap_or_default();
    let dest_dir = PathBuf::from(&local).join(r"BraveSoftware\Brave-Browser\User Data\Default");
    let src_bm = kit.dir.join("browser/Bookmarks");
    let mut notes = Vec::new();
    if src_bm.is_file() {
        let _ = fs::create_dir_all(&dest_dir);
        match fs::copy(&src_bm, dest_dir.join("Bookmarks")) {
            Ok(_) => notes.push("bookmarks copied (close Brave first if it was open)".into()),
            Err(e) => notes.push(format!("bookmarks: {e}")),
        }
    }
    let html = kit.dir.join("browser/extensions.html");
    if html.is_file() {
        let _ = Command::new("cmd")
            .args(["/C", "start", "", &html.display().to_string()])
            .status();
        notes.push("opened extensions.html — Add to Brave on each store page".into());
    } else if let Ok(raw) = fs::read_to_string(kit.dir.join("inventory/brave-extensions.json")) {
        if let Ok(exts) = serde_json::from_str::<Vec<BraveExt>>(&raw) {
            let path = write_extensions_html(kit.dir.join("browser/extensions.html"), &exts)?;
            let _ = Command::new("cmd")
                .args(["/C", "start", "", &path])
                .status();
            notes.push("opened extensions.html".into());
        }
    }
    if notes.is_empty() {
        Ok("no Brave data in kit".into())
    } else {
        Ok(notes.join("; "))
    }
}

pub fn write_extensions_html(path: PathBuf, exts: &[BraveExt]) -> Result<String, String> {
    if let Some(p) = path.parent() {
        fs::create_dir_all(p).map_err(|e| e.to_string())?;
    }
    let mut html = String::from(
        r#"<!doctype html><html lang="en"><head><meta charset="utf-8"><title>Brave extensions</title>
<style>body{font-family:Segoe UI,sans-serif;max-width:720px;margin:40px auto;background:#11161c;color:#e8eef6}
a{color:#7dcea0}ol{line-height:1.9}.note{background:#1b222b;padding:12px 16px;border-radius:8px}</style></head><body>
<h1>Install these Brave extensions</h1>
<p class="note">Brave is (or will be) installed via winget. Click <b>Add to Brave</b> on each Chrome Web Store page. Bookmarks were copied into the Brave profile if present.</p><ol>
"#,
    );
    for e in exts {
        html.push_str(&format!(
            "  <li><a href=\"{}\">{}</a></li>\n",
            e.url,
            e.name.replace('<', "")
        ));
    }
    html.push_str("</ol></body></html>\n");
    fs::write(&path, html).map_err(|e| e.to_string())?;
    Ok(path.display().to_string())
}
