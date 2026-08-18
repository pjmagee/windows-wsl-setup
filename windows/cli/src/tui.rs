use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Tabs};

use crate::backup::{write_kit, Selection};
use crate::inventory;
use crate::model::{LinuxProfile, LinuxTool};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Dest,
    Profile,
    Linux,
    Wsl,
    Host,
    Apps,
    Write,
}

const TABS: [Tab; 7] = [
    Tab::Dest,
    Tab::Profile,
    Tab::Linux,
    Tab::Wsl,
    Tab::Host,
    Tab::Apps,
    Tab::Write,
];

const TAB_TITLES: [&str; 7] = [
    "Dest",
    "Profile",
    "Linux",
    "WSL",
    "Host",
    "Apps",
    "Write",
];

struct App {
    tab: usize,
    cursor: usize,
    dest: usize,
    profile: LinuxProfile,
    extras: Vec<LinuxTool>,
    extra_home: Vec<bool>,
    extra_work: Vec<bool>,
    wsl_keep: Vec<bool>,
    app_keep: Vec<bool>,
    app_filter: String,
    filtering: bool,
    dev_drive: bool,
    docker: bool,
    browser: bool,
    dotfiles: bool,
    status: String,
    inv: crate::model::Inventory,
}

impl App {
    fn new(inv: crate::model::Inventory) -> Self {
        let extras: Vec<LinuxTool> = inv
            .linux_tools
            .tools
            .iter()
            .filter(|t| t.layer != "base")
            .cloned()
            .collect();
        let extra_home: Vec<bool> = extras.iter().map(|t| t.home).collect();
        let extra_work: Vec<bool> = extras.iter().map(|t| t.work).collect();
        let wsl_keep: Vec<bool> = inv.wsl.iter().map(|d| d.keep).collect();
        let app_keep: Vec<bool> = inv.apps.iter().map(|a| a.keep).collect();
        let dest = 0;
        let status = if inv.destinations.is_empty() {
            "No non-C: data drive. Assign a letter, then restart.".into()
        } else {
            format!("scan ok — {} apps, {} dest", inv.apps.len(), inv.destinations.len())
        };
        Self {
            tab: 0,
            cursor: 0,
            dest,
            profile: LinuxProfile::Home,
            extras,
            extra_home,
            extra_work,
            wsl_keep,
            app_keep,
            app_filter: String::new(),
            filtering: false,
            dev_drive: inv.dev_drive.keep,
            docker: inv.docker.keep,
            browser: inv.brave.keep,
            dotfiles: true,
            status,
            inv,
        }
    }

    fn tab_enum(&self) -> Tab {
        TABS[self.tab]
    }

    fn kit_root(&self) -> Option<String> {
        self.inv.destinations.get(self.dest).map(|d| d.suggested.clone())
    }

    fn visible_apps(&self) -> Vec<usize> {
        let q = self.app_filter.to_ascii_lowercase();
        self.inv
            .apps
            .iter()
            .enumerate()
            .filter(|(_, a)| {
                q.is_empty()
                    || a.id.to_ascii_lowercase().contains(&q)
                    || a.name.to_ascii_lowercase().contains(&q)
            })
            .map(|(i, _)| i)
            .collect()
    }

    fn clamp(&mut self) {
        let n = match self.tab_enum() {
            Tab::Dest => self.inv.destinations.len().max(1),
            Tab::Profile => 3,
            Tab::Linux => self.extras.len().max(1),
            Tab::Wsl => self.inv.wsl.len().max(1),
            Tab::Host => 4,
            Tab::Apps => self.visible_apps().len().max(1),
            Tab::Write => 1,
        };
        if self.cursor >= n {
            self.cursor = n - 1;
        }
    }

    fn toggle(&mut self) {
        match self.tab_enum() {
            Tab::Dest => self.dest = self.cursor,
            Tab::Profile => {
                self.profile = match self.cursor {
                    0 => LinuxProfile::Home,
                    1 => LinuxProfile::Work,
                    _ => LinuxProfile::Skip,
                };
            }
            Tab::Linux => {
                if let Some(v) = self.extra_home.get_mut(self.cursor) {
                    *v = !*v;
                }
            }
            Tab::Wsl => {
                if let Some(v) = self.wsl_keep.get_mut(self.cursor) {
                    *v = !*v;
                }
            }
            Tab::Host => match self.cursor {
                0 => self.dev_drive = !self.dev_drive,
                1 => self.docker = !self.docker,
                2 => self.browser = !self.browser,
                3 => self.dotfiles = !self.dotfiles,
                _ => {}
            },
            Tab::Apps => {
                let vis = self.visible_apps();
                if let Some(&i) = vis.get(self.cursor) {
                    if let Some(v) = self.app_keep.get_mut(i) {
                        *v = !*v;
                    }
                }
            }
            Tab::Write => self.write_now(),
        }
    }

    fn toggle_work(&mut self) {
        if self.tab_enum() == Tab::Linux {
            if let Some(v) = self.extra_work.get_mut(self.cursor) {
                *v = !*v;
            }
        }
    }

    fn write_now(&mut self) {
        let Some(kit) = self.kit_root() else {
            self.status = "pick a destination first".into();
            return;
        };
        if self.profile == LinuxProfile::Skip {
            // still write the Windows kit
        }
        let sel = Selection {
            kit_root: kit.clone(),
            profile: self.profile,
            inv: &self.inv,
            dest_idx: self.dest, // kit dest index
            app_keep: &self.app_keep,
            wsl_keep: &self.wsl_keep,
            extra_home: &self.extra_home,
            extra_work: &self.extra_work,
            extras: &self.extras,
            dev_drive: self.dev_drive,
            docker: self.docker,
            browser: self.browser,
            dotfiles: self.dotfiles,
        };
        match write_kit(&sel) {
            Ok(p) => self.status = format!("wrote {p}"),
            Err(e) => self.status = format!("error: {e}"),
        }
    }
}

pub fn run() -> Result<(), String> {
    let inv = inventory::collect()?;
    let mut app = App::new(inv);

    let mut term = ratatui::init();
    let result = loop {
        app.clamp();
        term.draw(|f| draw(f, &app)).map_err(|e| e.to_string())?;
        let ev = event::read().map_err(|e| e.to_string())?;
        let Event::Key(key) = ev else { continue };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        if app.filtering {
            match key.code {
                KeyCode::Esc | KeyCode::Enter => app.filtering = false,
                KeyCode::Backspace => {
                    app.app_filter.pop();
                }
                KeyCode::Char(c) => app.app_filter.push(c),
                _ => {}
            }
            continue;
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => break Ok(()),
            KeyCode::Tab => {
                app.tab = (app.tab + 1) % TABS.len();
                app.cursor = 0;
            }
            KeyCode::BackTab => {
                app.tab = (app.tab + TABS.len() - 1) % TABS.len();
                app.cursor = 0;
            }
            KeyCode::Down | KeyCode::Char('j') => app.cursor = app.cursor.saturating_add(1),
            KeyCode::Up | KeyCode::Char('k') => app.cursor = app.cursor.saturating_sub(1),
            KeyCode::Char(' ') | KeyCode::Enter => app.toggle(),
            KeyCode::Char('w') => app.toggle_work(),
            KeyCode::Char('/') if app.tab_enum() == Tab::Apps => {
                app.filtering = true;
                app.app_filter.clear();
            }
            KeyCode::Char('W') if app.tab_enum() == Tab::Write => app.write_now(),
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
        Constraint::Length(3),
        Constraint::Min(6),
        Constraint::Length(2),
        Constraint::Length(1),
    ])
    .split(f.area());

    let tabs = Tabs::new(TAB_TITLES)
        .select(app.tab)
        .highlight_style(Style::new().fg(mint()).bold())
        .block(
            Block::bordered()
                .title(" windows-wsl-setup  capture ")
                .border_style(Style::new().fg(Color::DarkGray)),
        );
    f.render_widget(tabs, chunks[0]);

    match app.tab_enum() {
        Tab::Dest => f.render_widget(dest_body(app), chunks[1]),
        Tab::Profile => f.render_widget(profile_body(app), chunks[1]),
        Tab::Linux => f.render_widget(linux_body(app), chunks[1]),
        Tab::Wsl => f.render_widget(wsl_body(app), chunks[1]),
        Tab::Host => f.render_widget(host_body(app), chunks[1]),
        Tab::Apps => f.render_widget(apps_body(app), chunks[1]),
        Tab::Write => f.render_widget(write_body(app), chunks[1]),
    }

    let help = match app.tab_enum() {
        Tab::Linux => "j/k move  space home  w work  tab section  q quit",
        Tab::Apps => "j/k move  space keep  / filter  tab section  q quit",
        Tab::Write => "Enter or W write kit  tab section  q quit",
        _ => "j/k move  space select  tab section  q quit",
    };
    f.render_widget(
        Paragraph::new(help).style(Style::new().fg(Color::DarkGray)),
        chunks[2],
    );
    f.render_widget(
        Paragraph::new(app.status.as_str()).style(Style::new().fg(mint())),
        chunks[3],
    );
}

fn dest_body(app: &App) -> List<'_> {
    let items: Vec<ListItem> = app
        .inv
        .destinations
        .iter()
        .enumerate()
        .map(|(i, d)| {
            let mark = if i == app.dest { "(*)" } else { "( )" };
            ListItem::new(format!(
                "{mark}  {l}: {lab}   {free:.0} GB free / {gb} GB",
                l = d.letter,
                lab = if d.label.is_empty() { "volume" } else { &d.label },
                free = d.free_gb,
                gb = d.gb
            ))
        })
        .collect();
    list(" kit destination — not C:, not the Dev Drive ", items, app.cursor)
}

fn profile_body(app: &App) -> List<'_> {
    let opts = [
        (LinuxProfile::Home, "home   extras ticked home (Grok / Claude by default)"),
        (LinuxProfile::Work, "work   extras ticked work (Copilot by default); drops home-only"),
        (LinuxProfile::Skip, "skip   do not run install.sh after restore"),
    ];
    let items: Vec<ListItem> = opts
        .iter()
        .map(|(p, label)| {
            let mark = if app.profile == *p { "(*)" } else { "( )" };
            ListItem::new(format!("{mark}  {label}"))
        })
        .collect();
    list(" linux profile for this machine ", items, app.cursor)
}

fn linux_body(app: &App) -> List<'_> {
    let items: Vec<ListItem> = app
        .extras
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let h = if app.extra_home.get(i).copied().unwrap_or(false) {
                "[x]"
            } else {
                "[ ]"
            };
            let w = if app.extra_work.get(i).copied().unwrap_or(false) {
                "[x]"
            } else {
                "[ ]"
            };
            ListItem::new(format!("{h} home  {w} work   {}", t.name))
        })
        .collect();
    list(
        " extras — space toggles home, w toggles work. base tools always install ",
        items,
        app.cursor,
    )
}

fn wsl_body(app: &App) -> List<'_> {
    let items: Vec<ListItem> = app
        .inv
        .wsl
        .iter()
        .enumerate()
        .map(|(i, d)| {
            let mark = if app.wsl_keep.get(i).copied().unwrap_or(false) {
                "[x]"
            } else {
                "[ ]"
            };
            ListItem::new(format!(
                "{mark}  {}   WSL{}  {:.1} GB  {}",
                d.name, d.version, d.gb, d.kind
            ))
        })
        .collect();
    list(" WSL distros — never unregisters ", items, app.cursor)
}

fn host_body(app: &App) -> List<'_> {
    let dd = if app.inv.dev_drive.present {
        format!(
            "Dev Drive {} ({:.0} GB VHDX)",
            app.inv.dev_drive.letter.map(|c| format!("{c}:")).unwrap_or_default(),
            app.inv.dev_drive.vhdx.first().map(|v| v.gb).unwrap_or(0.0)
        )
    } else {
        "Dev Drive (not found)".into()
    };
    let rows = [
        (app.dev_drive, dd),
        (
            app.docker,
            format!("Docker data VHDX  {:.2} GB", app.inv.docker.gb),
        ),
        (
            app.browser,
            format!(
                "Brave bookmarks + {} extensions",
                app.inv.brave.extensions.len()
            ),
        ),
        (app.dotfiles, "Host dotfiles (git/ssh/terminal/grok)".into()),
    ];
    let items: Vec<ListItem> = rows
        .into_iter()
        .map(|(on, label)| {
            let mark = if on { "[x]" } else { "[ ]" };
            ListItem::new(format!("{mark}  {label}"))
        })
        .collect();
    list(" host leftovers ", items, app.cursor)
}

fn apps_body(app: &App) -> List<'_> {
    let vis = app.visible_apps();
    let items: Vec<ListItem> = vis
        .iter()
        .map(|&i| {
            let a = &app.inv.apps[i];
            let mark = if app.app_keep[i] { "[x]" } else { "[ ]" };
            ListItem::new(format!("{mark}  {}  {}", a.id, a.version))
        })
        .collect();
    let title = if app.filtering {
        format!(" winget  filter: {}█ ", app.app_filter)
    } else {
        format!(" winget  {} kept  / to filter ", app.app_keep.iter().filter(|k| **k).count())
    };
    list(&title, items, app.cursor)
}

fn write_body(app: &App) -> Paragraph<'_> {
    let dest = app
        .kit_root()
        .unwrap_or_else(|| "(no destination)".into());
    let text = format!(
        "kit:     {dest}\nprofile: {}\napps:    {} kept\nwsl:     {}\n\nEnter / W  write small files (no VHDX copy yet)",
        app.profile.as_str(),
        app.app_keep.iter().filter(|k| **k).count(),
        app.inv
            .wsl
            .iter()
            .zip(app.wsl_keep.iter())
            .filter(|(_, k)| **k)
            .map(|(d, _)| d.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
    Paragraph::new(text).block(
        Block::bordered()
            .title(" write kit ")
            .border_style(Style::new().fg(Color::DarkGray)),
    )
}

fn list<'a>(title: impl Into<String>, items: Vec<ListItem<'a>>, cursor: usize) -> List<'a> {
    let items: Vec<ListItem<'a>> = items
        .into_iter()
        .enumerate()
        .map(|(i, it)| {
            if i == cursor {
                it.style(Style::new().bg(Color::Rgb(28, 38, 48)).fg(mint()))
            } else {
                it
            }
        })
        .collect();
    List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title.into())
            .border_style(Style::new().fg(Color::DarkGray)),
    )
}
