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
        eprintln!("Windows WSL Setup is a Windows console app.");
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
                "Windows WSL Setup — one Windows binary. No git clone. No scripts.\n\n  windows-wsl-setup              Collect or Restore\n  windows-wsl-setup collect      scan this PC, write a kit\n  windows-wsl-setup restore      install from a kit (optional path)\n  windows-wsl-setup inventory    print scan JSON\n"
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
