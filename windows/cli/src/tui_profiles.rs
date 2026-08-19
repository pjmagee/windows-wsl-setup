use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::prelude::*;
use ratatui::widgets::{Block, List, ListItem, Paragraph, Tabs};

use crate::apply;
use crate::catalog::{
    BundleDoc, LinuxProfileDoc, Store, WindowsProfileDoc, WslSpec,
};
use crate::classify;
use crate::profile;
use crate::suggest;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Pick,
    Windows,
    Linux,
    Apply,
}

const TABS: [Tab; 4] = [Tab::Pick, Tab::Windows, Tab::Linux, Tab::Apply];
const TITLES: [&str; 4] = ["Pick", "Windows", "Linux", "Apply"];

enum Row {
    Header(String),
    Item(String),
}

struct App {
    tab: usize,
    cursor: usize,
    store: Store,
    bundle_ids: Vec<String>,
    bundle_idx: usize,
    windows_sel: Vec<String>,
    linux_sel: Vec<String>,
    dirty: bool,
    log: Vec<String>,
    status: String,
}

impl App {
    fn new() -> Result<Self, String> {
        let store = Store::load()?;
        let bundle_ids: Vec<String> = store.bundles.keys().cloned().collect();
        let mut app = Self {
            tab: 0,
            cursor: 0,
            store,
            bundle_ids,
            bundle_idx: 0,
            windows_sel: Vec::new(),
            linux_sel: Vec::new(),
            dirty: false,
            log: Vec::new(),
            status: "Pick a bundle. Space ticks. s save custom. g suggest from this PC.".into(),
        };
        app.load_bundle();
        Ok(app)
    }

    fn current_id(&self) -> String {
        self.bundle_ids
            .get(self.bundle_idx)
            .cloned()
            .unwrap_or_else(|| "custom".into())
    }

    fn load_bundle(&mut self) {
        let id = self.current_id();
        if let Ok(r) = profile::resolve(&self.store, &id) {
            self.windows_sel = r.windows.packages;
            self.linux_sel = r.linux.tools;
            self.dirty = false;
            self.status = format!("loaded {id}");
        }
    }

    fn working_docs(&self) -> (WindowsProfileDoc, LinuxProfileDoc, BundleDoc) {
        let id = if self.dirty {
            "custom".into()
        } else {
            self.current_id()
        };
        (
            WindowsProfileDoc {
                schema_version: 1,
                id: id.clone(),
                name: id.clone(),
                packages: self.windows_sel.clone(),
            },
            LinuxProfileDoc {
                schema_version: 1,
                id: id.clone(),
                name: id.clone(),
                tools: self.linux_sel.clone(),
            },
            BundleDoc {
                schema_version: 1,
                id: id.clone(),
                name: id,
                windows: "custom".into(),
                linux: "custom".into(),
                wsl: WslSpec::default(),
            },
        )
    }

    fn save_custom(&mut self) {
        let (w, l, mut b) = self.working_docs();
        b.windows = "custom".into();
        b.linux = "custom".into();
        b.id = "custom".into();
        self.store.windows.insert("custom".into(), w.clone());
        self.store.linux.insert("custom".into(), l.clone());
        self.store.bundles.insert("custom".into(), b);
        self.store.windows_source.insert("custom".into(), "user");
        self.store.linux_source.insert("custom".into(), "user");
        self.store.bundle_source.insert("custom".into(), "user");
        match profile::save(&self.store, "custom") {
            Ok(_) => {
                if !self.bundle_ids.iter().any(|x| x == "custom") {
                    self.bundle_ids.push("custom".into());
                }
                self.bundle_idx = self
                    .bundle_ids
                    .iter()
                    .position(|x| x == "custom")
                    .unwrap_or(self.bundle_idx);
                self.dirty = false;
                self.status = "saved custom under %USERPROFILE%\\.windows-wsl-setup\\profiles".into();
            }
            Err(e) => self.status = e,
        }
    }

    fn suggest_now(&mut self) {
        self.status = "scanning winget…".into();
        let s = suggest::from_machine(&self.store);
        self.windows_sel = s.windows;
        self.linux_sel = s.linux;
        self.dirty = true;
        self.status = format!(
            "suggested {} windows / {} linux (s to save as custom)",
            self.windows_sel.len(),
            self.linux_sel.len()
        );
        self.log.push(self.status.clone());
    }

    fn windows_rows(&self) -> Vec<Row> {
        catalog_rows(
            classify::WINDOWS_CATEGORIES,
            &self.store.windows_catalog.packages.iter().map(|p| (p.category.as_str(), p.id.as_str(), p.name.as_str())).collect::<Vec<_>>(),
            &self.windows_sel,
        )
    }

    fn linux_rows(&self) -> Vec<Row> {
        catalog_rows(
            classify::LINUX_CATEGORIES,
            &self.store.linux_catalog.tools.iter().map(|t| (t.category.as_str(), t.id.as_str(), t.name.as_str())).collect::<Vec<_>>(),
            &self.linux_sel,
        )
    }

    fn clamp(&mut self) {
        let n = match TABS[self.tab] {
            Tab::Pick => self.bundle_ids.len().max(1),
            Tab::Windows => self.windows_rows().len().max(1),
            Tab::Linux => self.linux_rows().len().max(1),
            Tab::Apply => 1,
        };
        if self.cursor >= n {
            self.cursor = n - 1;
        }
    }

    fn toggle(&mut self) {
        match TABS[self.tab] {
            Tab::Pick => {
                if self.cursor < self.bundle_ids.len() {
                    self.bundle_idx = self.cursor;
                    self.load_bundle();
                }
            }
            Tab::Windows => toggle_row(&self.windows_rows(), self.cursor, &mut self.windows_sel, &mut self.dirty),
            Tab::Linux => toggle_row(&self.linux_rows(), self.cursor, &mut self.linux_sel, &mut self.dirty),
            Tab::Apply => {}
        }
    }

    fn apply_now(&mut self, term: &mut ratatui::DefaultTerminal) {
        let (w, l, b) = self.working_docs();
        let r = profile::Resolved {
            bundle: b,
            windows: w,
            linux: l,
            source: "tui".into(),
        };
        self.status = "applying (winget + optional New WSL)…".into();
        let _ = term.draw(|f| draw(f, self));
        let steps = apply::apply_resolved(&r, true, true);
        for st in &steps {
            self.log.push(format!(
                "{} {} {}",
                if st.ok { "ok" } else { "!!" },
                st.step,
                st.detail.chars().take(80).collect::<String>()
            ));
            if self.log.len() > 40 {
                self.log.remove(0);
            }
        }
        let bad = steps.iter().filter(|s| !s.ok).count();
        self.status = if bad == 0 {
            "done".into()
        } else {
            format!("{bad} steps failed — see log")
        };
        self.tab = 3;
    }
}

fn catalog_rows(order: &[&str], items: &[(&str, &str, &str)], selected: &[String]) -> Vec<Row> {
    let mut rows = Vec::new();
    for cat in order {
        let in_cat: Vec<_> = items.iter().filter(|(c, _, _)| c == cat).copied().collect();
        if in_cat.is_empty() {
            continue;
        }
        rows.push(Row::Header((*cat).into()));
        for (_, id, _) in in_cat {
            let _ = selected;
            rows.push(Row::Item(id.into()));
        }
    }
    rows
}

fn toggle_row(rows: &[Row], cursor: usize, sel: &mut Vec<String>, dirty: &mut bool) {
    match rows.get(cursor) {
        Some(Row::Item(id)) => {
            if let Some(i) = sel.iter().position(|x| x == id) {
                sel.remove(i);
            } else {
                sel.push(id.clone());
            }
            *dirty = true;
        }
        Some(Row::Header(_)) => {
            let mut ids = Vec::new();
            for row in rows.iter().skip(cursor + 1) {
                match row {
                    Row::Header(_) => break,
                    Row::Item(id) => ids.push(id.clone()),
                }
            }
            let all_on = ids.iter().all(|id| sel.iter().any(|x| x == id));
            if all_on {
                sel.retain(|x| !ids.iter().any(|id| id == x));
            } else {
                for id in ids {
                    if !sel.iter().any(|x| x == &id) {
                        sel.push(id);
                    }
                }
            }
            *dirty = true;
        }
        None => {}
    }
}

pub fn run() -> Result<(), String> {
    let mut app = App::new()?;
    let mut term = ratatui::init();
    let result = loop {
        app.clamp();
        term.draw(|f| draw(f, &app)).map_err(|e| e.to_string())?;
        let Event::Key(key) = event::read().map_err(|e| e.to_string())? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
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
            KeyCode::Char(' ') => app.toggle(),
            KeyCode::Char('s') => app.save_custom(),
            KeyCode::Char('g') => app.suggest_now(),
            KeyCode::Enter => match TABS[app.tab] {
                Tab::Pick => app.toggle(),
                Tab::Apply => app.apply_now(&mut term),
                _ => app.toggle(),
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
    let tabs = Tabs::new(TITLES)
        .select(app.tab)
        .highlight_style(Style::new().fg(mint()).bold())
        .block(
            Block::bordered()
                .title(" Windows WSL Setup  profiles ")
                .border_style(Style::new().fg(Color::DarkGray)),
        );
    f.render_widget(tabs, chunks[0]);
    match TABS[app.tab] {
        Tab::Pick => f.render_widget(pick_body(app), chunks[1]),
        Tab::Windows => f.render_widget(tick_body(" windows (winget) ", &app.windows_rows(), &app.windows_sel, app.cursor), chunks[1]),
        Tab::Linux => f.render_widget(tick_body(" linux (Ubuntu 26.04 + Homebrew) ", &app.linux_rows(), &app.linux_sel, app.cursor), chunks[1]),
        Tab::Apply => f.render_widget(apply_body(app), chunks[1]),
    }
    f.render_widget(
        Paragraph::new("j/k move  space tick  s save custom  g suggest  tab section  Enter apply  q quit")
            .style(Style::new().fg(Color::DarkGray)),
        chunks[2],
    );
    f.render_widget(
        Paragraph::new(app.status.as_str()).style(Style::new().fg(mint())),
        chunks[3],
    );
}

fn pick_body(app: &App) -> List<'_> {
    let items: Vec<ListItem> = app
        .bundle_ids
        .iter()
        .enumerate()
        .map(|(i, id)| {
            let b = app.store.bundles.get(id);
            let name = b.map(|x| x.name.as_str()).unwrap_or(id);
            let src = app.store.bundle_source.get(id).copied().unwrap_or("");
            let mark = if i == app.bundle_idx { "(*)" } else { "( )" };
            let it = ListItem::new(format!("{mark}  {id}  — {name}  [{src}]"));
            if i == app.cursor {
                it.style(Style::new().bg(Color::Rgb(28, 38, 48)).fg(mint()))
            } else {
                it
            }
        })
        .collect();
    List::new(items).block(Block::bordered().title(" bundles — Enter to load "))
}

fn tick_body<'a>(title: &'a str, rows: &'a [Row], sel: &'a [String], cursor: usize) -> List<'a> {
    let items: Vec<ListItem> = rows
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let it = match row {
                Row::Header(cat) => {
                    ListItem::new(format!("── {cat} ──")).style(Style::new().fg(Color::DarkGray))
                }
                Row::Item(id) => {
                    let mark = if sel.iter().any(|x| x == id) { "[x]" } else { "[ ]" };
                    ListItem::new(format!("{mark}  {id}"))
                }
            };
            if i == cursor {
                it.style(Style::new().bg(Color::Rgb(28, 38, 48)).fg(mint()))
            } else {
                it
            }
        })
        .collect();
    List::new(items).block(Block::bordered().title(title))
}

fn apply_body(app: &App) -> Paragraph<'_> {
    let mut lines = vec![
        format!(
            "bundle: {}{}\nwindows packages: {}\nlinux tools: {}\n\nEnter = winget install + New WSL (Ubuntu 26.04) + install.sh\nDoes not remount disks. Collect/Restore is the kit path.\n",
            app.current_id(),
            if app.dirty { " (edited)" } else { "" },
            app.windows_sel.len(),
            app.linux_sel.len()
        ),
    ];
    for l in app.log.iter().rev().take(8).collect::<Vec<_>>().into_iter().rev() {
        lines.push(l.clone());
    }
    Paragraph::new(lines.join("\n")).block(Block::bordered().title(" apply "))
}
