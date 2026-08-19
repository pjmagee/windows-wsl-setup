//! Draft a bundle from the winget packages on this PC.

use serde::Serialize;

use crate::catalog::{BundleDoc, LinuxProfileDoc, Store, WindowsProfileDoc, WslSpec};
use crate::inventory;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Suggestion {
    pub windows: Vec<String>,
    pub linux: Vec<String>,
    pub skipped_prefer_linux: Vec<Skipped>,
    pub unmapped: Vec<String>,
    pub bundle: BundleDoc,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Skipped {
    pub windows: String,
    pub linux: Option<String>,
    pub reason: &'static str,
}

pub fn from_machine(store: &Store) -> Suggestion {
    let apps = inventory::winget_apps_pub();
    let mut windows = Vec::new();
    let mut linux = Vec::new();
    let mut skipped = Vec::new();
    let mut unmapped = Vec::new();
    for a in apps {
        let prefer = store.prefer_linux(&a.id);
        let equiv = store.linux_equivalent(&a.id);
        if prefer {
            if let Some(lid) = equiv {
                if !linux.iter().any(|x| x == &lid) {
                    linux.push(lid.clone());
                }
                skipped.push(Skipped {
                    windows: a.id,
                    linux: Some(lid),
                    reason: "preferLinux",
                });
            } else {
                skipped.push(Skipped {
                    windows: a.id.clone(),
                    linux: None,
                    reason: "preferLinux-no-map",
                });
                unmapped.push(a.id);
            }
            continue;
        }
        if store.windows_pkg(&a.id).is_some() {
            if !windows.iter().any(|x| x == &a.id) {
                windows.push(a.id);
            }
            continue;
        }
        let cat = store.windows_category(&a.id);
        if cat != "other" {
            if !windows.iter().any(|x| x == &a.id) {
                windows.push(a.id);
            }
        } else {
            unmapped.push(a.id);
        }
    }
    if let Some(home) = store.linux.get("home") {
        for t in &home.tools {
            if !linux.iter().any(|x| x == t) {
                linux.push(t.clone());
            }
        }
    }
    Suggestion {
        windows: windows.clone(),
        linux: linux.clone(),
        skipped_prefer_linux: skipped,
        unmapped,
        bundle: BundleDoc {
            schema_version: 1,
            id: "suggested".into(),
            name: "Suggested from this PC".into(),
            windows: "suggested".into(),
            linux: "suggested".into(),
            wsl: WslSpec::default(),
        },
    }
}

#[allow(dead_code)]
pub fn as_docs(s: &Suggestion) -> (WindowsProfileDoc, LinuxProfileDoc, BundleDoc) {
    (
        WindowsProfileDoc {
            schema_version: 1,
            id: "suggested".into(),
            name: "Suggested".into(),
            packages: s.windows.clone(),
        },
        LinuxProfileDoc {
            schema_version: 1,
            id: "suggested".into(),
            name: "Suggested".into(),
            tools: s.linux.clone(),
        },
        s.bundle.clone(),
    )
}
