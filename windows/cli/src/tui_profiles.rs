use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::prelude::*;
use ratatui::widgets::{Block, List, ListItem, Paragraph, Tabs};

use crate::apply;
use crate::catalog::{BundleDoc, LinuxProfileDoc, Store, WindowsProfileDoc, WslSpec};
use crate::classify;
use crate::new_wsl;
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
    distros: Vec<new_wsl::DistroChoice>,
    distro: usize,
    create_wsl: bool,
    dirty: bool,
    naming: bool,
    name_buf: String,
    pending_delete: Option<String>,
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
            distros: new_wsl::distro_choices(),
            distro: 0,
            create_wsl: true,
            dirty: false,
            naming: false,
            name_buf: String::new(),
            pending_delete: None,
            log: Vec::new(),
            status: "Pick a bundle. Space ticks. s save as. d delete. g suggest.".into(),
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
            self.create_wsl = r.bundle.wsl.create_if_missing;
            if let Ok(d) = new_wsl::parse_distro(&r.bundle.wsl.distro) {
                if let Some(i) = self.distros.iter().position(|c| c.id == d) {
                    self.distro = i;
                }
            }
            self.dirty = false;
            self.status = format!("loaded {id}");
        }
    }

    fn distro_id(&self) -> &str {
        self.distros
            .get(self.distro)
            .map(|d| d.id)
            .unwrap_or(new_wsl::DISTRO)
    }

    fn label(&self) -> (String, String) {
        let id = self.current_id();
        let name = self
            .store
            .bundles
            .get(&id)
            .map(|b| {
                if b.name.is_empty() {
                    b.id.clone()
                } else {
                    b.name.clone()
                }
            })
            .unwrap_or_else(|| id.clone());
        (id, name)
    }

    fn snapshot(&self, id: &str, name: &str) -> (WindowsProfileDoc, LinuxProfileDoc, BundleDoc) {
        (
            WindowsProfileDoc {
                schema_version: 1,
                id: id.into(),
                name: name.into(),
                packages: self.windows_sel.clone(),
            },
            LinuxProfileDoc {
                schema_version: 1,
                id: id.into(),
                name: name.into(),
                tools: self.linux_sel.clone(),
            },
            BundleDoc {
                schema_version: 1,
                id: id.into(),
                name: name.into(),
                windows: id.into(),
                linux: id.into(),
                wsl: WslSpec {
                    distro: self.distro_id().into(),
                    create_if_missing: self.create_wsl,
                },
            },
        )
    }

    fn begin_save(&mut self) {
        let id = self.current_id();
        self.name_buf = if self.store.bundle_source.get(&id) == Some(&"user") {
            self.store
                .bundles
                .get(&id)
                .map(|b| {
                    if b.name.is_empty() {
                        b.id.clone()
                    } else {
                        b.name.clone()
                    }
                })
                .unwrap_or(id)
        } else {
            String::new()
        };
        self.naming = true;
        self.status = "Type a name, then Enter.".into();
    }

    fn confirm_save(&mut self) {
        if self.name_buf.trim().is_empty() {
            self.status = "type a name, then Enter".into();
            return;
        }
        let wsl = WslSpec {
            distro: self.distro_id().into(),
            create_if_missing: self.create_wsl,
        };
        let windows = self.windows_sel.clone();
        let linux = self.linux_sel.clone();
        let raw = self.name_buf.clone();
        match profile::put_user(&mut self.store, &raw, windows, linux, wsl)
            .and_then(|r| profile::save(&self.store, &r.bundle.id).map(|_| r))
        {
            Ok(r) => {
                let id = r.bundle.id.clone();
                if !self.bundle_ids.iter().any(|x| x == &id) {
                    self.bundle_ids.push(id.clone());
                }
                self.bundle_idx = self
                    .bundle_ids
                    .iter()
                    .position(|x| x == &id)
                    .unwrap_or(self.bundle_idx);
                self.dirty = false;
                self.naming = false;
                self.status = format!(
                    "saved {} — {}  (%USERPROFILE%\\.windows-wsl-setup\\profiles)",
                    r.bundle.id, r.bundle.name
                );
            }
            Err(e) => self.status = e,
        }
    }

    fn delete_target(&self) -> String {
        if TABS[self.tab] == Tab::Pick {
            self.bundle_ids
                .get(self.cursor)
                .cloned()
                .unwrap_or_else(|| self.current_id())
        } else {
            self.current_id()
        }
    }

    fn begin_delete(&mut self) {
        let id = self.delete_target();
        if self.store.bundle_source.get(&id) != Some(&"user") {
            self.status = format!("{id} is shipped — only user profiles can be deleted");
            return;
        }
        self.pending_delete = Some(id);
        self.status = "Enter deletes this profile. Esc cancels.".into();
    }

    fn confirm_delete(&mut self) {
        let Some(id) = self.pending_delete.take() else {
            return;
        };
        match profile::delete(&mut self.store, &id) {
            Ok(_) => {
                let present: Vec<String> = self.store.bundles.keys().cloned().collect();
                self.bundle_ids.retain(|x| present.iter().any(|k| k == x));
                if self.bundle_idx >= self.bundle_ids.len() {
                    self.bundle_idx = self.bundle_ids.len().saturating_sub(1);
                }
                self.dirty = false;
                self.load_bundle();
                self.status = if self.store.bundle_source.get(&id) == Some(&"shipped") {
                    format!("removed user overlay — {id} is shipped again")
                } else {
                    format!("deleted {id}")
                };
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
            "suggested {} windows / {} linux (s to name and save)",
            self.windows_sel.len(),
            self.linux_sel.len()
        );
        self.log.push(self.status.clone());
    }

    fn windows_rows(&self) -> Vec<Row> {
        catalog_rows(
            classify::WINDOWS_CATEGORIES,
            &self
                .store
                .windows_catalog
                .packages
                .iter()
                .map(|p| (p.category.as_str(), p.id.as_str(), p.name.as_str()))
                .collect::<Vec<_>>(),
            &self.windows_sel,
        )
    }

    fn linux_rows(&self) -> Vec<Row> {
        catalog_rows(
            classify::LINUX_CATEGORIES,
            &self
                .store
                .linux_catalog
                .tools
                .iter()
                .map(|t| (t.category.as_str(), t.id.as_str(), t.name.as_str()))
                .collect::<Vec<_>>(),
            &self.linux_sel,
        )
    }

    fn clamp(&mut self) {
        let n = match TABS[self.tab] {
            Tab::Pick => self.bundle_ids.len().max(1),
            Tab::Windows => self.windows_rows().len().max(1),
            Tab::Linux => self.linux_rows().len().max(1),
            Tab::Apply => self.distros.len() + 2,
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
            Tab::Windows => toggle_row(
                &self.windows_rows(),
                self.cursor,
                &mut self.windows_sel,
                &mut self.dirty,
            ),
            Tab::Linux => toggle_row(
                &self.linux_rows(),
                self.cursor,
                &mut self.linux_sel,
                &mut self.dirty,
            ),
            Tab::Apply => {
                if self.cursor < self.distros.len() {
                    self.distro = self.cursor;
                } else if self.cursor == self.distros.len() {
                    self.create_wsl = !self.create_wsl;
                }
            }
        }
    }

    fn apply_now(&mut self, term: &mut ratatui::DefaultTerminal) {
        if self.create_wsl {
            if let Some(d) = self.distros.get(self.distro) {
                if !d.online {
                    self.status = format!(
                        "{} is not in `wsl --list --online`. Pick another distro.",
                        d.id
                    );
                    return;
                }
            }
        }
        let (id, name) = self.label();
        let (w, l, b) = self.snapshot(&id, &name);
        let r = profile::Resolved {
            bundle: b,
            windows: w,
            linux: l,
            source: "tui".into(),
        };
        self.status = format!(
            "applying (winget + {} {})…",
            if self.create_wsl {
                "New WSL"
            } else {
                "existing"
            },
            self.distro_id()
        );
        let _ = term.draw(|f| draw(f, self));
        let steps = apply::apply_resolved(&self.store, &r, true, true);
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
        if app.naming {
            match key.code {
                KeyCode::Esc => {
                    app.naming = false;
                    app.status = "save cancelled".into();
                }
                KeyCode::Enter => app.confirm_save(),
                KeyCode::Backspace => {
                    app.name_buf.pop();
                }
                KeyCode::Char(c) if !key.modifiers.intersects(KeyModifiers::CONTROL) => {
                    if app.name_buf.len() < 60 {
                        app.name_buf.push(c);
                    }
                }
                _ => {}
            }
            continue;
        }
        if app.pending_delete.is_some() {
            match key.code {
                KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('q') => {
                    app.pending_delete = None;
                    app.status = "delete cancelled".into();
                }
                KeyCode::Enter | KeyCode::Char('y') => app.confirm_delete(),
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
            KeyCode::Char(' ') => app.toggle(),
            KeyCode::Char('s') => app.begin_save(),
            KeyCode::Char('d') => app.begin_delete(),
            KeyCode::Char('g') => app.suggest_now(),
            KeyCode::Enter => match TABS[app.tab] {
                Tab::Pick => app.toggle(),
                Tab::Apply => {
                    if app.cursor == app.distros.len() + 1 {
                        app.apply_now(&mut term);
                    } else {
                        app.toggle();
                    }
                }
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
        Tab::Windows => f.render_widget(
            tick_body(
                " windows (winget) ",
                &app.windows_rows(),
                &app.windows_sel,
                app.cursor,
            ),
            chunks[1],
        ),
        Tab::Linux => f.render_widget(
            tick_body(
                " linux (Homebrew on the selected distro) ",
                &app.linux_rows(),
                &app.linux_sel,
                app.cursor,
            ),
            chunks[1],
        ),
        Tab::Apply => f.render_widget(apply_body(app), chunks[1]),
    }
    if let Some(id) = app.pending_delete.as_deref() {
        let name = app
            .store
            .bundles
            .get(id)
            .map(|b| b.name.as_str())
            .filter(|n| !n.is_empty() && *n != id)
            .unwrap_or(id);
        let extra = if name == id {
            String::new()
        } else {
            format!(" — {name}")
        };
        f.render_widget(
            Paragraph::new(Text::from(vec![
                Line::from(vec![
                    Span::raw("Delete "),
                    Span::styled(id.to_string(), Style::new().fg(mint()).bold()),
                    Span::raw(extra),
                    Span::raw("?"),
                ]),
                Line::from("Enter yes   Esc / n no").style(Style::new().fg(Color::DarkGray)),
            ])),
            chunks[2],
        );
    } else if app.naming {
        let preview = match crate::catalog::sanitize_id(&app.name_buf) {
            Ok(id) => format!("id {id}  —  Enter save  Esc cancel"),
            Err(_) if app.name_buf.trim().is_empty() => "type a name  —  Esc cancel".into(),
            Err(e) => format!("{e}  —  Esc cancel"),
        };
        f.render_widget(
            Paragraph::new(Text::from(vec![
                Line::from(vec![
                    Span::raw("Save as: "),
                    Span::styled(format!("{}_", app.name_buf), Style::new().fg(mint())),
                ]),
                Line::from(preview).style(Style::new().fg(Color::DarkGray)),
            ])),
            chunks[2],
        );
    } else {
        f.render_widget(
            Paragraph::new(
                "j/k move  space tick  s save as  d delete  g suggest  tab  Enter apply  q quit",
            )
            .style(Style::new().fg(Color::DarkGray)),
            chunks[2],
        );
    }
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
    let items: Vec<ListItem> =
        rows.iter()
            .enumerate()
            .map(|(i, row)| {
                let it = match row {
                    Row::Header(cat) => ListItem::new(format!("── {cat} ──"))
                        .style(Style::new().fg(Color::DarkGray)),
                    Row::Item(id) => {
                        let mark = if sel.iter().any(|x| x == id) {
                            "[x]"
                        } else {
                            "[ ]"
                        };
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

fn apply_body(app: &App) -> List<'_> {
    let mut items: Vec<ListItem> = Vec::new();
    for (i, d) in app.distros.iter().enumerate() {
        let mark = if i == app.distro { "(*)" } else { "( )" };
        let st = if d.installed {
            "installed"
        } else if d.online {
            "available"
        } else {
            "not on this PC"
        };
        let mut it = ListItem::new(format!("{mark}  {}  —  {}  [{st}]", d.id, d.label));
        if app.cursor == i {
            it = it.style(Style::new().bg(Color::Rgb(28, 38, 48)).fg(mint()));
        }
        items.push(it);
    }
    let create_i = app.distros.len();
    let cmark = if app.create_wsl { "[x]" } else { "[ ]" };
    let mut create = ListItem::new(format!("{cmark}  create WSL if missing"));
    if app.cursor == create_i {
        create = create.style(Style::new().bg(Color::Rgb(28, 38, 48)).fg(mint()));
    }
    items.push(create);
    let go_i = app.distros.len() + 1;
    let mut go = ListItem::new(">>>  Apply  (winget + selected distro + install.sh)");
    if app.cursor == go_i {
        go = go.style(Style::new().bg(Color::Rgb(28, 38, 48)).fg(mint()));
    }
    items.push(go);
    for l in app
        .log
        .iter()
        .rev()
        .take(6)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
    {
        items.push(ListItem::new(l.as_str()).style(Style::new().fg(Color::DarkGray)));
    }
    List::new(items).block(Block::bordered().title(format!(
        " apply — {}{}  {} win / {} linux — pick a distro ",
        app.current_id(),
        if app.dirty { " (edited)" } else { "" },
        app.windows_sel.len(),
        app.linux_sel.len()
    )))
}
