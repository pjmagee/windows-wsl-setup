use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;

use crate::model::*;

#[allow(dead_code)]
pub struct Selection<'a> {
    pub kit_root: String,
    pub profile: LinuxProfile,
    pub inv: &'a Inventory,
    pub dest_idx: usize, // kept for restore metadata later
    pub app_keep: &'a [bool],
    pub wsl_keep: &'a [bool],
    pub extra_home: &'a [bool],
    pub extra_work: &'a [bool],
    pub extras: &'a [LinuxTool],
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
        let ext_json = serde_json::to_string_pretty(&sel.inv.brave.extensions)
            .map_err(|e| e.to_string())?;
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

    let mut tools = sel.inv.linux_tools.clone();
    let mut ei = 0usize;
    for t in &mut tools.tools {
        if t.layer == "base" {
            continue;
        }
        if ei < sel.extra_home.len() {
            t.home = sel.extra_home[ei];
            t.work = sel.extra_work[ei];
        }
        ei += 1;
    }
    fs::write(
        root.join("inventory/linux-tools.json"),
        serde_json::to_string_pretty(&tools).map_err(|e| e.to_string())?,
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

    let kit = serde_json::json!({
        "schemaVersion": 1,
        "computer": sel.inv.computer,
        "user": sel.inv.user,
        "windowsUserProfile": sel.inv.user_profile,
        "linuxProfile": sel.profile.as_str(),
        "linuxTools": tools,
        "kitRoot": sel.kit_root,
        "repo": "https://github.com/pjmagee/wsl-setup.git",
        "selections": {
            "apps": app_ids,
            "wsl": wsl_names,
            "devDrive": sel.dev_drive,
            "dockerData": sel.docker,
            "browser": sel.browser,
            "dotfiles": sel.dotfiles
        }
    });
    fs::write(
        root.join("KIT.json"),
        serde_json::to_string_pretty(&kit).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;

    let profile = sel.profile.as_str();
    let agents = format!(
        r#"# Restore this machine (generated kit)

If you are an agent on a freshly reset Windows 11 PC and the human says
anything like **restore this machine**, **execute this file**. Do not invent
a second plan. Do not format data drives. Do not `wsl --unregister`.

- Kit: `{kit}`
- Old computer: {pc}
- Windows user: {user}
- Linux profile: `./install.sh {profile}` (inside Ubuntu-26.04)
- Repo: https://github.com/pjmagee/wsl-setup.git

## Order

1. Clone the repo if needed (HTTPS is fine).
2. `windows\host\Restore-Kit.ps1 -KitRoot '{kit}'`
3. `windows\bootstrap.ps1 -SkipLinuxInstall`
4. Copy `inventory/linux-tools.json` to `~/.config/wsl-setup/tools.json` if present.
5. Inside Ubuntu-26.04: `cd ~/code/wsl-setup && git pull && ./install.sh {profile}`

There is no universal profile. Base tools always install; extras follow the ticks in `linux-tools.json`.

## Selected

- Apps: {apps} winget ids
- WSL: {wsl}
- Dev Drive: {dd}
- Docker data: {dk}
- Brave: {br}
- Dotfiles: {df}

## Manual leftovers

- 1Password → Settings → Developer → Use the SSH agent
- OpenSSH Authentication Agent service off
- Steam library path / Docker Desktop WSL integration
"#,
        kit = sel.kit_root,
        pc = sel.inv.computer,
        user = sel.inv.user,
        profile = profile,
        apps = app_ids.len(),
        wsl = if wsl_names.is_empty() {
            "none".into()
        } else {
            wsl_names.join(", ")
        },
        dd = sel.dev_drive,
        dk = sel.docker,
        br = sel.browser,
        df = sel.dotfiles,
    );
    fs::write(root.join("AGENTS.md"), agents).map_err(|e| e.to_string())?;

    let mut start = fs::File::create(root.join("START-HERE.txt")).map_err(|e| e.to_string())?;
    writeln!(
        start,
        "windows-wsl-setup kit — {} {}\n\nOn the new PC, tell the agent:\n  Read {}\\AGENTS.md and restore this machine.\n",
        sel.inv.computer,
        sel.kit_root,
        sel.kit_root
    )
    .map_err(|e| e.to_string())?;

    Ok(sel.kit_root.clone())
}
