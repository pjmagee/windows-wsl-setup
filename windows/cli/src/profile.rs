//! Named profiles: list, show, new, add, remove, save.

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
        let windows = store
            .windows
            .get(&b.windows)
            .cloned()
            .ok_or_else(|| format!("bundle {id} points at missing windows profile {}", b.windows))?;
        let linux = store
            .linux
            .get(&b.linux)
            .cloned()
            .ok_or_else(|| format!("bundle {id} points at missing linux profile {}", b.linux))?;
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

pub fn new_from(store: &mut Store, id: &str, from: &str) -> Result<Resolved, String> {
    let id = sanitize_id(id)?;
    let mut r = resolve(store, from)?;
    r.bundle.id = id.clone();
    r.bundle.name = id.clone();
    r.bundle.windows = id.clone();
    r.bundle.linux = id.clone();
    r.windows.id = id.clone();
    r.windows.name = id.clone();
    r.linux.id = id.clone();
    r.linux.name = id.clone();
    r.source = "user".into();
    store.linux.insert(id.clone(), r.linux.clone());
    store.windows.insert(id.clone(), r.windows.clone());
    store.bundles.insert(id.clone(), r.bundle.clone());
    store.linux_source.insert(id.clone(), "user");
    store.windows_source.insert(id.clone(), "user");
    store.bundle_source.insert(id.clone(), "user");
    Ok(r)
}

pub fn add(
    store: &mut Store,
    id: &str,
    linux: &[String],
    windows: &[String],
) -> Result<Resolved, String> {
    let id = sanitize_id(id)?;
    if !store.bundles.contains_key(&id) {
        new_from(store, &id, "default")?;
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
    r.windows.packages.retain(|t| !windows.iter().any(|x| x == t));
    store.linux.insert(id.clone(), r.linux.clone());
    store.windows.insert(id.clone(), r.windows.clone());
    Ok(resolve(store, &id)?)
}

pub fn save(store: &Store, id: &str) -> Result<serde_json::Value, String> {
    let r = resolve(store, id)?;
    let root = user_profiles();
    write_json(&root.join("linux").join(format!("{}.json", r.linux.id)), &r.linux)?;
    write_json(
        &root.join("windows").join(format!("{}.json", r.windows.id)),
        &r.windows,
    )?;
    write_json(&root.join("bundles").join(format!("{}.json", r.bundle.id)), &r.bundle)?;
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
        new_from(&mut s, "mine", "home").unwrap();
        add(&mut s, "mine", &["kubectl".into()], &[]).unwrap();
        let r = resolve(&s, "mine").unwrap();
        assert!(r.linux.tools.contains(&"kubectl".into()));
        assert!(r.linux.tools.contains(&"uv".into()));
    }
}
