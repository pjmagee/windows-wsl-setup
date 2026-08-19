use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::prelude::*;
use ratatui::widgets::{Block, List, ListItem, Paragraph};

use crate::catalog::Store;
use crate::new_wsl;

struct App {
    profiles: Vec<String>,
    profile: usize,
    distros: Vec<String>,
    distro: usize,
    store: Store,
    log: Vec<String>,
    status: String,
}

impl App {
    fn new() -> Self {
        let store = Store::load().unwrap_or_else(|_| Store::shipped().expect("shipped"));
        let mut profiles: Vec<String> = store.linux.keys().cloned().collect();
        if profiles.is_empty() {
            profiles = vec!["home".into(), "work".into()];
        }
        let distros: Vec<String> = new_wsl::SUPPORTED.iter().map(|s| (*s).to_string()).collect();
        let have = distros
            .iter()
            .map(|d| {
                if new_wsl::has_named(d) {
                    format!("{d} is installed")
                } else {
                    format!("{d} is not installed yet")
                }
            })
            .collect::<Vec<_>>()
            .join(" · ");
        Self {
            profiles,
            profile: 0,
            distros,
            distro: 0,
            store,
            log: vec![have],
            status: "Enter = create the selected distro and install tools. You never clone a repo."
                .into(),
        }
    }

    fn profile_id(&self) -> &str {
        self.profiles
            .get(self.profile)
            .map(|s| s.as_str())
            .unwrap_or("home")
    }

    fn distro_id(&self) -> &str {
        self.distros
            .get(self.distro)
            .map(|s| s.as_str())
            .unwrap_or(new_wsl::DISTRO)
    }

    fn push(&mut self, line: impl Into<String>) {
        self.log.push(line.into());
        if self.log.len() > 80 {
            self.log.remove(0);
        }
    }

    fn step(
        &mut self,
        term: &mut ratatui::DefaultTerminal,
        status: &str,
        work: impl FnOnce() -> Result<String, String>,
    ) -> bool {
        self.status = status.into();
        let _ = term.draw(|f| draw(f, self));
        match work() {
            Ok(s) => {
                if s.trim().is_empty() {
                    self.push(status);
                } else {
                    for line in s.lines() {
                        let t = line.trim();
                        if !t.is_empty() {
                            self.push(t);
                        }
                    }
                }
                true
            }
            Err(e) => {
                for line in e.lines() {
                    let t = line.trim();
                    if !t.is_empty() {
                        self.push(t);
                    }
                }
                self.status = "stopped".into();
                false
            }
        }
    }

    fn run_setup(&mut self, term: &mut ratatui::DefaultTerminal) {
        let profile = self.profile_id().to_string();
        let distro = self.distro_id().to_string();
        self.push(new_wsl::ensure_wslconfig());
        if !self.step(term, "checking WSL…", new_wsl::ensure_wsl) {
            return;
        }
        if !self.step(term, "installing the selected distro if needed…", || {
            new_wsl::install_distro(&distro)
        }) {
            return;
        }
        if !self.step(term, "waiting for first boot…", || {
            new_wsl::wait_for_root(&distro)
        }) {
            return;
        }
        if !self.step(term, "creating the Linux user…", || {
            new_wsl::create_user_if_needed(&distro)
        }) {
            return;
        }
        if !self.step(term, "sudo for that user…", || {
            new_wsl::ensure_passwordless_sudo(&distro)
        }) {
            return;
        }
        if !self.step(term, "default WSL distro…", || {
            new_wsl::set_default_distro(&distro)
        }) {
            return;
        }
        self.status = format!("installing {profile} tools inside {distro} (several minutes)…");
        let _ = term.draw(|f| draw(f, self));
        let json = self
            .store
            .linux
            .get(&profile)
            .and_then(|p| serde_json::to_string(p).ok());
        match new_wsl::install_toolchain(&distro, &profile, json.as_deref()) {
            Ok(s) => {
                for line in s
                    .lines()
                    .rev()
                    .take(20)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                {
                    self.push(line);
                }
                self.status = format!("done. Open a new {distro} tab.");
            }
            Err(e) => {
                for line in e
                    .lines()
                    .rev()
                    .take(20)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                {
                    self.push(line);
                }
                self.status = "install.sh reported errors — scroll the log".into();
            }
        }
    }
}

pub fn run() -> Result<(), String> {
    let mut app = App::new();
    let mut term = ratatui::init();
    let result = loop {
        term.draw(|f| draw(f, &app)).map_err(|e| e.to_string())?;
        let Event::Key(key) = event::read().map_err(|e| e.to_string())? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => break Ok(()),
            KeyCode::Left | KeyCode::Char('h') => {
                if !app.profiles.is_empty() {
                    app.profile = (app.profile + app.profiles.len() - 1) % app.profiles.len();
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if !app.profiles.is_empty() {
                    app.profile = (app.profile + 1) % app.profiles.len();
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if !app.distros.is_empty() {
                    app.distro = (app.distro + app.distros.len() - 1) % app.distros.len();
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !app.distros.is_empty() {
                    app.distro = (app.distro + 1) % app.distros.len();
                }
            }
            KeyCode::Enter => app.run_setup(&mut term),
            _ => {}
        }
    };
    ratatui::restore();
    result
}

fn mint() -> Color {
    Color::Rgb(125, 206, 160)
}

fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Length(6),
        Constraint::Min(6),
        Constraint::Length(2),
    ])
    .split(f.area());

    let names: String = app
        .profiles
        .iter()
        .enumerate()
        .map(|(i, p)| {
            if i == app.profile {
                format!("(*) {p}")
            } else {
                format!("( ) {p}")
            }
        })
        .collect::<Vec<_>>()
        .join("   ");
    let distros: String = app
        .distros
        .iter()
        .enumerate()
        .map(|(i, p)| {
            if i == app.distro {
                format!("(*) {p}")
            } else {
                format!("( ) {p}")
            }
        })
        .collect::<Vec<_>>()
        .join("   ");
    let head = format!(
        "New WSL = a Linux distro + a software profile (Homebrew CLIs).\n\
         {distros}     j/k distro\n\
         {names}     ← → profile"
    );
    f.render_widget(
        Paragraph::new(head).block(
            Block::bordered()
                .title(" Windows WSL Setup  new WSL ")
                .border_style(Style::new().fg(Color::DarkGray)),
        ),
        chunks[0],
    );

    let items: Vec<ListItem> = app.log.iter().map(|l| ListItem::new(l.as_str())).collect();
    f.render_widget(
        List::new(items).block(Block::bordered().title(" log ")),
        chunks[1],
    );
    f.render_widget(
        Paragraph::new(format!("{}   Enter run   q quit", app.status))
            .style(Style::new().fg(mint())),
        chunks[2],
    );
}
