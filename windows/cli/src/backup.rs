use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;

use crate::model::*;

#[allow(dead_code)]
pub struct Selection<'a> {
    pub kit_root: String,
    pub profile: String,
    pub inv: &'a Inventory,
    pub dest_idx: usize, // kept for restore metadata later
    pub app_keep: &'a [bool],
    pub wsl_keep: &'a [bool],
    pub dev_drive: bool,
    pub docker: bool,
    pub browser: bool,
    pub dotfiles: bool,
}

pub fn write_kit(sel: &Selection<'_>) -> Result<String, String> {
    let root = Path::new(&sel.kit_root);
    if sel
        .kit_root
        .chars()
        .next()
        .map(|c| c.eq_ignore_ascii_case(&'c'))
        .unwrap_or(false)
        && sel.kit_root.chars().nth(1) == Some(':')
    {
        return Err("refusing to write a kit on C:".into());
    }
    for sub in [
        "inventory",
        "config/ssh",
        "config/git",
        "config/grok",
        "config/terminal",
        "config/powershell",
        "browser",
        "apps",
        "vhdx",
    ] {
        fs::create_dir_all(root.join(sub)).map_err(|e| e.to_string())?;
    }

    if sel.dotfiles {
        for d in &sel.inv.dotfiles {
            if !d.present {
                continue;
            }
            let dest = match d.key.as_str() {
                "wslconfig" => root.join("config/wslconfig"),
                "gitconfig" => root.join("config/git/gitconfig"),
                "sshConfig" => root.join("config/ssh/config"),
                "sshKnownHosts" => root.join("config/ssh/known_hosts"),
                "grokConfig" => root.join("config/grok/config.toml"),
                "terminal" => root.join("config/terminal/settings.json"),
                "psProfile" => root.join("config/powershell/Microsoft.PowerShell_profile.ps1"),
                _ => continue,
            };
            let _ = fs::copy(&d.path, dest);
        }
    }

    if sel.browser {
        if let Some(bm) = &sel.inv.brave.bookmarks_path {
            let _ = fs::copy(bm, root.join("browser/Bookmarks"));
            let bak = format!("{bm}.bak");
            let _ = fs::copy(&bak, root.join("browser/Bookmarks.bak"));
        }
        let mut html = String::from(
            "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><title>Brave extensions</title></head><body><h1>Install these Brave extensions</h1><ol>\n",
        );
        for e in &sel.inv.brave.extensions {
            html.push_str(&format!(
                "  <li><a href=\"{}\">{}</a></li>\n",
                e.url,
                e.name.replace('<', "")
            ));
        }
        html.push_str("</ol></body></html>\n");
        fs::write(root.join("browser/extensions.html"), html).map_err(|e| e.to_string())?;
        let ext_json =
            serde_json::to_string_pretty(&sel.inv.brave.extensions).map_err(|e| e.to_string())?;
        fs::write(root.join("inventory/brave-extensions.json"), ext_json)
            .map_err(|e| e.to_string())?;
    }

    let app_ids: Vec<&str> = sel
        .inv
        .apps
        .iter()
        .zip(sel.app_keep.iter())
        .filter(|(_, k)| **k)
        .map(|(a, _)| a.id.as_str())
        .collect();
    let selected_doc = serde_json::json!({
        "$schema": "https://aka.ms/winget-packages.schema.2.0.json",
        "Sources": [{
            "SourceDetails": { "Name": "winget" },
            "Packages": app_ids.iter().map(|id| serde_json::json!({ "PackageIdentifier": id })).collect::<Vec<_>>()
        }]
    });
    fs::write(
        root.join("apps/winget-selected.json"),
        serde_json::to_string_pretty(&selected_doc).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let kept_apps: Vec<_> = sel
        .inv
        .apps
        .iter()
        .zip(sel.app_keep.iter())
        .filter(|(_, k)| **k)
        .map(|(a, _)| a)
        .collect();
    fs::write(
        root.join("inventory/apps.json"),
        serde_json::to_string_pretty(&kept_apps).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let export = root.join("apps/winget-export-raw.json");
    let _ = Command::new("winget")
        .args([
            "export",
            "--output",
            export.to_str().unwrap_or("winget-export-raw.json"),
            "--accept-source-agreements",
            "--disable-interactivity",
        ])
        .output();

    fs::write(
        root.join("inventory/linux-tools.json"),
        serde_json::to_string_pretty(&sel.inv.linux_tools).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;

    let wsl_names: Vec<&str> = sel
        .inv
        .wsl
        .iter()
        .zip(sel.wsl_keep.iter())
        .filter(|(_, k)| **k)
        .map(|(d, _)| d.name.as_str())
        .collect();

    let mut wsl_disks = Vec::new();
    for (d, keep) in sel.inv.wsl.iter().zip(sel.wsl_keep.iter()) {
        if !*keep {
            continue;
        }
        wsl_disks.push(serde_json::json!({
            "name": d.name,
            "vhdx": d.vhdx,
            "onDevDrive": d.vhdx.as_deref().map(|p| p.starts_with("D:\\") || p.contains("\\Dev Drive")).unwrap_or(false),
        }));
    }
    let dev_drive_disk = if sel.dev_drive {
        let live = sel.inv.dev_drive.vhdx.iter().find(|v| !v.on_c).or(sel
            .inv
            .dev_drive
            .vhdx
            .first());
        Some(serde_json::json!({
            "keep": true,
            "livePath": live.map(|v| v.path.clone()),
            "letter": sel.inv.dev_drive.letter.map(|c| c.to_string()),
        }))
    } else {
        None
    };

    if let Ok(exe) = std::env::current_exe() {
        let _ = fs::copy(&exe, root.join("windows-wsl-setup.exe"));
    }

    let kit = serde_json::json!({
        "schemaVersion": 1,
        "computer": sel.inv.computer,
        "user": sel.inv.user,
        "windowsUserProfile": sel.inv.user_profile,
        "linuxProfile": sel.profile,
        "linuxTools": sel.inv.linux_tools,
        "kitRoot": sel.kit_root,
        "selections": {
            "apps": app_ids,
            "wsl": wsl_names,
            "devDrive": sel.dev_drive,
            "dockerData": sel.docker,
            "browser": sel.browser,
            "dotfiles": sel.dotfiles
        },
        "wslDisks": wsl_disks,
        "devDriveDisk": dev_drive_disk,
    });
    fs::write(
        root.join("KIT.json"),
        serde_json::to_string_pretty(&kit).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;

    let mut start = fs::File::create(root.join("START-HERE.txt")).map_err(|e| e.to_string())?;
    writeln!(
        start,
        "Windows WSL Setup kit — {pc}\n\nOn the new Windows 11 PC:\n  1. Do not wipe this data drive.\n  2. Download windows-wsl-setup.exe from GitHub Releases (or use windows-wsl-setup.exe in this folder).\n  3. Run it and choose Restore.\n  4. Tick the winget packages to install, then Apply.\n\nKit folder: {kit}\n",
        pc = sel.inv.computer,
        kit = sel.kit_root,
    )
    .map_err(|e| e.to_string())?;

    Ok(sel.kit_root.clone())
}
