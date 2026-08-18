use std::path::PathBuf;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::prelude::*;
use ratatui::widgets::{Block, List, ListItem, Paragraph, Tabs};

use crate::kit::{self, LoadedKit};
use crate::restore;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Kit,
    Apps,
    Disks,
    Browser,
    Apply,
}

const TABS: [Tab; 5] = [Tab::Kit, Tab::Apps, Tab::Disks, Tab::Browser, Tab::Apply];
const TITLES: [&str; 5] = ["Kit", "Apps", "Disks", "Browser", "Apply"];

struct App {
    tab: usize,
    cursor: usize,
    kits: Vec<PathBuf>,
    kit_idx: usize,
    loaded: Option<LoadedKit>,
    app_keep: Vec<bool>,
    filter: String,
    filtering: bool,
    remount: bool,
    restore_wsl: bool,
    restore_browser: bool,
    log: Vec<String>,
    status: String,
}

impl App {
    fn new(prefer: Option<PathBuf>) -> Self {
        let kits = kit::find_kits();
        let kit_idx = prefer
            .as_ref()
            .and_then(|p| kits.iter().position(|k| k == p))
            .unwrap_or(0);
        let mut app = Self {
            tab: 0,
            cursor: 0,
            kits,
            kit_idx,
            loaded: None,
            app_keep: Vec::new(),
            filter: String::new(),
            filtering: false,
            remount: true,
            restore_wsl: true,
            restore_browser: true,
            log: Vec::new(),
            status: String::new(),
        };
        app.reload();
        app
    }

    fn reload(&mut self) {
        self.loaded = None;
        self.app_keep.clear();
        if let Some(dir) = self.kits.get(self.kit_idx).cloned() {
            match kit::load_kit(&dir) {
                Ok(k) => {
                    self.app_keep = vec![true; k.apps.len()];
                    self.status = format!("loaded {} ({} packages)", dir.display(), k.apps.len());
                    self.loaded = Some(k);
                }
                Err(e) => self.status = e,
            }
        } else {
            self.status = "no KIT.json found on D:–Z:\\Backups".into();
        }
    }

    fn visible_apps(&self) -> Vec<usize> {
        let Some(k) = &self.loaded else {
            return Vec::new();
        };
        let q = self.filter.to_ascii_lowercase();
        k.apps
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

    fn apply(&mut self, term: &mut ratatui::DefaultTerminal) {
        let Some(kit) = self.loaded.clone() else {
            self.status = "load a kit first".into();
            return;
        };
        self.log.clear();
        let ids: Vec<String> = kit
            .apps
            .iter()
            .zip(self.app_keep.iter())
            .filter(|(_, k)| **k)
            .map(|(a, _)| a.id.clone())
            .collect();
        let n = ids.len();
        for (i, id) in ids.iter().enumerate() {
            self.status = format!("winget install {id} ({}/{})", i + 1, n);
            let _ = term.draw(|f| draw(f, self));
            let r = restore::install_id(id);
            self.log.push(format!(
                "{}  {}  {}",
                if r.ok { "ok" } else { "!!" },
                r.id,
                r.detail
            ));
        }
        if self.remount {
            self.status = "remounting Dev Drive…".into();
            let _ = term.draw(|f| draw(f, self));
            match restore::remount_dev_drive(&kit) {
                Ok(s) => self.log.push(s),
                Err(e) => self.log.push(format!("Dev Drive: {e}")),
            }
        }
        if self.restore_wsl {
            self.status = "restoring WSL…".into();
            let _ = term.draw(|f| draw(f, self));
            match restore::restore_wsl(&kit) {
                Ok(s) => self.log.push(s),
                Err(e) => self.log.push(format!("WSL: {e}")),
            }
        }
        if self.restore_browser {
            self.status = "browser bookmarks + extensions page…".into();
            let _ = term.draw(|f| draw(f, self));
            match restore::restore_browser(&kit) {
                Ok(s) => self.log.push(s),
                Err(e) => self.log.push(format!("browser: {e}")),
            }
        }
        self.status = "done. 1Password SSH / Steam library / Docker WSL integration stay manual.".into();
        self.tab = 4;
    }
}

pub fn run(prefer: Option<PathBuf>) -> Result<(), String> {
    let mut app = App::new(prefer);
    let mut term = ratatui::init();
    let result = loop {
        term.draw(|f| draw(f, &app)).map_err(|e| e.to_string())?;
        let Event::Key(key) = event::read().map_err(|e| e.to_string())? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        if app.filtering {
            match key.code {
                KeyCode::Esc | KeyCode::Enter => app.filtering = false,
                KeyCode::Backspace => {
                    app.filter.pop();
                }
                KeyCode::Char(c) => app.filter.push(c),
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
            KeyCode::Char('/') if TABS[app.tab] == Tab::Apps => {
                app.filtering = true;
                app.filter.clear();
            }
            KeyCode::Char(' ') | KeyCode::Enter => match TABS[app.tab] {
                Tab::Kit => {
                    if app.cursor < app.kits.len() {
                        app.kit_idx = app.cursor;
                        app.reload();
                    }
                }
                Tab::Apps => {
                    let vis = app.visible_apps();
                    if let Some(&i) = vis.get(app.cursor) {
                        if let Some(v) = app.app_keep.get_mut(i) {
                            *v = !*v;
                        }
                    }
                }
                Tab::Disks => match app.cursor {
                    0 => app.remount = !app.remount,
                    1 => app.restore_wsl = !app.restore_wsl,
                    _ => {}
                },
                Tab::Browser => app.restore_browser = !app.restore_browser,
                Tab::Apply => app.apply(&mut term),
            },
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
    f.render_widget(
        Tabs::new(TITLES)
            .select(app.tab)
            .highlight_style(Style::new().fg(mint()).bold())
            .block(
                Block::bordered()
                    .title(" wsl-setup  restore ")
                    .border_style(Style::new().fg(Color::DarkGray)),
            ),
        chunks[0],
    );
    match TABS[app.tab] {
        Tab::Kit => f.render_widget(kit_body(app), chunks[1]),
        Tab::Apps => f.render_widget(apps_body(app), chunks[1]),
        Tab::Disks => f.render_widget(disks_body(app), chunks[1]),
        Tab::Browser => f.render_widget(browser_body(app), chunks[1]),
        Tab::Apply => f.render_widget(apply_body(app), chunks[1]),
    }
    f.render_widget(
        Paragraph::new("j/k move  space toggle  tab section  / filter  q quit")
            .style(Style::new().fg(Color::DarkGray)),
        chunks[2],
    );
    f.render_widget(
        Paragraph::new(app.status.as_str()).style(Style::new().fg(mint())),
        chunks[3],
    );
}

fn kit_body(app: &App) -> List<'static> {
    let items: Vec<ListItem> = if app.kits.is_empty() {
        vec![ListItem::new("No KIT.json under D:–Z:\\Backups")]
    } else {
        app.kits
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let mark = if i == app.kit_idx { "(*)" } else { "( )" };
                let it = ListItem::new(format!("{mark}  {}", p.display()));
                if i == app.cursor {
                    it.style(Style::new().fg(mint()).bg(Color::Rgb(28, 38, 48)))
                } else {
                    it
                }
            })
            .collect()
    };
    List::new(items).block(Block::bordered().title(" kits found on data drives "))
}

fn apps_body(app: &App) -> List<'static> {
    let vis = app.visible_apps();
    let items: Vec<ListItem> = vis
        .iter()
        .map(|&i| {
            let a = &app.loaded.as_ref().unwrap().apps[i];
            let mark = if app.app_keep[i] { "[x]" } else { "[ ]" };
            ListItem::new(format!("{mark}  {}  {}", a.id, a.version))
        })
        .enumerate()
        .map(|(row, it)| {
            if row == app.cursor {
                it.style(Style::new().fg(mint()).bg(Color::Rgb(28, 38, 48)))
            } else {
                it
            }
        })
        .collect();
    let n = app.app_keep.iter().filter(|k| **k).count();
    List::new(items).block(Block::bordered().title(format!(" winget packages in kit  {n} selected ")))
}

fn disks_body(app: &App) -> List<'static> {
    let rows = [
        (app.remount, "Remount Dev Drive VHDX as D:"),
        (app.restore_wsl, "Import WSL distros (import-in-place + ACL)"),
    ];
    let items: Vec<ListItem> = rows
        .into_iter()
        .enumerate()
        .map(|(i, (on, label))| {
            let mark = if on { "[x]" } else { "[ ]" };
            let it = ListItem::new(format!("{mark}  {label}"));
            if i == app.cursor {
                it.style(Style::new().fg(mint()).bg(Color::Rgb(28, 38, 48)))
            } else {
                it
            }
        })
        .collect();
    List::new(items).block(Block::bordered().title(" disks "))
}

fn browser_body(app: &App) -> List<'static> {
    let mark = if app.restore_browser { "[x]" } else { "[ ]" };
    let it = ListItem::new(format!(
        "{mark}  Copy Brave bookmarks and open extensions.html (Add to Brave by hand)"
    ))
    .style(Style::new().fg(mint()).bg(Color::Rgb(28, 38, 48)));
    List::new(vec![it]).block(Block::bordered().title(" browser "))
}

fn apply_body(app: &App) -> Paragraph<'static> {
    let mut t = String::from("Enter  run selected winget installs, then disks, then browser.\n\n");
    if app.log.is_empty() {
        t.push_str("Nothing run yet.");
    } else {
        for line in app.log.iter().rev().take(16).rev() {
            t.push_str(line);
            t.push('\n');
        }
    }
    Paragraph::new(t).block(Block::bordered().title(" apply "))
}
