//! Apply a bundle: winget install (never uninstall) + optional New WSL.

use serde::Serialize;

use crate::catalog::{LinuxProfileDoc, Store};
use crate::new_wsl;
use crate::profile::{self, Resolved};
use crate::restore;

#[derive(Debug, Clone, Serialize)]
pub struct Step {
    pub step: String,
    pub ok: bool,
    pub detail: String,
}

pub fn apply_windows(ids: &[String]) -> Vec<Step> {
    let mut out = Vec::new();
    for id in ids {
        let r = restore::install_id(id);
        out.push(Step {
            step: format!("winget {id}"),
            ok: r.ok,
            detail: r.detail,
        });
    }
    out
}

pub fn apply_linux(linux: &LinuxProfileDoc, create_wsl: bool) -> Vec<Step> {
    let mut out = Vec::new();
    if create_wsl {
        match new_wsl::ensure_wsl() {
            Ok(s) => out.push(ok("wsl", s)),
            Err(e) => {
                out.push(fail("wsl", e));
                return out;
            }
        }
        match new_wsl::install_distro() {
            Ok(s) => out.push(ok("distro", s)),
            Err(e) => {
                out.push(fail("distro", e));
                return out;
            }
        }
        match new_wsl::wait_for_root() {
            Ok(s) => out.push(ok("boot", s)),
            Err(e) => {
                out.push(fail("boot", e));
                return out;
            }
        }
        match new_wsl::create_user_if_needed() {
            Ok(s) => out.push(ok("user", s)),
            Err(e) => {
                out.push(fail("user", e));
                return out;
            }
        }
        match new_wsl::ensure_passwordless_sudo() {
            Ok(s) => out.push(ok("sudo", s)),
            Err(e) => {
                out.push(fail("sudo", e));
                return out;
            }
        }
        match new_wsl::set_default_distro() {
            Ok(s) => out.push(ok("default-distro", s)),
            Err(e) => out.push(fail("default-distro", e)),
        }
    }
    let json = serde_json::to_string(linux).ok();
    match new_wsl::install_toolchain(&linux.id, json.as_deref()) {
        Ok(s) => {
            let tail: String = s.lines().rev().take(8).collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>().join("\n");
            out.push(ok("install.sh", tail));
        }
        Err(e) => {
            let tail: String = e.lines().rev().take(12).collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>().join("\n");
            out.push(fail("install.sh", tail));
        }
    }
    out
}

pub fn apply_resolved(r: &Resolved, windows: bool, linux: bool) -> Vec<Step> {
    let mut out = Vec::new();
    if windows {
        out.extend(apply_windows(&r.windows.packages));
    }
    if linux {
        out.extend(apply_linux(&r.linux, r.bundle.wsl.create_if_missing));
    }
    out
}

pub fn apply_id(store: &Store, id: &str, windows: bool, linux: bool) -> Result<Vec<Step>, String> {
    let r = profile::resolve(store, id)?;
    Ok(apply_resolved(&r, windows, linux))
}

fn ok(step: &str, detail: String) -> Step {
    Step {
        step: step.into(),
        ok: true,
        detail,
    }
}

fn fail(step: &str, detail: String) -> Step {
    Step {
        step: step.into(),
        ok: false,
        detail,
    }
}
