//! Named profiles: list, show, new, add, remove, save, delete.

use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::catalog::{
    sanitize_id, user_profiles, write_json, BundleDoc, LinuxProfileDoc, Store, WindowsProfileDoc,
    WslSpec,
};

#[derive(Debug, Clone, Serialize)]
pub struct Resolved {
    pub bundle: BundleDoc,
    pub windows: WindowsProfileDoc,
    pub linux: LinuxProfileDoc,
    pub source: String,
}

pub fn resolve(store: &Store, id: &str) -> Result<Resolved, String> {
    let id = sanitize_id(id)?;
    if let Some(b) = store.bundles.get(&id) {
        let windows = store.windows.get(&b.windows).cloned().ok_or_else(|| {
            format!(
                "bundle {id} points at missing windows profile {}",
                b.windows
            )
        })?;
        let linux =
            store.linux.get(&b.linux).cloned().ok_or_else(|| {
                format!("bundle {id} points at missing linux profile {}", b.linux)
            })?;
        let source = store.bundle_source.get(&id).copied().unwrap_or("unknown");
        return Ok(Resolved {
            bundle: b.clone(),
            windows,
            linux,
            source: source.into(),
        });
    }
    // Bare linux or windows id: synthesize a bundle.
    if store.linux.contains_key(&id) || store.windows.contains_key(&id) {
        let linux = store
            .linux
            .get(&id)
            .cloned()
            .or_else(|| store.linux.get("home").cloned())
            .ok_or_else(|| format!("no linux profile {id}"))?;
        let windows = store
            .windows
            .get(&id)
            .cloned()
            .or_else(|| store.windows.get("default").cloned())
            .ok_or_else(|| format!("no windows profile {id}"))?;
        return Ok(Resolved {
            bundle: BundleDoc {
                schema_version: 1,
                id: id.clone(),
                name: linux.name.clone(),
                windows: windows.id.clone(),
                linux: linux.id.clone(),
                wsl: WslSpec::default(),
            },
            windows,
            linux,
            source: "synthesized".into(),
        });
    }
    Err(format!("unknown profile {id}"))
}

pub fn list_json(store: &Store) -> serde_json::Value {
    let bundles: Vec<_> = store
        .bundles
        .values()
        .map(|b| {
            serde_json::json!({
                "id": b.id,
                "name": b.name,
                "kind": "bundle",
                "windows": b.windows,
                "linux": b.linux,
                "source": store.bundle_source.get(&b.id).copied().unwrap_or(""),
            })
        })
        .collect();
    let linux: Vec<_> = store
        .linux
        .values()
        .map(|p| {
            serde_json::json!({
                "id": p.id,
                "name": p.name,
                "kind": "linux",
                "tools": p.tools.len(),
                "source": store.linux_source.get(&p.id).copied().unwrap_or(""),
            })
        })
        .collect();
    let windows: Vec<_> = store
        .windows
        .values()
        .map(|p| {
            serde_json::json!({
                "id": p.id,
                "name": p.name,
                "kind": "windows",
                "packages": p.packages.len(),
                "source": store.windows_source.get(&p.id).copied().unwrap_or(""),
            })
        })
        .collect();
    serde_json::json!({ "bundles": bundles, "linux": linux, "windows": windows })
}

pub fn show_json(store: &Store, id: &str) -> Result<serde_json::Value, String> {
    let r = resolve(store, id)?;
    Ok(serde_json::to_value(&r).unwrap_or_default())
}

pub fn display_name(raw: &str, id: &str, name: Option<&str>) -> String {
    let from_flag = name.map(str::trim).filter(|s| !s.is_empty());
    if let Some(n) = from_flag {
        return n.to_string();
    }
    let raw = raw.trim();
    if raw.is_empty() {
        id.to_string()
    } else {
        raw.to_string()
    }
}

pub fn new_from(
    store: &mut Store,
    raw: &str,
    from: &str,
    name: Option<&str>,
) -> Result<Resolved, String> {
    let id = sanitize_id(raw)?;
    let display = display_name(raw, &id, name);
    let mut r = resolve(store, from)?;
    r.bundle.id = id.clone();
    r.bundle.name = display.clone();
    r.bundle.windows = id.clone();
    r.bundle.linux = id.clone();
    r.windows.id = id.clone();
    r.windows.name = display.clone();
    r.linux.id = id.clone();
    r.linux.name = display;
    r.source = "user".into();
    store.linux.insert(id.clone(), r.linux.clone());
    store.windows.insert(id.clone(), r.windows.clone());
    store.bundles.insert(id.clone(), r.bundle.clone());
    store.linux_source.insert(id.clone(), "user");
    store.windows_source.insert(id.clone(), "user");
    store.bundle_source.insert(id.clone(), "user");
    Ok(r)
}

/// Write a user bundle from the current tick lists. Refuses shipped ids.
pub fn put_user(
    store: &mut Store,
    raw_name: &str,
    windows: Vec<String>,
    linux: Vec<String>,
    wsl: WslSpec,
) -> Result<Resolved, String> {
    let id = sanitize_id(raw_name)?;
    if store.bundle_source.get(&id) == Some(&"shipped") {
        return Err(format!("{id} is a shipped profile; pick another name"));
    }
    let name = display_name(raw_name, &id, None);
    store.windows.insert(
        id.clone(),
        WindowsProfileDoc {
            schema_version: 1,
            id: id.clone(),
            name: name.clone(),
            packages: windows,
        },
    );
    store.linux.insert(
        id.clone(),
        LinuxProfileDoc {
            schema_version: 1,
            id: id.clone(),
            name: name.clone(),
            tools: linux,
        },
    );
    store.bundles.insert(
        id.clone(),
        BundleDoc {
            schema_version: 1,
            id: id.clone(),
            name,
            windows: id.clone(),
            linux: id.clone(),
            wsl,
        },
    );
    store.linux_source.insert(id.clone(), "user");
    store.windows_source.insert(id.clone(), "user");
    store.bundle_source.insert(id.clone(), "user");
    resolve(store, &id)
}

pub fn add(
    store: &mut Store,
    id: &str,
    linux: &[String],
    windows: &[String],
) -> Result<Resolved, String> {
    let id = sanitize_id(id)?;
    if !store.bundles.contains_key(&id) {
        new_from(store, &id, "default", None)?;
    }
    let mut r = resolve(store, &id)?;
    for t in linux {
        if store.linux_tool(t).is_none() {
            return Err(format!("unknown linux catalog id {t}"));
        }
        if !r.linux.tools.iter().any(|x| x == t) {
            r.linux.tools.push(t.clone());
        }
    }
    for p in windows {
        if !r.windows.packages.iter().any(|x| x == p) {
            r.windows.packages.push(p.clone());
        }
    }
    store.linux.insert(id.clone(), r.linux.clone());
    store.windows.insert(id.clone(), r.windows.clone());
    Ok(resolve(store, &id)?)
}

pub fn remove(
    store: &mut Store,
    id: &str,
    linux: &[String],
    windows: &[String],
) -> Result<Resolved, String> {
    let id = sanitize_id(id)?;
    let mut r = resolve(store, &id)?;
    r.linux.tools.retain(|t| !linux.iter().any(|x| x == t));
    r.windows
        .packages
        .retain(|t| !windows.iter().any(|x| x == t));
    store.linux.insert(id.clone(), r.linux.clone());
    store.windows.insert(id.clone(), r.windows.clone());
    Ok(resolve(store, &id)?)
}

fn unlink(path: &Path) -> Result<bool, String> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(true),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(format!("{}: {e}", path.display())),
    }
}

fn reclaim_linux(
    store: &mut Store,
    shipped: &Store,
    id: &str,
    root: &Path,
) -> Result<Vec<PathBuf>, String> {
    if store.bundles.values().any(|b| b.linux == id) {
        return Ok(Vec::new());
    }
    reclaim_side(store, shipped, id, root, "linux")
}

fn reclaim_windows(
    store: &mut Store,
    shipped: &Store,
    id: &str,
    root: &Path,
) -> Result<Vec<PathBuf>, String> {
    if store.bundles.values().any(|b| b.windows == id) {
        return Ok(Vec::new());
    }
    reclaim_side(store, shipped, id, root, "windows")
}

fn reclaim_side(
    store: &mut Store,
    shipped: &Store,
    id: &str,
    root: &Path,
    kind: &str,
) -> Result<Vec<PathBuf>, String> {
    let user = match kind {
        "linux" => store.linux_source.get(id) == Some(&"user"),
        _ => store.windows_source.get(id) == Some(&"user"),
    };
    if !user {
        return Ok(Vec::new());
    }
    let path = root.join(kind).join(format!("{id}.json"));
    let mut files = Vec::new();
    if unlink(&path)? {
        files.push(path);
    }
    match kind {
        "linux" => {
            if let Some(p) = shipped.linux.get(id) {
                store.linux.insert(id.into(), p.clone());
                store.linux_source.insert(id.into(), "shipped");
            } else {
                store.linux.remove(id);
                store.linux_source.remove(id);
            }
        }
        _ => {
            if let Some(p) = shipped.windows.get(id) {
                store.windows.insert(id.into(), p.clone());
                store.windows_source.insert(id.into(), "shipped");
            } else {
                store.windows.remove(id);
                store.windows_source.remove(id);
            }
        }
    }
    Ok(files)
}

/// Delete a user bundle and its unused linux/windows files. Restores shipped overlays.
pub fn delete(store: &mut Store, id: &str) -> Result<serde_json::Value, String> {
    delete_in(store, id, &user_profiles())
}

fn delete_in(store: &mut Store, raw: &str, root: &Path) -> Result<serde_json::Value, String> {
    let id = sanitize_id(raw)?;
    if !store.bundles.contains_key(&id) {
        return Err(format!("unknown profile {id}"));
    }
    if store.bundle_source.get(&id) != Some(&"user") {
        return Err(format!("{id} is a shipped profile"));
    }
    let bundle = store.bundles.get(&id).cloned().expect("checked");
    let linux_id = bundle.linux.clone();
    let windows_id = bundle.windows.clone();

    store.bundles.remove(&id);
    store.bundle_source.remove(&id);

    let mut files = Vec::new();
    let bundle_path = root.join("bundles").join(format!("{id}.json"));
    if unlink(&bundle_path)? {
        files.push(bundle_path);
    }

    let shipped = Store::shipped()?;
    files.extend(reclaim_linux(store, &shipped, &linux_id, root)?);
    files.extend(reclaim_windows(store, &shipped, &windows_id, root)?);

    let mut restored = false;
    if let Some(b) = shipped.bundles.get(&id) {
        store.bundles.insert(id.clone(), b.clone());
        store.bundle_source.insert(id.clone(), "shipped");
        restored = true;
    }

    Ok(serde_json::json!({
        "deleted": id,
        "restored": if restored { serde_json::Value::String("shipped".into()) } else { serde_json::Value::Null },
        "files": files,
    }))
}

pub fn save(store: &Store, id: &str) -> Result<serde_json::Value, String> {
    let r = resolve(store, id)?;
    let root = user_profiles();
    write_json(
        &root.join("linux").join(format!("{}.json", r.linux.id)),
        &r.linux,
    )?;
    write_json(
        &root.join("windows").join(format!("{}.json", r.windows.id)),
        &r.windows,
    )?;
    write_json(
        &root.join("bundles").join(format!("{}.json", r.bundle.id)),
        &r.bundle,
    )?;
    Ok(serde_json::json!({
        "saved": r.bundle.id,
        "dir": root,
    }))
}

pub fn search_linux(store: &Store, q: &str) -> serde_json::Value {
    let q = q.to_ascii_lowercase();
    let hits: Vec<_> = store
        .linux_catalog
        .tools
        .iter()
        .filter(|t| {
            t.id.to_ascii_lowercase().contains(&q)
                || t.name.to_ascii_lowercase().contains(&q)
                || t.pkg.to_ascii_lowercase().contains(&q)
                || t.category.to_ascii_lowercase().contains(&q)
        })
        .collect();
    serde_json::json!({ "query": q, "hits": hits })
}

pub fn map_windows(store: &Store, id: &str) -> serde_json::Value {
    let cat = store.windows_category(id);
    let prefer = store.prefer_linux(id);
    let linux = store.linux_equivalent(id);
    let known = store.windows_pkg(id).cloned();
    serde_json::json!({
        "windows": id,
        "category": cat,
        "preferLinux": prefer,
        "linux": linux,
        "catalog": known,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_linux_to_copy() {
        let mut s = Store::shipped().unwrap();
        new_from(&mut s, "mine", "home", None).unwrap();
        add(&mut s, "mine", &["kubectl".into()], &[]).unwrap();
        let r = resolve(&s, "mine").unwrap();
        assert!(r.linux.tools.contains(&"kubectl".into()));
        assert!(r.linux.tools.contains(&"uv".into()));
    }

    #[test]
    fn new_from_keeps_display_name() {
        let mut s = Store::shipped().unwrap();
        let r = new_from(&mut s, "Media PC", "home", None).unwrap();
        assert_eq!(r.bundle.id, "media-pc");
        assert_eq!(r.bundle.name, "Media PC");
        let r = new_from(&mut s, "lab", "home", Some("Home lab")).unwrap();
        assert_eq!(r.bundle.id, "lab");
        assert_eq!(r.bundle.name, "Home lab");
    }

    #[test]
    fn put_user_names_and_refuses_shipped() {
        let mut s = Store::shipped().unwrap();
        let e = put_user(&mut s, "home", vec![], vec![], WslSpec::default()).unwrap_err();
        assert!(e.contains("shipped"));
        let r = put_user(
            &mut s,
            "Media PC",
            vec!["Brave.Brave".into()],
            vec!["uv".into()],
            WslSpec::default(),
        )
        .unwrap();
        assert_eq!(r.bundle.id, "media-pc");
        assert_eq!(r.bundle.name, "Media PC");
        assert_eq!(r.windows.packages, ["Brave.Brave"]);
        assert_eq!(s.bundle_source.get("media-pc"), Some(&"user"));
    }

    fn scratch() -> PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static N: AtomicU64 = AtomicU64::new(0);
        let p = std::env::temp_dir().join(format!(
            "wws-del-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(p.join("bundles")).unwrap();
        std::fs::create_dir_all(p.join("linux")).unwrap();
        std::fs::create_dir_all(p.join("windows")).unwrap();
        p
    }

    #[test]
    fn delete_user_profile_and_files() {
        let mut s = Store::shipped().unwrap();
        let r = put_user(
            &mut s,
            "Media PC",
            vec!["Brave.Brave".into()],
            vec!["uv".into()],
            WslSpec::default(),
        )
        .unwrap();
        let dir = scratch();
        save_to(&s, &r.bundle.id, &dir).unwrap();
        assert!(dir.join("bundles/media-pc.json").is_file());
        let out = delete_in(&mut s, "media-pc", &dir).unwrap();
        assert_eq!(out["deleted"], "media-pc");
        assert!(out["restored"].is_null());
        assert!(!s.bundles.contains_key("media-pc"));
        assert!(!s.linux.contains_key("media-pc"));
        assert!(!dir.join("bundles/media-pc.json").exists());
        assert!(s.bundles.contains_key("home"));
    }

    #[test]
    fn delete_refuses_shipped_and_restores_overlay() {
        let mut s = Store::shipped().unwrap();
        let dir = scratch();
        assert!(delete_in(&mut s, "home", &dir)
            .unwrap_err()
            .contains("shipped"));
        new_from(&mut s, "home", "work", None).unwrap();
        assert_eq!(s.bundle_source.get("home"), Some(&"user"));
        let out = delete_in(&mut s, "home", &dir).unwrap();
        assert_eq!(out["restored"], "shipped");
        assert_eq!(s.bundle_source.get("home"), Some(&"shipped"));
        assert_eq!(s.linux["home"].id, "home");
    }

    fn save_to(store: &Store, id: &str, root: &Path) -> Result<(), String> {
        let r = resolve(store, id)?;
        write_json(
            &root.join("linux").join(format!("{}.json", r.linux.id)),
            &r.linux,
        )?;
        write_json(
            &root.join("windows").join(format!("{}.json", r.windows.id)),
            &r.windows,
        )?;
        write_json(
            &root.join("bundles").join(format!("{}.json", r.bundle.id)),
            &r.bundle,
        )?;
        Ok(())
    }
}
