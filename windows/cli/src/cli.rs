//! Machine-readable commands. JSON on stdout, errors on stderr.

use crate::catalog::Store;
use crate::profile;
use crate::suggest;
use crate::winget_search;

fn print_json(v: &impl serde::Serialize) -> i32 {
    match serde_json::to_string_pretty(v) {
        Ok(s) => {
            println!("{s}");
            0
        }
        Err(e) => {
            eprintln!("{e}");
            1
        }
    }
}

fn err(e: impl std::fmt::Display) -> i32 {
    eprintln!("{e}");
    1
}

fn store() -> Result<Store, String> {
    Store::load()
}

fn take_named<'a>(args: &'a [String], flag: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < args.len() {
        if args[i] == flag {
            if let Some(v) = args.get(i + 1) {
                if !v.starts_with("--") {
                    out.push(v.clone());
                    i += 2;
                    continue;
                }
            }
        }
        i += 1;
    }
    out
}

fn flag(args: &[String], name: &str) -> bool {
    args.iter().any(|a| a == name)
}

fn positional(args: &[String]) -> Vec<&str> {
    let mut out = Vec::new();
    let mut skip = false;
    for a in args {
        if skip {
            skip = false;
            continue;
        }
        if a == "--from"
            || a == "--linux"
            || a == "--windows"
            || a == "--profile"
            || a == "--distro"
            || a == "--name"
            || a == "--location"
            || a == "--src"
            || a == "--to"
        {
            skip = true;
            continue;
        }
        if a.starts_with("--") {
            continue;
        }
        out.push(a.as_str());
    }
    out
}

pub fn dispatch(cmd: &str, args: &[String]) -> i32 {
    match cmd {
        "catalog" => catalog(args),
        "profile" | "profiles" => profiles(args),
        "search" => search(args),
        "map" => map_cmd(args),
        "suggest" => suggest_cmd(),
        "apply" => apply_cmd(args),
        "distro" | "distros" => distro_cmd(cmd, args),
        "spec" | "opencli" => spec_cmd(),
        _ => {
            eprintln!("unknown command {cmd}");
            2
        }
    }
}

fn catalog(args: &[String]) -> i32 {
    let s = match store() {
        Ok(s) => s,
        Err(e) => return err(e),
    };
    match args.first().map(|x| x.as_str()) {
        Some("linux") => print_json(&s.linux_catalog),
        Some("windows") => print_json(&s.windows_catalog),
        None | Some("all") => print_json(&serde_json::json!({
            "linux": s.linux_catalog,
            "windows": s.windows_catalog,
        })),
        Some(other) => err(format!("catalog linux|windows, not {other}")),
    }
}

fn profiles(args: &[String]) -> i32 {
    if args.is_empty() {
        return crate::tui_profiles::run()
            .map(|_| 0)
            .unwrap_or_else(|e| err(e));
    }
    let sub = args[0].as_str();
    let rest = &args[1..];
    let mut s = match store() {
        Ok(s) => s,
        Err(e) => return err(e),
    };
    match sub {
        "list" => print_json(&profile::list_json(&s)),
        "show" => {
            let Some(id) = rest.first() else {
                return err("profile show <id>");
            };
            match profile::show_json(&s, id) {
                Ok(v) => print_json(&v),
                Err(e) => err(e),
            }
        }
        "new" => {
            let pos = positional(rest);
            let Some(id) = pos.first() else {
                return err("profile new <id> [--from home] [--name \"Media PC\"]");
            };
            let from = take_named(rest, "--from")
                .into_iter()
                .next()
                .unwrap_or_else(|| "default".into());
            let name = take_named(rest, "--name").into_iter().next();
            match profile::new_from(&mut s, id, &from, name.as_deref())
                .and_then(|r| profile::save(&s, &r.bundle.id))
            {
                Ok(v) => print_json(&v),
                Err(e) => err(e),
            }
        }
        "add" => {
            let pos = positional(rest);
            let Some(id) = pos.first() else {
                return err("profile add <id> --linux <tool> --windows <wingetId>");
            };
            let linux = take_named(rest, "--linux");
            let windows = take_named(rest, "--windows");
            match profile::add(&mut s, id, &linux, &windows).and_then(|_| profile::save(&s, id)) {
                Ok(v) => print_json(&v),
                Err(e) => err(e),
            }
        }
        "remove" => {
            let pos = positional(rest);
            let Some(id) = pos.first() else {
                return err("profile remove <id> --linux <tool> --windows <wingetId>");
            };
            let linux = take_named(rest, "--linux");
            let windows = take_named(rest, "--windows");
            match profile::remove(&mut s, id, &linux, &windows).and_then(|_| profile::save(&s, id))
            {
                Ok(v) => print_json(&v),
                Err(e) => err(e),
            }
        }
        "save" => {
            let Some(id) = rest.first() else {
                return err("profile save <id>");
            };
            match profile::save(&s, id) {
                Ok(v) => print_json(&v),
                Err(e) => err(e),
            }
        }
        "delete" => {
            let pos = positional(rest);
            let Some(id) = pos.first() else {
                return err("profile delete <id>");
            };
            match profile::delete(&mut s, id) {
                Ok(v) => print_json(&v),
                Err(e) => err(e),
            }
        }
        "tui" => crate::tui_profiles::run()
            .map(|_| 0)
            .unwrap_or_else(|e| err(e)),
        other => err(format!(
            "profile list|show|new|add|remove|delete|save, not {other}"
        )),
    }
}

fn search(args: &[String]) -> i32 {
    let s = match store() {
        Ok(s) => s,
        Err(e) => return err(e),
    };
    match args.first().map(|x| x.as_str()) {
        Some("linux") => {
            let q = args.get(1).map(|x| x.as_str()).unwrap_or("");
            print_json(&profile::search_linux(&s, q))
        }
        Some("winget") | Some("windows") => {
            let q = args.get(1).map(|x| x.as_str()).unwrap_or("");
            if q.is_empty() {
                return err("search winget <query>");
            }
            print_json(&winget_search::search(q))
        }
        _ => err("search linux <q>  |  search winget <q>"),
    }
}

fn map_cmd(args: &[String]) -> i32 {
    let Some(id) = args.first() else {
        return err("map <winget-id>");
    };
    let s = match store() {
        Ok(s) => s,
        Err(e) => return err(e),
    };
    print_json(&profile::map_windows(&s, id))
}

fn suggest_cmd() -> i32 {
    let s = match store() {
        Ok(s) => s,
        Err(e) => return err(e),
    };
    print_json(&suggest::from_machine(&s))
}

fn apply_cmd(args: &[String]) -> i32 {
    let pos = positional(args);
    let Some(id) = pos.first() else {
        return err("apply <profile> [--windows-only|--linux-only] [--distro fedora] [--location D:\\WSL]");
    };
    let windows_only = flag(args, "--windows-only");
    let linux_only = flag(args, "--linux-only");
    let do_win = !linux_only;
    let do_lin = !windows_only;
    let distro = take_named(args, "--distro").into_iter().next();
    let location = take_named(args, "--location").into_iter().next();
    let s = match store() {
        Ok(s) => s,
        Err(e) => return err(e),
    };
    match crate::apply::apply_id_at(&s, id, do_win, do_lin, distro.as_deref(), location.as_deref()) {
        Ok(steps) => print_json(&steps),
        Err(e) => err(e),
    }
}

fn spec_cmd() -> i32 {
    let raw = include_str!("../../../schema/wwm.opencli.json");
    print!("{raw}");
    if !raw.ends_with('\n') {
        println!();
    }
    0
}

fn distro_cmd(cmd: &str, args: &[String]) -> i32 {
    if cmd == "distros" || args.is_empty() || args[0] == "list" {
        return distros_cmd();
    }
    match args[0].as_str() {
        "sync" => match crate::terminal::sync(None) {
            Ok(r) => print_json(&r),
            Err(e) => err(e),
        },
        "remove" => distro_remove(&args[1..]),
        "move" => distro_move(&args[1..]),
        "clone" => distro_clone(&args[1..]),
        other => err(format!("distro list|sync|remove|move|clone, not {other}")),
    }
}

fn distro_move(args: &[String]) -> i32 {
    let pos = positional(args);
    let Some(name) = pos.first() else {
        return err("distro move <name> <dir>");
    };
    let dir = take_named(args, "--location")
        .into_iter()
        .next()
        .or_else(|| pos.get(1).map(|s| (*s).to_string()));
    let Some(dir) = dir else {
        return err("distro move <name> <dir>");
    };
    match crate::new_wsl::move_distro(name, &dir) {
        Ok(s) => match crate::terminal::sync(None) {
            Ok(r) => print_json(&serde_json::json!({"ok": true, "detail": s, "terminal": r})),
            Err(e) => err(format!("{s}; terminal sync failed: {e}")),
        },
        Err(e) => err(e),
    }
}

fn distro_clone(args: &[String]) -> i32 {
    let pos = positional(args);
    let Some(src) = pos.first() else {
        return err("distro clone <src> <new> [--location D:\\WSL\\name]");
    };
    let Some(new_name) = pos.get(1) else {
        return err("distro clone <src> <new> [--location D:\\WSL\\name]");
    };
    let location = take_named(args, "--location").into_iter().next();
    match crate::new_wsl::clone_distro(src, new_name, location.as_deref()) {
        Ok(s) => match crate::terminal::sync(None) {
            Ok(r) => print_json(&serde_json::json!({"ok": true, "detail": s, "terminal": r})),
            Err(e) => err(format!("{s}; terminal sync failed: {e}")),
        },
        Err(e) => err(e),
    }
}

fn distro_remove(args: &[String]) -> i32 {
    let pos = positional(args);
    let Some(name) = pos.first() else {
        return err("distro remove <name> --yes");
    };
    if !flag(args, "--yes") {
        return err(format!(
            "pass --yes to unregister {name} (deletes that Linux disk) and drop its Terminal profile"
        ));
    }
    match crate::new_wsl::unregister_distro(name) {
        Ok(unreg) => match crate::terminal::sync(None) {
            Ok(r) => print_json(&serde_json::json!({
                "ok": true,
                "unregistered": unreg,
                "terminal": r,
            })),
            Err(e) => err(format!("{unreg}; terminal sync failed: {e}")),
        },
        Err(e) => err(e),
    }
}

fn distros_cmd() -> i32 {
    let choices = crate::new_wsl::distro_choices();
    print_json(&serde_json::json!({
        "supported": choices.iter().map(|d| serde_json::json!({
            "id": d.id,
            "label": d.label,
            "family": d.family,
            "bootstrap": d.bootstrap,
        })).collect::<Vec<_>>(),
        "online": crate::new_wsl::online_names(),
        "installed": crate::new_wsl::distro_names(),
        "choices": choices,
        "default": crate::new_wsl::default_distro_name(),
    }))
}

#[cfg(test)]
mod tests {
    #[test]
    fn opencli_parses() {
        let v: serde_json::Value =
            serde_json::from_str(include_str!("../../../schema/wwm.opencli.json")).unwrap();
        assert_eq!(v["opencli"], "0.1");
        assert_eq!(v["command"]["name"], "wwm");
        let cmds = v["command"]["commands"].as_array().unwrap();
        assert!(cmds.iter().any(|c| c["name"] == "new-wsl"));
        assert!(cmds.iter().any(|c| c["name"] == "spec"));
        let new_wsl = cmds.iter().find(|c| c["name"] == "new-wsl").unwrap();
        let opts = new_wsl["options"].as_array().unwrap();
        let profile = opts.iter().find(|o| o["name"] == "--profile").unwrap();
        let vals = profile["arguments"][0]["acceptedValues"]
            .as_array()
            .unwrap();
        assert!(vals.iter().any(|v| v == "blank"));
    }
}
