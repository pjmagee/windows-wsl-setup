mod apply;
mod backup;
mod catalog;
mod classify;
mod cli;
mod inventory;
mod kit;
mod model;
mod new_wsl;
mod profile;
mod restore;
mod suggest;
mod tui;
mod tui_new_wsl;
mod tui_profiles;
mod tui_restore;
mod winget_search;

fn main() {
    #[cfg(not(windows))]
    {
        eprintln!("Windows WSL Setup is a Windows console app.");
        std::process::exit(1);
    }

    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = if args.is_empty() {
        "tui".into()
    } else {
        args.remove(0)
    };
    let code = match cmd.as_str() {
        "inventory" => match inventory::collect() {
            Ok(inv) => {
                println!("{}", serde_json::to_string_pretty(&inv).unwrap_or_default());
                0
            }
            Err(e) => {
                eprintln!("{e}");
                1
            }
        },
        "collect" | "capture" => match tui::run_collect() {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("{e}");
                1
            }
        },
        "restore" => {
            let kit = args.first().map(std::path::PathBuf::from);
            match tui_restore::run(kit) {
                Ok(()) => 0,
                Err(e) => {
                    eprintln!("{e}");
                    1
                }
            }
        }
        "new-wsl" | "new_wsl" => {
            let profile = named(&args, "--profile");
            if let Some(p) = profile {
                match crate::apply::apply_id(
                    &match crate::catalog::Store::load() {
                        Ok(s) => s,
                        Err(e) => {
                            eprintln!("{e}");
                            std::process::exit(1);
                        }
                    },
                    &p,
                    false,
                    true,
                ) {
                    Ok(steps) => {
                        println!("{}", serde_json::to_string_pretty(&steps).unwrap_or_default());
                        0
                    }
                    Err(e) => {
                        eprintln!("{e}");
                        1
                    }
                }
            } else {
                match tui_new_wsl::run() {
                    Ok(()) => 0,
                    Err(e) => {
                        eprintln!("{e}");
                        1
                    }
                }
            }
        }
        "tui" => match tui::run() {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("{e}");
                1
            }
        },
        "catalog" | "profile" | "profiles" | "search" | "map" | "suggest" | "apply" => {
            cli::dispatch(&cmd, &args)
        }
        "-h" | "--help" | "help" => {
            eprintln!(
                "Windows WSL Setup — one Windows binary. No git clone.\n\n\
  windows-wsl-setup                 Collect / Restore / New WSL / Profiles\n\
  windows-wsl-setup collect         scan this PC, write a kit\n\
  windows-wsl-setup restore         install from a kit (optional path)\n\
  windows-wsl-setup new-wsl         empty Ubuntu 26.04 + linux profile\n\
  windows-wsl-setup new-wsl --profile home\n\
  windows-wsl-setup profiles        edit / suggest / apply a profile\n\n\
  JSON (agents):\n\
  windows-wsl-setup catalog linux|windows\n\
  windows-wsl-setup profile list|show <id>|new <id> --from home\n\
  windows-wsl-setup profile add <id> --linux kubectl --windows Brave.Brave\n\
  windows-wsl-setup search linux <q> | search winget <q>\n\
  windows-wsl-setup map <winget-id>\n\
  windows-wsl-setup suggest\n\
  windows-wsl-setup apply <id> [--windows-only|--linux-only]\n"
            );
            0
        }
        other => {
            eprintln!("unknown command {other}");
            2
        }
    };
    std::process::exit(code);
}

fn named(args: &[String], flag: &str) -> Option<String> {
    let mut i = 0;
    while i + 1 < args.len() {
        if args[i] == flag {
            return Some(args[i + 1].clone());
        }
        i += 1;
    }
    None
}
