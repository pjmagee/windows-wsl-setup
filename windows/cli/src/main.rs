mod backup;
mod inventory;
mod kit;
mod model;
mod restore;
mod tui;
mod tui_restore;

fn main() {
    #[cfg(not(windows))]
    {
        eprintln!("wsl-setup capture is a Windows console app.");
        std::process::exit(1);
    }

    let mut args = std::env::args().skip(1);
    let cmd = args.next().unwrap_or_else(|| "tui".into());
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
            let kit = args.next().map(std::path::PathBuf::from);
            match tui_restore::run(kit) {
                Ok(()) => 0,
                Err(e) => {
                    eprintln!("{e}");
                    1
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
        "-h" | "--help" | "help" => {
            eprintln!(
                "wsl-setup — single Windows binary. No git clone. No scripts.\n\n  wsl-setup              Collect or Restore\n  wsl-setup collect      scan this PC, write a kit\n  wsl-setup restore      install from a kit (optional path)\n  wsl-setup inventory    print scan JSON\n"
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
