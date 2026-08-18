mod backup;
mod inventory;
mod model;
mod tui;

fn main() {
    #[cfg(not(windows))]
    {
        eprintln!("wsl-setup capture is a Windows console app.");
        std::process::exit(1);
    }

    let mut args = std::env::args().skip(1);
    let cmd = args.next().unwrap_or_else(|| "capture".into());
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
        "capture" | "tui" => match tui::run() {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("{e}");
                1
            }
        },
        "-h" | "--help" | "help" => {
            eprintln!(
                "wsl-setup — native Windows capture TUI\n\n  wsl-setup capture     interactive (default)\n  wsl-setup inventory   print scan JSON\n"
            );
            0
        }
        other => {
            eprintln!("unknown command {other}. try capture or inventory");
            2
        }
    };
    std::process::exit(code);
}
