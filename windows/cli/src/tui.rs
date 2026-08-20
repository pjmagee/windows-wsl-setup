use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Tabs};

use crate::backup::{write_kit, Selection};
use crate::catalog::Store;
use crate::classify;
use crate::inventory;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Dest,
    Apps,
    Wsl,
    Host,
    Write,
}

const TABS: [Tab; 5] = [Tab::Dest, Tab::Apps, Tab::Wsl, Tab::Host, Tab::Write];

const TAB_TITLES: [&str; 5] = ["Dest", "Apps", "WSL", "Host", "Write"];

enum AppRow {
    Header(String),
    App(usize),
}

struct App {
    tab: usize,
    cursor: usize,
    dest: usize,
    profile: String,
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
    store: Store,
}

impl App {
    fn new(inv: crate::model::Inventory) -> Self {
        let wsl_keep: Vec<bool> = inv.wsl.iter().map(|d| d.keep).collect();
        let app_keep: Vec<bool> = inv.apps.iter().map(|a| a.keep).collect();
        let dest = 0;
        let status = if inv.destinations.is_empty() {
            "No non-C: data drive. Assign a letter, then restart.".into()
        } else {
            format!(
                "scan ok — {} apps, {} dest",
                inv.apps.len(),
                inv.destinations.len()
            )
        };
        Self {
            tab: 0,
            cursor: 0,
            dest,
            profile: "home".into(),
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
            store: Store::load().unwrap_or_else(|_| Store::shipped().expect("shipped catalogs")),
        }
    }

    fn tab_enum(&self) -> Tab {
        TABS[self.tab]
    }

    fn kit_root(&self) -> Option<String> {
        self.inv
            .destinations
            .get(self.dest)
            .map(|d| d.suggested.clone())
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

    fn app_rows(&self) -> Vec<AppRow> {
        let vis = self.visible_apps();
        let mut groups: Vec<(String, Vec<usize>)> = classify::WINDOWS_CATEGORIES
            .iter()
            .map(|c| ((*c).to_string(), Vec::new()))
            .collect();
        for i in vis {
            let cat = self.store.windows_category(&self.inv.apps[i].id);
            if let Some((_, ids)) = groups.iter_mut().find(|(c, _)| c == &cat) {
                ids.push(i);
            }
        }
        let mut rows = Vec::new();
        for (cat, ids) in groups {
            if ids.is_empty() {
                continue;
            }
            rows.push(AppRow::Header(cat));
            for i in ids {
                rows.push(AppRow::App(i));
            }
        }
        rows
    }

    fn clamp(&mut self) {
        let n = match self.tab_enum() {
            Tab::Dest => self.inv.destinations.len().max(1),
            Tab::Apps => self.app_rows().len().max(1),
            Tab::Wsl => self.inv.wsl.len().max(1),
            Tab::Host => 4,
            Tab::Write => 1,
        };
        if self.cursor >= n {
            self.cursor = n - 1;
        }
    }

    fn toggle(&mut self) {
        match self.tab_enum() {
            Tab::Dest => self.dest = self.cursor,
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
                let rows = self.app_rows();
                match rows.get(self.cursor) {
                    Some(AppRow::App(i)) => {
                        if let Some(v) = self.app_keep.get_mut(*i) {
                            *v = !*v;
                        }
                    }
                    Some(AppRow::Header(_)) => {
                        let mut idxs = Vec::new();
                        for row in rows.iter().skip(self.cursor + 1) {
                            match row {
                                AppRow::Header(_) => break,
                                AppRow::App(i) => idxs.push(*i),
                            }
                        }
                        let all_on = idxs
                            .iter()
                            .all(|&i| self.app_keep.get(i).copied().unwrap_or(false));
                        for i in idxs {
                            if let Some(v) = self.app_keep.get_mut(i) {
                                *v = !all_on;
                            }
                        }
                    }
                    None => {}
                }
            }
            Tab::Write => self.write_now(),
        }
    }

    fn write_now(&mut self) {
        let Some(kit) = self.kit_root() else {
            self.status = "pick a destination first".into();
            return;
        };
        let sel = Selection {
            kit_root: kit.clone(),
            profile: self.profile.clone(),
            inv: &self.inv,
            dest_idx: self.dest,
            app_keep: &self.app_keep,
            wsl_keep: &self.wsl_keep,
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
    match home_choice()? {
        HomeChoice::Collect => run_collect(),
        HomeChoice::Restore => crate::tui_restore::run(None),
        HomeChoice::NewWsl => crate::tui_new_wsl::run(None),
        HomeChoice::Profiles => crate::tui_profiles::run(),
        HomeChoice::Quit => Ok(()),
    }
}

enum HomeChoice {
    Collect,
    Restore,
    NewWsl,
    Profiles,
    Quit,
}

const HOME_ITEMS: [&str; 4] = [
    "Collect    snapshot this PC (apps, Linux disks, data volumes) onto a data drive",
    "Restore    read a kit, install apps, remount disks, restore browser bookmarks",
    "New WSL    Ubuntu, Debian, or Arch + a linux profile",
    "Profiles   named software lists — suggest, save, apply (no kit)",
];

fn home_choice() -> Result<HomeChoice, String> {
    let mut cursor = 0usize;
    let mut term = ratatui::init();
    let result = loop {
        term.draw(|f| {
            let list_items: Vec<ListItem> = HOME_ITEMS
                .iter()
                .enumerate()
                .map(|(i, t)| {
                    let mark = if i == cursor { "(*)" } else { "( )" };
                    let it = ListItem::new(format!("{mark}  {t}"));
                    if i == cursor {
                        it.style(Style::new().fg(mint()).bg(Color::Rgb(28, 38, 48)))
                    } else {
                        it
                    }
                })
                .collect();
            let w = List::new(list_items).block(
                Block::bordered()
                    .title(" Windows WSL Setup ")
                    .border_style(Style::new().fg(Color::DarkGray)),
            );
            let chunks =
                Layout::vertical([Constraint::Min(8), Constraint::Length(1)]).split(f.area());
            f.render_widget(w, chunks[0]);
            f.render_widget(
                Paragraph::new("j/k move   Enter choose   q quit")
                    .style(Style::new().fg(Color::DarkGray)),
                chunks[1],
            );
        })
        .map_err(|e| e.to_string())?;
        let Event::Key(key) = event::read().map_err(|e| e.to_string())? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => break Ok(HomeChoice::Quit),
            KeyCode::Down | KeyCode::Char('j') => {
                cursor = (cursor + 1) % HOME_ITEMS.len();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                cursor = (cursor + HOME_ITEMS.len() - 1) % HOME_ITEMS.len();
            }
            KeyCode::Char('1') => cursor = 0,
            KeyCode::Char('2') => cursor = 1,
            KeyCode::Char('3') => cursor = 2,
            KeyCode::Char('4') => cursor = 3,
            KeyCode::Enter | KeyCode::Char(' ') => {
                break Ok(match cursor {
                    0 => HomeChoice::Collect,
                    1 => HomeChoice::Restore,
                    2 => HomeChoice::NewWsl,
                    _ => HomeChoice::Profiles,
                });
            }
            _ => {}
        }
    };
    ratatui::restore();
    result
}

pub fn run_collect() -> Result<(), String> {
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
                .title(" Windows WSL Setup  collect ")
                .border_style(Style::new().fg(Color::DarkGray)),
        );
    f.render_widget(tabs, chunks[0]);

    match app.tab_enum() {
        Tab::Dest => f.render_widget(dest_body(app), chunks[1]),
        Tab::Apps => f.render_widget(apps_body(app), chunks[1]),
        Tab::Wsl => f.render_widget(wsl_body(app), chunks[1]),
        Tab::Host => f.render_widget(host_body(app), chunks[1]),
        Tab::Write => f.render_widget(write_body(app), chunks[1]),
    }

    let help = match app.tab_enum() {
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
                lab = if d.label.is_empty() {
                    "volume"
                } else {
                    &d.label
                },
                free = d.free_gb,
                gb = d.gb
            ))
        })
        .collect();
    list(
        " kit destination — not C:, not the Dev Drive ",
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
            app.inv
                .dev_drive
                .letter
                .map(|c| format!("{c}:"))
                .unwrap_or_default(),
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
    let rows = app.app_rows();
    let items: Vec<ListItem> = rows
        .iter()
        .map(|row| match row {
            AppRow::Header(cat) => {
                ListItem::new(format!("── {cat} ──")).style(Style::new().fg(Color::DarkGray))
            }
            AppRow::App(i) => {
                let a = &app.inv.apps[*i];
                let mark = if app.app_keep[*i] { "[x]" } else { "[ ]" };
                ListItem::new(format!("{mark}  {}  {}", a.id, a.version))
            }
        })
        .collect();
    let title = if app.filtering {
        format!(" winget  filter: {}█ ", app.app_filter)
    } else {
        format!(
            " winget  {} kept  / to filter ",
            app.app_keep.iter().filter(|k| **k).count()
        )
    };
    list(&title, items, app.cursor)
}

fn write_body(app: &App) -> Paragraph<'_> {
    let dest = app.kit_root().unwrap_or_else(|| "(no destination)".into());
    let text = format!(
        "kit:  {dest}\napps: {} kept\nwsl:  {}\n\nEnter / W  write the kit. Then take this folder to the new PC.",
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
