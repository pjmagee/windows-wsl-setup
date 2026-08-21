//! Apply a bundle: winget install (never uninstall) + optional New WSL.

use serde::Serialize;

use crate::catalog::{LinuxProfileDoc, Store};
use crate::new_wsl;
use crate::profile::{self, Resolved};
use crate::restore;
use crate::terminal;

#[derive(Debug, Clone, Serialize)]
pub struct Step {
    pub step: String,
    pub ok: bool,
    pub detail: String,
}

pub fn apply_windows(store: &Store, ids: &[String]) -> Vec<Step> {
    let mut ids: Vec<String> = ids.to_vec();
    ids.sort_by_key(|id| store.windows_pkg(id).map(|p| p.priority).unwrap_or(100));
    let mut out = Vec::new();
    for id in &ids {
        let r = restore::install_id(id);
        out.push(Step {
            step: format!("winget {id}"),
            ok: r.ok,
            detail: r.detail,
        });
    }
    out
}

pub fn apply_linux(linux: &LinuxProfileDoc, create_wsl: bool, distro: &str) -> Vec<Step> {
    let distro = match new_wsl::parse_distro(distro) {
        Ok(d) => d,
        Err(e) => return vec![fail("distro", e)],
    };
    let mut out = Vec::new();
    if create_wsl {
        match new_wsl::ensure_wsl() {
            Ok(s) => out.push(ok("wsl", s)),
            Err(e) => {
                out.push(fail("wsl", e));
                return out;
            }
        }
        match new_wsl::install_distro(distro) {
            Ok(s) => out.push(ok("distro", s)),
            Err(e) => {
                out.push(fail("distro", e));
                return out;
            }
        }
        match terminal::sync(None) {
            Ok(r) => out.push(ok(
                "terminal",
                format!("{} ({})", r.profiles.join(", "), r.detail),
            )),
            Err(e) => out.push(fail("terminal", e)),
        }
        match new_wsl::wait_for_root(distro) {
            Ok(s) => out.push(ok("boot", s)),
            Err(e) => {
                out.push(fail("boot", e));
                return out;
            }
        }
        match new_wsl::create_user_if_needed(distro) {
            Ok(s) => out.push(ok("user", s)),
            Err(e) => {
                out.push(fail("user", e));
                return out;
            }
        }
        match new_wsl::ensure_passwordless_sudo(distro) {
            Ok(s) => out.push(ok("sudo", s)),
            Err(e) => {
                out.push(fail("sudo", e));
                return out;
            }
        }
        match new_wsl::maybe_set_default_distro(distro) {
            Ok((changed, s)) => {
                out.push(ok("default-distro", s));
                if changed {
                    match terminal::sync(Some(distro)) {
                        Ok(r) => out.push(ok(
                            "terminal-default",
                            r.default_profile
                                .unwrap_or_else(|| distro.to_string()),
                        )),
                        Err(e) => out.push(fail("terminal-default", e)),
                    }
                }
            }
            Err(e) => out.push(fail("default-distro", e)),
        }
    }
    if new_wsl::is_blank_profile(&linux.id) {
        match new_wsl::mark_blank_profile(distro) {
            Ok(s) => out.push(ok("install.sh", s)),
            Err(e) => out.push(fail("install.sh", e)),
        }
        return out;
    }
    let json = serde_json::to_string(linux).ok();
    match new_wsl::install_toolchain(distro, &linux.id, json.as_deref()) {
        Ok(s) => {
            let tail: String = s
                .lines()
                .rev()
                .take(8)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
                .join("\n");
            out.push(ok("install.sh", tail));
        }
        Err(e) => {
            let tail: String = e
                .lines()
                .rev()
                .take(12)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
                .join("\n");
            out.push(fail("install.sh", tail));
        }
    }
    out
}

pub fn apply_resolved(store: &Store, r: &Resolved, windows: bool, linux: bool) -> Vec<Step> {
    let mut out = Vec::new();
    if windows {
        out.extend(apply_windows(store, &r.windows.packages));
    }
    if linux {
        out.extend(apply_linux(
            &r.linux,
            r.bundle.wsl.create_if_missing,
            &r.bundle.wsl.distro,
        ));
    }
    out
}

pub fn apply_id(
    store: &Store,
    id: &str,
    windows: bool,
    linux: bool,
    distro: Option<&str>,
) -> Result<Vec<Step>, String> {
    let mut r = profile::resolve(store, id)?;
    if let Some(d) = distro {
        r.bundle.wsl.distro = new_wsl::parse_distro(d)?.to_string();
    } else {
        r.bundle.wsl.distro = new_wsl::parse_distro(&r.bundle.wsl.distro)?.to_string();
    }
    Ok(apply_resolved(store, &r, windows, linux))
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
