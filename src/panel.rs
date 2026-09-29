//! The ticket panel, on the right of a project's terminal: the project's
//! tickets, and a ticket's thread — where it stands and whose turn it is on
//! top, the talk folded up to its latest snapshot, and the gestures in one
//! place under it: accept or reject, moderate, reply, close or reopen.

use std::collections::{HashMap, HashSet};

use gpui_kit::component::button::ButtonVariants as _;
use gpui_kit::component::input::{Input, InputEvent, InputState, Paste, Textarea, TextareaState};
use serde_json::json;
use gpui_kit::component::text::TextView;
use gpui_kit::component::Disableable as _;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::aiball::{Aiball, Comment, Thread, TicketHeader, TicketRow};
use crate::ui::buttons::{self, Look as _};
use crate::ui::ticketref;
use crate::rowstate::{self, Band, Glyph, RowState, Stripe, Turn};
use crate::thread::{self as reading, DecisionState, Entry, Shape};
use crate::thread;
use crate::composer;
use crate::icons::{self, Icon};
use crate::tip::Tip as _;
use crate::theme::p;
use gpui_kit::component::scroll::{ScrollableElement as _, Scrollbar, ScrollbarAxis};


/// The user folds the panel away.
pub struct CollapsePanel;

/// A ticket row's height in the panel, near enough: two lines and padding.
const ROW_HEIGHT: f32 = 56.;

/// The user wants the tickets full screen.
pub struct OpenFullList;

/// The user flipped the thread's order: newest first when true.
pub struct OrderChanged(pub bool);

/// The detail went full screen, or back to the panel.
pub struct FullChanged;


/// What the panel is about: a project, and the agent of the terminal shown.
#[derive(Clone, Debug, PartialEq)]
pub struct Scope {
    pub project: String,
    pub agent: Option<String>,
    /// Shown from the projects' list, no session of it open.
    pub sessionless: bool,
}

struct Detail {
    ticket: u64,
    /// Its project: the panel's, or another one when opened from the
    /// full-screen list.
    project: Option<String>,
    thread: Option<Thread>,
    error: Option<String>,
    /// A gesture is on its way to aiball.
    busy: bool,
    /// Folded comments the user opened, and the ticket's own body.
    unfolded: HashSet<u64>,
    /// The thread's scroll: it opens on its latest word, next to the reply.
    scroll: ScrollHandle,
    /// Questions the reply answers: (message, question id).
    answers: Vec<(u64, String)>,
    /// The next reply notifies nobody.
    quiet: bool,
    /// The short menus under the reply box: snooze, priority.
    menu: Option<Menu>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Menu {
    Snooze,
    Priority,
}

/// Full screen, the invariant being changed in the left column.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Editing {
    Content,
    Intent,
    Priority,
    Level,
    Scope,
    Milestone,
    Tags,
    Owner,
    Assignee,
    Relation,
    Project,
}

/// What the full-screen detail offers to choose from, per project.
#[derive(Clone, Default)]
struct Catalog {
    project: String,
    tags: Vec<String>,
    milestones: Vec<(u64, String)>,
    agents: Vec<String>,
    projects: Vec<String>,
}

pub struct TicketPanel {
    aiball: Aiball,
    scope: Option<Scope>,
    tickets: Vec<TicketRow>,
    critical: Option<u64>,
    /// The tickets sunk in the shown agent's backlog, and until when:
    /// asked of aiball for that agent and project.
    sunk: HashMap<u64, String>,
    sunk_for: Option<(String, String)>,
    /// Asks counted, the latest answer only kept.
    sunk_asked: u64,
    detail: Option<Detail>,
    reply: Entity<TextareaState>,
    /// The bands folded to their title, and each band's own scroll.
    folded: HashSet<Band>,
    scrolls: HashMap<Band, ScrollHandle>,
    /// The thread's order: newest first (top-down) or last (by the reply).
    newest_first: bool,
    /// Who can be @mentioned, read once.
    mentions: Vec<String>,
    /// The detail fills the window: the ticket's invariants on the left
    /// third, the thread on the rest.
    full: bool,
    /// Full screen, the talk shows whole unless folded on demand.
    full_folded: bool,
    /// Full screen, the reply shown as it will read (the Preview tab).
    reply_preview: bool,
    editing: Option<Editing>,
    catalog: Option<Catalog>,
    edit_title: Entity<InputState>,
    edit_body: Entity<TextareaState>,
    relation_target: Entity<InputState>,
    /// What is typed to search the open list of choices (tags, people,
    /// projects): the long lists are searched, not scrolled.
    search: Entity<InputState>,
    /// Full screen, a comment's ⋯ menu, the one being edited, the one
    /// whose deletion waits for a confirming click.
    comment_menu: Option<u64>,
    comment_editing: Option<u64>,
    confirm_delete: Option<u64>,
    edit_comment: Entity<TextareaState>,
    /// The threads' images, read once.
    images: crate::images::Cache,
}

impl EventEmitter<OrderChanged> for TicketPanel {}
impl EventEmitter<FullChanged> for TicketPanel {}
impl EventEmitter<CollapsePanel> for TicketPanel {}
impl EventEmitter<OpenFullList> for TicketPanel {}


impl TicketPanel {
    pub fn new(aiball: Aiball, window: &mut Window, cx: &mut Context<Self>) -> Self {
        // The full-screen fields column's width, dragged here or on another
        // full page.
        cx.observe_global::<crate::sidecol::SideWidth>(|_, cx| cx.notify()).detach();
        let reply = cx.new(|cx| {
            TextareaState::new(window, cx)
                .placeholder("Reply… (ctrl+enter sends)")
                .auto_grow(2, 8)
        });
        // Reject waits for a reason: redraw as the reason is typed. (Ctrl+Enter
        // sends: the reply is a composer, see keymap's ComposerSend.)
        cx.subscribe_in(&reply, window, |_, _, event: &InputEvent, _, cx| {
            if let InputEvent::Change = event {
                cx.notify()
            }
        })
        .detach();
        let edit_title = cx.new(|cx| InputState::new(window, cx));
        let edit_body = cx.new(|cx| TextareaState::new(window, cx).auto_grow(4, 16));
        let relation_target = cx.new(|cx| InputState::new(window, cx).placeholder("#ticket"));
        let search = cx.new(|cx| InputState::new(window, cx).placeholder("search…"));
        cx.subscribe(&search, |_, _, event: &InputEvent, cx| {
            if matches!(event, InputEvent::Change) {
                cx.notify();
            }
        })
        .detach();
        let edit_comment = cx.new(|cx| TextareaState::new(window, cx).auto_grow(3, 16));
        // The rows a notification is about shine while it is up.
        cx.subscribe(&crate::bus::bus(cx), |_, _, signal: &crate::bus::Signal, cx| {
            if matches!(signal, crate::bus::Signal::Notices) {
                cx.notify();
            }
        })
        .detach();
        let reader = aiball.clone();
        cx.spawn(async move |this, cx| {
            let mentions = cx.background_executor().spawn(async move { reader.mention_suggestions() }).await;
            if let Ok(mentions) = mentions {
                let _ = this.update(cx, |panel, _| panel.mentions = mentions);
            }
        })
        .detach();
        Self {
            aiball,
            scope: None,
            sunk: HashMap::new(),
            sunk_for: None,
            sunk_asked: 0,
            tickets: Vec::new(),
            critical: None,
            detail: None,
            reply,
            folded: HashSet::new(),
            scrolls: HashMap::new(),
            newest_first: false,
            mentions: Vec::new(),
            full: false,
            full_folded: false,
            reply_preview: false,
            editing: None,
            catalog: None,
            edit_title,
            edit_body,
            relation_target,
            search,
            comment_menu: None,
            comment_editing: None,
            confirm_delete: None,
            edit_comment,
            images: Default::default(),
        }
    }

    /// A gesture on one comment: the menu closes once aiball has it.
    fn on_comment(
        &mut self,
        what: &str,
        run: impl FnOnce(&Aiball) -> anyhow::Result<()> + Send + 'static,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.gesture(
            what.to_string(),
            String::new(),
            run,
            |panel, _, _| {
                panel.comment_menu = None;
                panel.comment_editing = None;
                panel.confirm_delete = None;
            },
            window,
            cx,
        );
    }

    fn edit_comment_start(&mut self, comment: u64, body: String, window: &mut Window, cx: &mut Context<Self>) {
        self.comment_editing = Some(comment);
        self.comment_menu = None;
        self.edit_comment.update(cx, |input, cx| input.set_value(body, window, cx));
        cx.notify();
    }

    fn edit_comment_save(&mut self, comment: u64, window: &mut Window, cx: &mut Context<Self>) {
        let body = self.edit_comment.read(cx).value().to_string();
        self.on_comment("comment edited", move |aiball| aiball.edit(comment, json!({ "body": body })), window, cx);
    }

    pub fn is_full(&self) -> bool {
        self.full && self.detail.is_some()
    }

    pub fn set_full(&mut self, full: bool, cx: &mut Context<Self>) {
        if self.full != full {
            self.full = full;
            self.editing = None;
            if full {
                self.read_catalog(cx);
            }
            cx.emit(FullChanged);
            cx.notify();
        }
    }

    /// What the panel shows, to come back to: the open ticket (its project,
    /// its id), or `None` for the list.
    pub fn snapshot(&self) -> Option<(Option<String>, u64)> {
        self.detail.as_ref().map(|d| (d.project.clone(), d.ticket))
    }

    /// Shows again what [`TicketPanel::snapshot`] took.
    pub fn restore(&mut self, snapshot: Option<(Option<String>, u64)>, cx: &mut Context<Self>) {
        match snapshot {
            Some((project, ticket)) => {
                let same = self.detail.as_ref().is_some_and(|d| d.ticket == ticket && d.project == project);
                if !same {
                    self.open_in(project, ticket, cx);
                }
            }
            None => self.detail = None,
        }
        cx.notify();
    }

    /// The project the panel is about, if any.
    pub fn scope_project(&self) -> Option<String> {
        self.scope.as_ref().map(|s| s.project.clone())
    }

    /// The open ticket's project.
    fn project(&self) -> Option<String> {
        self.detail
            .as_ref()
            .and_then(|d| d.project.clone())
            .or_else(|| self.scope.as_ref().map(|s| s.project.clone()))
    }

    /// Reads what the left column offers to choose from.
    fn read_catalog(&mut self, cx: &mut Context<Self>) {
        let Some(project) = self.project() else { return };
        if self.catalog.as_ref().is_some_and(|c| c.project == project) {
            return;
        }
        let aiball = self.aiball.clone();
        cx.spawn(async move |this, cx| {
            let catalog = cx
                .background_executor()
                .spawn(async move {
                    let (projects, agents) = aiball.projects_and_agents().unwrap_or_default();
                    Catalog {
                        tags: aiball.tag_catalog(&project).unwrap_or_default(),
                        milestones: aiball.milestones(&project).unwrap_or_default(),
                        agents,
                        projects,
                        project,
                    }
                })
                .await;
            let _ = this.update(cx, |panel, cx| {
                panel.catalog = Some(catalog);
                cx.notify();
            });
        })
        .detach();
    }

    /// The names matching what is typed in the search, those that start
    /// with it first; nothing typed, the first ones. A handful at most.
    fn searched<'a>(&self, names: impl Iterator<Item = &'a String>, cx: &App) -> Vec<&'a String> {
        const SHOWN: usize = 8;
        let query = self.search.read(cx).value().trim().to_lowercase();
        let (mut first, rest): (Vec<&String>, Vec<&String>) = names
            .filter(|n| n.to_lowercase().contains(&query))
            .partition(|n| n.to_lowercase().starts_with(&query));
        first.extend(rest);
        first.truncate(SHOWN);
        first
    }

    fn search_box(&self) -> impl IntoElement + use<> {
        div().w(px(160.)).child(Input::new(&self.search))
    }

    fn start_editing(&mut self, editing: Editing, window: &mut Window, cx: &mut Context<Self>) {
        if self.editing == Some(editing) {
            self.editing = None;
        } else {
            self.editing = Some(editing);
            if matches!(editing, Editing::Tags | Editing::Owner | Editing::Assignee | Editing::Project) {
                self.search.update(cx, |input, cx| {
                    input.set_value("", window, cx);
                    input.focus(window, cx);
                });
            }
            if editing == Editing::Content {
                let ticket = self.detail.as_ref().and_then(|d| d.thread.as_ref()).map(|t| t.ticket.clone());
                if let Some(ticket) = ticket {
                    self.edit_title.update(cx, |input, cx| input.set_value(ticket.title.clone(), window, cx));
                    self.edit_body.update(cx, |input, cx| input.set_value(ticket.body.clone().unwrap_or_default(), window, cx));
                }
            }
        }
        cx.notify();
    }

    /// Changes an invariant: no reply is posted with it.
    fn change(
        &mut self,
        what: impl Into<String>,
        run: impl FnOnce(&Aiball, u64) -> anyhow::Result<()> + Send + 'static,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(ticket) = self.detail.as_ref().map(|d| d.ticket) else { return };
        self.gesture(
            what.into(),
            String::new(),
            move |aiball| run(aiball, ticket),
            |panel, _, _| panel.editing = None,
            window,
            cx,
        );
    }

    fn save_content(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let title = self.edit_title.read(cx).value().trim().to_string();
        let body = self.edit_body.read(cx).value().to_string();
        if title.is_empty() {
            return;
        }
        self.change("title and body edited", move |aiball, ticket| aiball.edit(ticket, json!({ "title": title, "body": body })), window, cx);
    }

    fn add_relation(&mut self, kind: &'static str, window: &mut Window, cx: &mut Context<Self>) {
        let text = self.relation_target.read(cx).value().to_string();
        let Ok(target) = text.trim().trim_start_matches(['#', 'B', '.']).parse::<u64>() else { return };
        self.relation_target.update(cx, |input, cx| input.set_value("", window, cx));
        self.change(format!("related to #{target} ({kind})"), move |aiball, ticket| aiball.relate(ticket, target, kind), window, cx);
    }

    pub fn set_newest_first(&mut self, newest_first: bool, cx: &mut Context<Self>) {
        self.newest_first = newest_first;
        cx.notify();
    }

    fn flip_order(&mut self, cx: &mut Context<Self>) {
        self.newest_first = !self.newest_first;
        if let Some(detail) = &self.detail {
            if self.newest_first {
                detail.scroll.set_offset(point(px(0.), px(0.)));
            } else {
                detail.scroll.scroll_to_bottom();
            }
        }
        cx.emit(OrderChanged(self.newest_first));
        cx.notify();
    }

    /// Shows another terminal's tickets; closes the open ticket when the
    /// project changes.
    pub fn set_scope(&mut self, scope: Option<Scope>, cx: &mut Context<Self>) {
        if self.scope.as_ref().map(|s| &s.project) != scope.as_ref().map(|s| &s.project) {
            self.detail = None;
        }
        self.scope = scope;
        self.load_sunk(cx);
        cx.notify();
    }

    /// Asks aiball which tickets are sunk in the shown agent's backlog (none
    /// without an agent), and again when the first pause ends.
    fn load_sunk(&mut self, cx: &mut Context<Self>) {
        let wanted = self.scope.as_ref().and_then(|s| Some((s.agent.clone()?, s.project.clone())));
        if wanted != self.sunk_for {
            self.sunk.clear();
            self.sunk_for = wanted.clone();
        }
        self.sunk_asked += 1;
        let asked = self.sunk_asked;
        let Some((agent, project)) = wanted else { return };
        let aiball = self.aiball.clone();
        cx.spawn(async move |this, cx| {
            let read = cx.background_executor().spawn(async move { aiball.agent_backlog(&agent, &project) }).await;
            let _ = this.update(cx, |panel, cx| {
                if panel.sunk_asked != asked {
                    return;
                }
                match read {
                    Ok(backlog) => {
                        panel.sunk = sunk_of(&backlog, crate::status::now());
                        cx.notify();
                    }
                    Err(error) => log::debug!("sunk tickets: {error:#}"),
                }
                // Read again as the first pause ends.
                let first = panel.sunk.values().filter_map(|until| crate::status::parse_time(until)).min();
                if let Some(first) = first {
                    let wait = first.saturating_sub(crate::status::now()) + 1;
                    cx.spawn(async move |this, cx| {
                        cx.background_executor().timer(std::time::Duration::from_secs(wait)).await;
                        let _ = this.update(cx, |panel, cx| {
                            if panel.sunk_asked == asked {
                                panel.load_sunk(cx);
                            }
                        });
                    })
                    .detach();
                }
            });
        })
        .detach();
    }

    /// The board as last read: this project's tickets, and who tvty is.
    pub fn set_board(
        &mut self,
        aiball: &Aiball,
        tickets: Vec<TicketRow>,
        critical: Option<u64>,
        cx: &mut Context<Self>,
    ) {
        self.aiball = aiball.clone();
        if self.tickets != tickets || self.critical != critical {
            // The open ticket moved (someone wrote, decided…): read its
            // thread again.
            let open = self.detail.as_ref().map(|d| d.ticket);
            let moved = open.is_some_and(|id| {
                let row = |rows: &[TicketRow]| rows.iter().find(|t| t.id == id).map(|t| (t.last_activity.clone(), t.comment_count));
                row(&self.tickets) != row(&tickets)
            });
            self.tickets = tickets;
            self.critical = critical;
            self.keep_scrolls();
            // A ticket moved: its pause may have ended.
            self.load_sunk(cx);
            if let (true, Some(id)) = (moved, open) {
                self.load(id, false, cx);
            }
            cx.notify();
        }
    }

    pub fn open(&mut self, ticket: u64, cx: &mut Context<Self>) {
        self.open_in(None, ticket, cx);
    }

    /// Opens a ticket of `project` (`None`: the panel's own).
    pub fn open_in(&mut self, project: Option<String>, ticket: u64, cx: &mut Context<Self>) {
        self.detail = Some(Detail {
            ticket,
            project,
            thread: None,
            error: None,
            busy: false,
            unfolded: HashSet::new(),
            scroll: ScrollHandle::new(),
            answers: Vec::new(),
            quiet: false,
            menu: None,
        });
        self.load(ticket, true, cx);
        cx.notify();
    }

    /// Reads the thread again; `mark_read` clears its unread for the user.
    fn load(&mut self, ticket: u64, mark_read: bool, cx: &mut Context<Self>) {
        let aiball = self.aiball.clone();
        let images = self.images.clone();
        cx.spawn(async move |this, cx| {
            let thread = cx
                .background_executor()
                .spawn(async move {
                    let mut thread = aiball.thread(ticket);
                    // Its images go in its texts, which cannot load them.
                    if let Ok(thread) = thread.as_mut() {
                        crate::images::inline(thread, &aiball, &images);
                    }
                    if mark_read && thread.is_ok() {
                        let _ = aiball.mark_read(ticket);
                    }
                    thread
                })
                .await;
            let _ = this.update(cx, |panel, cx| {
                let newest_first = panel.newest_first;
                if let Some(detail) = panel.detail.as_mut().filter(|d| d.ticket == ticket) {
                    match thread {
                        Ok(thread) => {
                            // First read, or the answer to a gesture: the
                            // latest word is what to see.
                            let grew = detail
                                .thread
                                .as_ref()
                                .is_none_or(|t| t.comments.len() != thread.comments.len());
                            if grew && !newest_first {
                                detail.scroll.scroll_to_bottom();
                            }
                            detail.thread = Some(thread);
                            detail.error = None;
                        }
                        Err(error) => detail.error = Some(format!("{error:#}")),
                    }
                    detail.busy = false;
                }
                if mark_read {
                    crate::bus::emit(cx, crate::bus::Signal::BoardChanged);
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// Sends a gesture to aiball, then reads the thread and the board again.
    /// Runs a gesture on the ticket shown; `what` says it to the activity
    /// service ("closed", "plan accepted"), which notifies how it went.
    fn gesture(
        &mut self,
        what: String,
        quote: String,
        run: impl FnOnce(&Aiball) -> anyhow::Result<()> + Send + 'static,
        on_success: impl FnOnce(&mut Self, &mut Window, &mut Context<Self>) + 'static,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let scope_project = self.scope.as_ref().map(|s| s.project.clone());
        let Some(detail) = self.detail.as_mut() else {
            return;
        };
        let ticket = detail.ticket;
        let about = detail.project.clone().or(scope_project).map(|project| (project, ticket));
        // Said as the others' are: the ticket's title, then what was done.
        let title = detail.thread.as_ref().map(|t| t.ticket.title.clone());
        detail.busy = true;
        detail.error = None;
        let aiball = self.aiball.clone();
        let window_handle = window.window_handle();
        cx.spawn(async move |this, cx| {
            let result = cx.background_executor().spawn(async move { run(&aiball) }).await;
            let _ = cx.update_window(window_handle, |_, window, cx| {
                let _ = this.update(cx, |panel, cx| {
                    match result {
                        Ok(()) => {
                            on_success(panel, window, cx);
                            panel.load(ticket, false, cx);
                            crate::bus::emit(cx, crate::bus::Signal::BoardChanged);
                            if !what.is_empty() {
                                let said = match title {
                                    Some(title) => format!("{title} — {what}"),
                                    None => what,
                                };
                                crate::activity::publish(cx, crate::activity::Activity::done(about, said).quoting(quote));
                            }
                        }
                        Err(error) => {
                            if let Some(detail) = panel.detail.as_mut() {
                                detail.busy = false;
                                detail.error = Some(format!("{error:#}"));
                            }
                            crate::activity::publish(cx, crate::activity::Activity::failed(about, &what, format!("{error:#}")));
                        }
                    }
                    cx.notify();
                });
            });
        })
        .detach();
        cx.notify();
    }

    /// Posts what the user typed, then `then`: every gesture carries its
    /// why.
    fn act(
        &mut self,
        what: &str,
        then: impl FnOnce(&Aiball, &str, u64) -> anyhow::Result<()> + Send + 'static,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let body = self.reply.read(cx).value().trim().to_string();
        // What it says: the gesture, and the reply that carries its why.
        let what = match (what.is_empty(), body.is_empty()) {
            (true, _) => "reply posted".to_string(),
            (false, true) => what.to_string(),
            (false, false) => format!("{what}, with a reply"),
        };
        let Some(detail) = self.detail.as_ref() else {
            return;
        };
        let Some(project) = detail
            .project
            .clone()
            .or_else(|| self.scope.as_ref().map(|s| s.project.clone()))
        else {
            return;
        };
        let ticket = detail.ticket;
        let (quiet, answers) = (detail.quiet, detail.answers.clone());
        // The reply's words, in its notification.
        let quote = crate::thread::excerpt(Some(&body), crate::live::EXCERPT);
        self.gesture(
            what,
            quote,
            move |aiball| {
                if !body.is_empty() {
                    let comment = aiball.reply(&project, ticket, &body, quiet)?;
                    // The questions it answers get their box ticked.
                    for (message, question) in &answers {
                        aiball.answer_question(*message, question, comment)?;
                    }
                }
                then(aiball, &project, ticket)
            },
            |panel, window, cx| {
                panel
                    .reply
                    .update(cx, |reply, cx| reply.set_value("", window, cx));
                panel.reply_preview = false;
                if let Some(detail) = panel.detail.as_mut() {
                    detail.answers.clear();
                    detail.quiet = false;
                    detail.menu = None;
                }
            },
            window,
            cx,
        );
    }

    /// Snoozes the ticket for `hours` (or wakes it: `None`).
    fn snooze(&mut self, hours: Option<u64>, window: &mut Window, cx: &mut Context<Self>) {
        let until = hours.map(|h| {
            let at = std::time::SystemTime::now() + std::time::Duration::from_secs(h * 3600);
            crate::status::format_time(at)
        });
        self.act(if until.is_some() { "snoozed" } else { "woken" }, move |aiball, _, ticket| aiball.snooze(ticket, until.as_deref()), window, cx);
    }

    fn set_priority(&mut self, priority: &'static str, window: &mut Window, cx: &mut Context<Self>) {
        self.act(&format!("priority {priority}"), move |aiball, _, ticket| aiball.set_priority(ticket, priority), window, cx);
    }

    fn toggle_menu(&mut self, menu: Menu, cx: &mut Context<Self>) {
        if let Some(detail) = self.detail.as_mut() {
            detail.menu = if detail.menu == Some(menu) { None } else { Some(menu) };
            cx.notify();
        }
    }

    /// Quotes a question into the reply; sending the reply ticks it.
    fn answer(&mut self, message: u64, question: thread::Question, window: &mut Window, cx: &mut Context<Self>) {
        let Some(detail) = self.detail.as_mut() else { return };
        if detail.answers.iter().any(|(m, q)| *m == message && *q == question.id) {
            return;
        }
        detail.answers.push((message, question.id));
        let text = self.reply.read(cx).value().to_string();
        crate::field::append(&self.reply, &composer::quote_block(&text, &question.text), window, cx);
        cx.notify();
    }

    /// The `@name` being typed at the end of the reply, if any.
    fn mention_typed(&self, cx: &App) -> Option<String> {
        composer::typed_mention(&crate::field::around_cursor(&self.reply, cx).0)
    }

    fn complete_mention(&mut self, name: String, window: &mut Window, cx: &mut Context<Self>) {
        let (before, _) = crate::field::around_cursor(&self.reply, cx);
        let Some(start) = composer::mention_start(&before) else { return };
        crate::field::replace_range(&self.reply, start..before.len(), &format!("@{name} "), window, cx);
        cx.notify();
    }

    /// Ctrl+V with an image on the clipboard: uploads it to aiball and puts
    /// its link in the reply. Answers whether it took the paste.
    fn paste_image(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let Some((bytes, content_type, name)) = composer::clipboard_image(cx) else {
            return false;
        };
        let aiball = self.aiball.clone();
        let window_handle = window.window_handle();
        if let Some(detail) = self.detail.as_mut() {
            detail.busy = true;
        }
        cx.spawn(async move |this, cx| {
            let uploaded = cx
                .background_executor()
                .spawn(async move { aiball.upload(&bytes, &content_type, &name) })
                .await;
            let _ = cx.update_window(window_handle, |_, window, cx| {
                let _ = this.update(cx, |panel, cx| {
                    if let Some(detail) = panel.detail.as_mut() {
                        detail.busy = false;
                        match uploaded {
                            Ok(url) => {
                                let (before, after) = crate::field::around_cursor(&panel.reply, cx);
                                crate::field::insert_at_cursor(&panel.reply, &composer::image_snippet(&before, &after, &url), window, cx);
                            }
                            Err(error) => detail.error = Some(format!("{error:#}")),
                        }
                    }
                    cx.notify();
                });
            });
        })
        .detach();
        true
    }

    fn decide(&mut self, message: u64, accept: bool, window: &mut Window, cx: &mut Context<Self>) {
        self.act(if accept { "decision accepted" } else { "decision rejected" }, move |aiball, _, _| aiball.decide(message, accept), window, cx);
    }

    fn moderate(&mut self, message: u64, approve: bool, window: &mut Window, cx: &mut Context<Self>) {
        self.act(if approve { "approved" } else { "rejected in moderation" }, move |aiball, _, _| aiball.moderate(message, approve), window, cx);
    }

    fn set_closed(&mut self, closed: bool, window: &mut Window, cx: &mut Context<Self>) {
        self.act(if closed { "closed" } else { "reopened" }, move |aiball, project, ticket| aiball.set_closed(project, ticket, closed), window, cx);
    }

    fn send_reply(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.reply.read(cx).value().trim().is_empty() {
            return;
        }
        self.act("", |_, _, _| Ok(()), window, cx);
    }

    fn toggle_fold(&mut self, id: u64, cx: &mut Context<Self>) {
        if let Some(detail) = self.detail.as_mut() {
            if !detail.unfolded.remove(&id) {
                detail.unfolded.insert(id);
            }
            cx.notify();
        }
    }

    fn list(&self, cx: &mut Context<Self>) -> AnyElement {
        if self.scope.is_none() {
            return hint("No aiball project on this terminal.").into_any_element();
        }
        let user = self.aiball.user.as_str();
        // Bands first, from "it is yours" to "it runs by itself"; the most
        // recent activity first within a band.
        let mut rows: Vec<(RowState, &TicketRow)> =
            self.tickets.iter().map(|t| (rowstate::of(t, user), t)).collect();
        rows.sort_by(|(a, ta), (b, tb)| {
            a.band
                .rank()
                .cmp(&b.band.rank())
                .then_with(|| tb.last_activity.cmp(&ta.last_activity))
        });

        // One section per band, each with its title — a click folds it —
        // and its own scroll: the bands share the height, a long one never
        // pushes the others out of sight.
        // When even the bands' minimums do not fit, the list scrolls as a
        // whole rather than letting them overlap.
        let mut list = crate::accordion::list("ticket-list").pb_1();
        if self.tickets.is_empty() {
            return list.child(hint("No open ticket.")).into_any_element();
        }
        let mut at = 0;
        let mut above: Option<SharedString> = None;
        while at < rows.len() {
            let band = rows[at].0.band;
            let end = rows[at..].iter().position(|(s, _)| s.band != band).map_or(rows.len(), |n| at + n);
            let count = end - at;
            let scroll = self.scrolls.get(&band).cloned().unwrap_or_default();
            // Only the rows in view are laid out: a band of hundreds stays cheap.
            let shown = crate::accordion::window(count, ROW_HEIGHT, &scroll);
            let id = SharedString::from(format!("band-{band:?}"));
            let section = crate::accordion::Section {
                list: "tickets".into(),
                above: above.replace(id.clone()),
                id,
                title: band.title().to_string(),
                count,
                folded: self.folded.contains(&band),
                // Two rows, or all of them when fewer.
                keep: count.min(2) as f32 * ROW_HEIGHT,
                body: rows[at + shown.start..at + shown.end]
                    .iter()
                    .map(|(state, ticket)| self.row(ticket, *state, cx).into_any_element())
                    .collect(),
                before: shown.start as f32 * ROW_HEIGHT,
                after: (count - shown.end) as f32 * ROW_HEIGHT,
                scroll,
                windowed: Some(crate::accordion::Windowed { drawn: shown.clone(), row: ROW_HEIGHT, owner: cx.entity_id() }),
            };
            list = list.child(section.render(cx.listener(move |panel, _, _, cx| {
                if !panel.folded.remove(&band) {
                    panel.folded.insert(band);
                }
                cx.notify();
            })));
            at = end;
        }
        list.into_any_element()
    }

    /// Keeps a scroll per band across renders.
    fn keep_scrolls(&mut self) {
        let user = self.aiball.user.clone();
        for ticket in &self.tickets {
            let band = rowstate::of(ticket, &user).band;
            self.scrolls.entry(band).or_default();
        }
    }

    /// One ticket: a stripe for whose turn, one state glyph, the title
    /// (bold when unread), then who spoke last, who holds it, and when.
    fn row(&self, ticket: &TicketRow, state: RowState, cx: &mut Context<Self>) -> impl IntoElement {
        let id = ticket.id;
        // Sunk in the shown agent's backlog: steps back, a ⤓ says until when.
        let sunk = self.sunk.get(&id).and_then(|until| crate::status::resume_short(until));
        let group: SharedString = format!("ticket-row-{id}").into();
        let sunk_tip = sunk.as_ref().map(|until| {
            let agent = self.sunk_for.as_ref().map(|(a, _)| a.clone()).unwrap_or_default();
            format!("sunk in {agent}'s backlog until {until}: its loop won't bring it up before, unless the thread moves")
        });
        let lit = crate::notify::lit(cx, id);
        let yours = state.turn == Turn::You;
        let glyph_colour = |glyph: Glyph| glyph_colour(glyph, yours);
        let speaker = ticket.last_speaker.as_deref().map(|s| {
            if s == self.aiball.user {
                "you".to_string()
            } else {
                s.to_string()
            }
        });
        let meta = div()
            .flex()
            .overflow_hidden()
            .whitespace_nowrap()
            .items_center()
            .gap_x_2()
            .text_xs()
            .text_color(p().muted)
            .when_some(speaker, |d, who| d.child(who))
            .children(comment_count(ticket, &self.aiball.user))
            .children(holder_chip(ticket))
            .children(critical_chip(ticket))
            .children(priority_chip(ticket, 14.))
            .child(div().flex_1())
            .when_some(sunk_tip, |d, tip| d.child(div().id(("sunk", id)).flex_none().child("⤓").tip(tip)))
            .when_some(ticket.last_activity.as_deref().and_then(ago), |d, when| d.child(when));
        // Stepped back while sunk; itself again under the pointer.
        let dim = sunk.is_some();
        let content = div()
            .flex()
            .flex_col()
            .gap_0p5()
            .flex_1()
            .min_w_0()
            .when(dim, |d| d.opacity(0.5).group_hover(group.clone(), |s| s.opacity(1.)));

        div()
            .id(("ticket", id))
            .group(group.clone())
            .h(px(ROW_HEIGHT))
            .flex_none()
            .overflow_hidden()
            .flex()
            .gap_2()
            .pl_2()
            .pr_3()
            .py_1p5()
            .cursor_pointer()
            .hover(|d| d.bg(p().hover))
            .child(row_edge(&state))
            .child(
                div()
                    .w(px(16.))
                    .pt_0p5()
                    .flex_none()
                    .when_some(state.glyph, |d, glyph| d.child(glyph_chip(glyph, glyph_colour(glyph), 16., ticket))),
            )
            .child(
                content
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .items_center()
                            .child(div().flex_none().text_color(p().muted).child(format!("#{id}")))
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .truncate()
                                    // Weight says unread, and nothing else does: read
                                    // titles step back so bold stands out.
                                    .map(|d| {
                                        if ticket.unread {
                                            d.font_weight(FontWeight::BOLD).text_color(p().text)
                                        } else {
                                            d.font_weight(FontWeight::NORMAL).text_color(p().text.opacity(0.72))
                                        }
                                    })
                                    .child(ticket.title.clone()),
                            ),
                    )
                    .child(meta),
            )
            // A notification about it is up: it shines.
            .map(|d| crate::notify::halo(d, lit))
            .on_click(cx.listener(move |panel, _, _, cx| panel.open(id, cx)))
    }

    fn detail(&self, detail: &Detail, cx: &mut Context<Self>) -> AnyElement {
        let Some(thread) = &detail.thread else {
            return match &detail.error {
                Some(error) => hint(error.clone()).into_any_element(),
                None => hint("Loading…").into_any_element(),
            };
        };
        let ticket = &thread.ticket;
        let user = self.aiball.user.clone();
        let read = reading::read(thread);
        let row = self.tickets.iter().find(|t| t.id == ticket.id);
        let state = row.map(|r| rowstate::of(r, &user));
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        let sentence = reading::sentence(
            thread,
            &read,
            state.as_ref(),
            row.is_some_and(|r| r.stalled_step),
            &user,
            now,
        );
        let glyph = state.as_ref().map_or_else(|| reading::glyph(thread, &read), |s| s.glyph);
        let yours = state.as_ref().is_some_and(|s| s.turn == Turn::You);

        // ── Where it stands: said once ──────────────────────────────────
        let title = div()
            .flex()
            .gap_2()
            .text_lg()
            .font_weight(FontWeight::BOLD)
            .items_center()
            .when_some(glyph, |d, glyph| d.child(icons::icon(icons::of_glyph(glyph), glyph_colour(glyph, yours), 20.)))
            .child(div().flex_1().min_w_0().child(format!("#{} {}", ticket.id, ticket.title)))
            .child(
                buttons::link("thread-order", if self.newest_first { "⇅ newest first" } else { "⇅ newest last" })
                    .text_xs()
                    .font_weight(FontWeight::NORMAL)
                    .on_click(cx.listener(|panel, _, _, cx| panel.flip_order(cx))),
            )
            .child(
                buttons::link("thread-full", if self.full { "✕  Esc" } else { "⤢ more" })
                    .text_xs()
                    .font_weight(FontWeight::NORMAL)
                    .on_click(cx.listener(|panel, _, _, cx| {
                        let full = !panel.full;
                        panel.set_full(full, cx)
                    })),
            );
        let turn = (!sentence.is_empty()).then(|| {
            let (stripe_kind, colour) = state
                .as_ref()
                .map_or((Stripe::None, p().border), |s| (s.stripe, stripe_colour(s)));
            div()
                .flex()
                .gap_2()
                .child(stripe(stripe_kind, colour))
                .child(
                    div()
                        .flex_1()
                        .when(yours, |d| d.font_weight(FontWeight::BOLD))
                        .text_color(if yours { p().text } else { p().muted })
                        .child(ticketref::inline("turn", &sentence)),
                )
        });
        let mut chips: Vec<AnyElement> = Vec::new();
        if let Some(holder) = ticket.holder() {
            let hot = row.is_some_and(|r| r.hot);
            let text = format!("held by {holder}");
            chips.push(if hot {
                icons::labelled(Icon::Hot, p().warning, 12., text).into_any_element()
            } else {
                div().child(text).into_any_element()
            });
        }
        {
            // The priority, a click away from changing.
            let priority = ticket.priority.clone().unwrap_or_else(|| "normal".into());
            let label = match icons::priority(&priority) {
                Some(icon) => icons::labelled(icon, icons::priority_colour(&priority), 14., priority.clone()),
                None => div().child(priority.clone()),
            };
            chips.push(
                buttons::chip("priority-chip", label)
                    .tip("the priority: a click changes it")
                    .on_click(cx.listener(|panel, _, _, cx| panel.toggle_menu(Menu::Priority, cx)))
                    .into_any_element(),
            );
        }
        if let Some(until) = ticket.postponed_until.as_deref() {
            chips.push(div().child(format!("snoozed until {}", until.get(..16).unwrap_or(until).replace('T', " "))).into_any_element());
        }
        if let Some(critical) = &ticket.critical {
            chips.push(icons::pill(Icon::Critical, critical.said(), p().danger).into_any_element());
        }
        if let Some(usage) = &ticket.token_usage {
            let total = usage.tokens_in + usage.tokens_out + usage.cache_w;
            if total > 0 {
                chips.push(div().child(format!("{} tok", count(total))).into_any_element());
            }
        }
        for relation in &ticket.relations {
            // aiball gives a relation as seen from this ticket, a reciprocal
            // one's kind already turned round.
            let verb = match relation.kind.as_str() {
                "depends_on" => "depends on",
                "blocks" => "blocks",
                _ => continue,
            };
            let target = relation.target_ticket_id;
            let glyph = self
                .tickets
                .iter()
                .find(|t| t.id == target)
                .and_then(|row| rowstate::of(row, &user).glyph);
            let stage = relation.target_stage.clone().filter(|_| glyph.is_none());
            chips.push(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(verb)
                    .child(ticketref::link(SharedString::from(format!("chip-rel-{target}")), target, None))
                    .when_some(stage, |d, stage| d.child(stage))
                    .when_some(glyph, |d, glyph| d.child(icons::icon(icons::of_glyph(glyph), p().muted, 12.)))
                    .into_any_element(),
            );
        }
        let summary = read.summary.clone().map(|(text, by)| {
            div()
                .flex()
                .flex_col()
                .gap_0p5()
                .px_2()
                .py_1p5()
                .rounded_md()
                .bg(p().hover)
                .child(div().text_xs().text_color(p().muted).child(format!("Where it stands · {}", who(&by, &user))))
                // Selectable, as the thread's words: it is copied as often.
                .child(
                    TextView::markdown("thread-summary", crate::ui::ticketref::linkify(&text))
                        .selectable(true)
                        .on_link_click(crate::ui::ticketref::on_link),
                )
        });
        // Full screen, the title spans the top, the state and chips go to
        // the left column, the summary heads the talk; in the panel they
        // all make the head.
        let chips_row = |chips: Vec<AnyElement>| {
            div()
                .flex()
                .flex_wrap()
                .items_center()
                .gap_x_3()
                .gap_y_1()
                .text_xs()
                .text_color(p().muted)
                .children(chips)
        };
        let (head, full_parts) = if self.full {
            (None, Some((title, turn, chips, summary)))
        } else {
            let head = div()
                .flex()
                .flex_col()
                .flex_none()
                .gap_1p5()
                .px_3()
                .pt_2()
                .pb_2()
                .border_b_1()
                .border_color(p().border)
                // The way back to the list, above what it leads back from.
                .child(
                    div().flex().child(
                        buttons::link("back", "← Tickets").text_sm().on_click(cx.listener(|panel, _, _, cx| {
                            panel.detail = None;
                            cx.notify();
                        })),
                    ),
                )
                .child(title)
                .children(turn)
                .when(!chips.is_empty(), |d| d.child(chips_row(chips)))
                .children(summary);
            (Some(head), None)
        };

        // ── The talk, folded up to its latest snapshot ──────────────────
        let has_talk = read.entries.iter().any(|e| !matches!(e.shape, Shape::Event { .. }));
        let body_open = !has_talk || detail.unfolded.contains(&ticket.id);
        let mut talk: Vec<AnyElement> = Vec::new();
        let opening = div()
            .flex()
            .flex_col()
            .gap_2()
            .child(self.entry_head(
                ticket.id,
                &ticket.by_agent,
                &ticket.created_at,
                read.ticket_decision.clone(),
                None,
                false,
                has_talk.then_some(body_open),
                &user,
                cx,
            ))
            .map(|d| match (&ticket.body, body_open) {
                (Some(text), true) => d.child(self.rich_text(format!("body-{}", ticket.id), text, cx)),
                (Some(text), false) => d.child(folded_line(ticket.id, reading::first_line(Some(text)), cx)),
                (None, _) => d,
            });
        talk.push(opening.into_any_element());
        let comments: std::collections::HashMap<u64, &Comment> =
            thread.comments.iter().map(|c| (c.id, c)).collect();
        for entry in &read.entries {
            let Some(comment) = comments.get(&entry.id) else {
                continue;
            };
            talk.push(self.entry(entry, comment, detail, &user, cx));
        }
        if self.newest_first {
            talk.reverse();
        }
        let body = div().flex().flex_col().px_3().py_2().gap_2().children(talk);

        // ── The gestures, in one place ──────────────────────────────────
        let typed = !self.reply.read(cx).value().trim().is_empty();
        // A ticket waiting for moderation is not let through yet: its
        // proposal is read, commented, but decided only once it is.
        let unmoderated = ticket.status == "pending";
        let decision = read.active.clone().filter(|a| a.by != user).map(|active| {
            let message = active.message;
            let accept = match active.kind.as_str() {
                "resolution" => "Accept → close",
                "wontfix" => "Accept → close, no fix",
                "escalation" => "Done → accept",
                _ => "Accept → go",
            };
            div()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .child(icons::labelled(
                                    icons::of_kind(&active.kind),
                                    p().warning,
                                    16.,
                                    {
                                        let noun = reading::kind_noun(&active.kind);
                                        let article = if noun.starts_with(['a', 'e', 'i', 'o', 'u']) { "an" } else { "a" };
                                        format!("{} proposes {article} {noun}", active.by)
                                    },
                                )),
                        )
                        .when(unmoderated, |d| {
                            d.child(div().flex_none().text_xs().text_color(p().muted).child("decided once the ticket is approved"))
                        })
                        .when(!unmoderated, |d| d.child(
                            buttons::answer("reject", "Reject")
                                .danger()
                                .disabled(detail.busy || !typed)
                                .on_click(cx.listener(move |panel, _, window, cx| panel.decide(message, false, window, cx))),
                        )
                        .child(
                            buttons::answer("accept", accept)
                                .success()
                                .disabled(detail.busy)
                                .on_click(cx.listener(move |panel, _, window, cx| panel.decide(message, true, window, cx))),
                        )),
                )
                .when(!typed && !unmoderated, |d| {
                    d.child(div().text_xs().text_color(p().muted).child("To reject, say why below first."))
                })
        });
        let moderation = unmoderated.then(|| {
            let id = ticket.id;
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(div().flex_1().child("This ticket waits for moderation"))
                .child(
                    buttons::answer("moderate-reject", "Reject")
                        .danger()
                        .disabled(detail.busy)
                        .on_click(cx.listener(move |panel, _, window, cx| panel.moderate(id, false, window, cx))),
                )
                .child(
                    buttons::answer("moderate-approve", "Approve")
                        .success()
                        .disabled(detail.busy)
                        .on_click(cx.listener(move |panel, _, window, cx| panel.moderate(id, true, window, cx))),
                )
        });
        let closed = ticket.closed;
        let snoozed = ticket.postponed_until.is_some();
        // `@` at the end of the reply: who it can be.
        let mentions = self.mention_typed(cx).map(|typed| {
            composer::mention_chips(&self.mentions, &typed, "mention", cx, |panel: &mut Self, name, window, cx| {
                panel.complete_mention(name, window, cx)
            })
        });
        let chip = |id: &'static str, label: &'static str| buttons::chip(id, label).py_0p5().text_xs();
        let menu = detail.menu.map(|menu| {
            let row = div().flex().flex_wrap().items_center().gap_1();
            match menu {
                Menu::Snooze => row
                    .child(div().text_xs().text_color(p().muted).child("Snooze for"))
                    .child(chip("snooze-1h", "1 hour").on_click(cx.listener(|panel, _, window, cx| panel.snooze(Some(1), window, cx))))
                    .child(chip("snooze-1d", "a day").on_click(cx.listener(|panel, _, window, cx| panel.snooze(Some(24), window, cx))))
                    .child(chip("snooze-1w", "a week").on_click(cx.listener(|panel, _, window, cx| panel.snooze(Some(24 * 7), window, cx)))),
                Menu::Priority => {
                    let mut row = row.child(div().text_xs().text_color(p().muted).child("Priority"));
                    for (id, priority) in [("prio-urgent", "urgent"), ("prio-high", "high"), ("prio-normal", "normal"), ("prio-low", "low")] {
                        row = row.child(
                            chip(id, priority).on_click(cx.listener(move |panel, _, window, cx| panel.set_priority(priority, window, cx))),
                        );
                    }
                    row
                }
            }
        });
        let quiet = detail.quiet;
        let actions = div()
            .id("ticket-actions")
            .flex()
            .flex_col()
            .flex_none()
            .gap_2()
            .p_3()
            .map(|d| if self.newest_first { d.border_b_1() } else { d.border_t_1() })
            .border_color(p().border)
            // A paste with an image: it goes to aiball, its link to the
            // reply. Caught before the reply box pastes text.
            .capture_action(cx.listener(|panel, _: &Paste, window, cx| {
                if panel.paste_image(window, cx) {
                    cx.stop_propagation();
                }
            }))
            .when_some(detail.error.clone(), |d, error| {
                d.child(div().text_color(p().danger).child(error))
            })

            .children(moderation)
            .children(decision)
            // Full screen, Write and Preview above the reply.
            .when(self.full, |d| {
                d.child(crate::composer::write_tabs("reply", self.reply_preview, cx, |panel: &mut Self, on, cx| {
                    panel.reply_preview = on;
                    if on {
                        panel.load_preview_images(cx);
                    }
                    cx.notify();
                }))
            })
            .child(if self.full && self.reply_preview {
                let text = self.reply.read(cx).value().to_string();
                div()
                    .min_h(px(96.))
                    .p_3()
                    .rounded_md()
                    .border_1()
                    .border_color(p().border)
                    .child(if text.trim().is_empty() {
                        div().text_color(p().muted).child("Nothing to preview yet.")
                    } else {
                        self.rich_text("reply-preview".into(), &crate::images::preview(&text, &self.images, false), cx)
                    })
                    .into_any_element()
            } else {
                // A composer: Ctrl+Enter sends (keymap's ComposerSend), no
                // new line put in.
                crate::focusmode::on_hover(
                    div()
                        .key_context(crate::keymap::COMPOSER)
                        .on_action(cx.listener(|panel, _: &crate::keymap::ComposerSend, window, cx| panel.send_reply(window, cx)))
                        .child(Textarea::new(&self.reply)),
                    self.reply.read(cx).focus_handle(cx),
                )
                .into_any_element()
            })
            .children(mentions)
            .when(!detail.answers.is_empty(), |d| {
                d.child(div().text_xs().text_color(p().muted).child(format!(
                    "Sending answers {} question{}.",
                    detail.answers.len(),
                    if detail.answers.len() > 1 { "s" } else { "" }
                )))
            })
            .children(menu)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        buttons::secondary("close", if closed { "Reopen" } else { "Close" })
                            .disabled(detail.busy)
                            .on_click(cx.listener(move |panel, _, window, cx| panel.set_closed(!closed, window, cx))),
                    )
                    .when(!closed, |d| {
                        d.child(if snoozed {
                            buttons::secondary("wake", "Wake")
                                .disabled(detail.busy)
                                .on_click(cx.listener(|panel, _, window, cx| panel.snooze(None, window, cx)))
                        } else {
                            buttons::secondary("snooze", "Snooze ▾")
                                .disabled(detail.busy)
                                .on_click(cx.listener(|panel, _, _, cx| panel.toggle_menu(Menu::Snooze, cx)))
                        })
                    })
                    .child(div().flex_1())
                    .child(
                        buttons::chip("quiet", "without notifying")
                            .py_0p5()
                            .text_xs()
                            .when(!quiet, |d| d.text_color(p().muted))
                            .chosen(quiet)
                            .on_click(cx.listener(|panel, _, _, cx| {
                                if let Some(detail) = panel.detail.as_mut() {
                                    detail.quiet = !detail.quiet;
                                }
                                cx.notify();
                            })),
                    )
                    .child(
                        crate::tips::target(
                            "panel.reply",
                            buttons::primary("send", "Reply")
                                .loading(detail.busy)
                                .on_click(cx.listener(|panel, _, window, cx| panel.send_reply(window, cx))),
                        )
                        .flex_none(),
                    ),
            );

        let thread_view = div()
            .relative()
            .flex_1()
            .min_h_0()
            .child(
                div()
                    .id("ticket-thread")
                    .size_full()
                    .track_scroll(&detail.scroll)
                    .overflow_y_scroll()
                    .child(body),
            )
            .child(
                div().absolute().inset_0().child(
                    Scrollbar::new(&detail.scroll)
                        .axis(ScrollbarAxis::Vertical)
                        .viewport_from_layout(),
                ),
            );
        // Newest last, the reply sits under the talk; newest first, above it.
        let (first, second) = if self.newest_first {
            (actions.into_any_element(), thread_view.into_any_element())
        } else {
            (thread_view.into_any_element(), actions.into_any_element())
        };
        let Some((title, turn_line, chips, summary_full)) = full_parts else {
            return div()
                .flex()
                .flex_col()
                .flex_1()
                .min_h_0()
                .children(head)
                .child(first)
                .child(second)
                .into_any_element();
        };

        // ── Full screen: the invariants on the left third ───────────────
        let left = self.invariants(ticket, turn_line, chips_row(chips), &user, cx);
        // How much was said, and whose word is the last: the list's count.
        let said: Vec<&Comment> = thread.comments.iter().filter(|c| c.kind == "comment_added").collect();
        let count = said.last().map(|last| {
            let mine = last.by_agent == user;
            let colour = if mine { p().muted.opacity(0.55) } else { p().text };
            let who = if mine { "you".to_string() } else { last.by_agent.clone() };
            icons::labelled(
                if mine { Icon::CommentsMine } else { Icon::Comments },
                colour,
                13.,
                format!("{} comment{} · {who} spoke last", said.len(), if said.len() == 1 { "" } else { "s" }),
            )
            .text_xs()
            .text_color(colour)
        });
        let fold = buttons::link("fold-all", if self.full_folded { "unfold all" } else { "fold before the summary" })
            .text_xs()
            .on_click(cx.listener(|panel, _, _, cx| {
                panel.full_folded = !panel.full_folded;
                cx.notify();
            }));
        div()
            .flex()
            .flex_col()
            .size_full()
            .child(
                div()
                    .flex_none()
                    .px_4()
                    .py_2()
                    .border_b_1()
                    .border_color(p().border)
                    .child(title),
            )
            .child(
                crate::sidecol::row()
                    .child(left)
                    .child(crate::sidecol::edge("fields-edge"))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .min_w_0()
                            .child(
                                // The talk takes the whole width left.
                                div()
                                    .flex()
                                    .flex_col()
                                    .w_full()
                                    .h_full()
                                    // The count and the fold on their line, the
                                    // summary under them, the talk's width: a
                                    // long one wraps instead of widening it.
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .flex_none()
                                            .gap_1()
                                            .px_3()
                                            .pt_2()
                                            .min_w_0()
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .gap_2()
                                                    .child(div().flex_1())
                                                    .children(count)
                                                    .child(fold),
                                            )
                                            .children(summary_full.map(|s| s.w_full().min_w_0())),
                                    )
                                    .child(first)
                                    .child(second),
                            ),
                    ),
            )
            .into_any_element()
    }

    /// The full-screen detail's left third: what holds for the ticket as a
    /// whole — its state, its fields, who is on it, what it is linked to.
    /// A click on a field offers its values underneath; a click on one sets
    /// it (no reply goes with it).
    fn invariants(&self, ticket: &TicketHeader, turn: Option<Div>, chips: Div, user: &str, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let editing = self.editing;
        let busy = self.detail.as_ref().is_some_and(|d| d.busy);
        let catalog = self.catalog.clone().unwrap_or_default();
        let label = |text: &'static str| div().w(px(96.)).flex_none().text_color(p().muted).child(text);
        // A row: a click opens its choices, when it has some.
        let row = |id: &'static str, text: &'static str, value: String, edit: Option<Editing>, cx: &mut Context<Self>| {
            let open = edit.is_some() && edit == editing;
            div()
                .id(id)
                .flex()
                .gap_2()
                .py_0p5()
                .px_1()
                .rounded_sm()
                .when(open, |d| d.bg(p().active))
                .when_some(edit, |d, edit| {
                    d.cursor_pointer()
                        .hover(|d| d.bg(p().hover))
                        .on_click(cx.listener(move |panel, _, window, cx| panel.start_editing(edit, window, cx)))
                })
                .child(label(text))
                .child(div().flex_1().min_w_0().child(value))
        };
        let choice = |id: SharedString, text: String, on: bool, cx: &mut Context<Self>, set: Box<dyn Fn(&mut Self, &mut Window, &mut Context<Self>)>| {
            buttons::chip(id, text)
                .py_0p5()
                .text_xs()
                .when(!on, |d| d.text_color(p().muted))
                .chosen(on)
                .on_click(cx.listener(move |panel, _, window, cx| {
                    if !busy {
                        set(panel, window, cx)
                    }
                }))
        };
        let choices = || div().flex().flex_wrap().gap_1().pl(px(100.)).pb_1();
        let group = |title: &'static str| {
            div()
                .pt_3()
                .pb_1()
                .text_xs()
                .font_weight(FontWeight::BOLD)
                .text_color(p().muted)
                .child(title.to_uppercase())
        };
        let date = |when: &str| when.get(..16).map(|w| w.replace('T', " ")).unwrap_or_else(|| when.to_string());
        let lifecycle = match (ticket.closed, ticket.resolved) {
            (true, true) => "closed, resolved".to_string(),
            (true, false) => "closed".to_string(),
            (false, _) if ticket.status == "pending" => "waits for moderation".to_string(),
            (false, _) => "open".to_string(),
        };

        let mut col = div()
            .id("ticket-invariants")
            .flex()
            .flex_col()
            // Room for the scrollbar.
            .pr_3()
            .text_sm()
            .child(group("State"))
            .children(turn)
            .child(div().pt_1().child(chips))
            .child(row("inv-lifecycle", "lifecycle", lifecycle, None, cx))
            .children(ticket.postponed_until.as_deref().map(|until| row("inv-snoozed", "snoozed", format!("until {}", date(until)), None, cx)));

        // ── Content ──
        col = col.child(group("Content")).child(row(
            "inv-content",
            "title, body",
            if editing == Some(Editing::Content) { "editing…".into() } else { "edit".into() },
            Some(Editing::Content),
            cx,
        ));
        if editing == Some(Editing::Content) {
            col = col.child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .pb_2()
                    .child(Input::new(&self.edit_title))
                    .child(Textarea::new(&self.edit_body))
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .justify_end()
                            .child(
                                buttons::secondary("content-cancel", "Cancel")
                                    .on_click(cx.listener(|panel, _, window, cx| panel.start_editing(Editing::Content, window, cx))),
                            )
                            .child(
                                buttons::primary("content-save", "Save")
                                    .loading(busy)
                                    .on_click(cx.listener(|panel, _, window, cx| panel.save_content(window, cx))),
                            ),
                    ),
            );
        }

        // ── Fields ──
        col = col.child(group("Fields"));
        let fields: [(&'static str, &'static str, Option<String>, &'static str, Editing, &'static [&'static str]); 4] = [
            ("inv-intent", "intent", ticket.intent.clone(), "intent", Editing::Intent, &["request", "question", "fyi", "feature", "panic"]),
            ("inv-priority", "priority", ticket.priority.clone(), "priority", Editing::Priority, &["urgent", "high", "normal", "low"]),
            ("inv-level", "level", ticket.level.clone(), "level", Editing::Level, &["task", "milestone", "roadmap"]),
            ("inv-scope", "scope", ticket.scope.clone(), "scope", Editing::Scope, &["internal", "default", "broadcast"]),
        ];
        for (id, text, value, field, edit, values) in fields {
            col = col.child(row(id, text, value.clone().unwrap_or_else(|| "—".into()), Some(edit), cx));
            if editing == Some(edit) {
                let mut list = choices();
                for v in values {
                    let v: &'static str = v;
                    list = list.child(choice(
                        SharedString::from(format!("{id}-{v}")),
                        v.to_string(),
                        value.as_deref() == Some(v),
                        cx,
                        Box::new(move |panel, window, cx| {
                            panel.change(format!("{field} {v}"), move |aiball, ticket| aiball.edit(ticket, json!({ field: v })), window, cx)
                        }),
                    ));
                }
                col = col.child(list);
            }
        }
        let milestone = ticket.milestone.as_ref().and_then(|m| m.title.clone());
        col = col.child(row("inv-milestone", "milestone", milestone.clone().unwrap_or_else(|| "—".into()), Some(Editing::Milestone), cx));
        if editing == Some(Editing::Milestone) {
            let mut list = choices().child(choice(
                "milestone-none".into(),
                "none".into(),
                milestone.is_none(),
                cx,
                Box::new(|panel, window, cx| panel.change("milestone removed", |aiball, ticket| aiball.set_milestone(ticket, None), window, cx)),
            ));
            for (id, title) in &catalog.milestones {
                let id = *id;
                list = list.child(choice(
                    SharedString::from(format!("milestone-{id}")),
                    title.clone(),
                    milestone.as_deref() == Some(title.as_str()),
                    cx,
                    Box::new(move |panel, window, cx| panel.change("milestone set", move |aiball, ticket| aiball.set_milestone(ticket, Some(id)), window, cx)),
                ));
            }
            if catalog.milestones.is_empty() {
                list = list.child(div().text_xs().text_color(p().muted).child("no open milestone in this project"));
            }
            col = col.child(list);
        }
        let tags: Vec<String> = ticket.tags.iter().map(|t| t.name.clone()).collect();
        col = col.child(row(
            "inv-tags",
            "tags",
            if tags.is_empty() { "—".into() } else { tags.join(", ") },
            Some(Editing::Tags),
            cx,
        ));
        if editing == Some(Editing::Tags) {
            let mut list = choices().child(self.search_box());
            for tag in self.searched(catalog.tags.iter(), cx) {
                let on = tags.contains(tag);
                let name = tag.clone();
                list = list.child(choice(
                    SharedString::from(format!("tag-{tag}")),
                    tag.clone(),
                    on,
                    cx,
                    Box::new(move |panel, window, cx| {
                        let name = name.clone();
                        panel.change(
                            if on { format!("tag {name} removed") } else { format!("tagged {name}") },
                            move |aiball, ticket| if on { aiball.remove_tag(ticket, &name) } else { aiball.add_tag(ticket, &name) },
                            window,
                            cx,
                        )
                    }),
                ));
            }
            col = col.child(list);
        }

        // ── People ──
        let claim = match &ticket.claimant {
            Some(claimant) if ticket.held.lapsed() => format!("{} (lapsed)", who(claimant, user)),
            Some(claimant) => format!(
                "{}{}",
                who(claimant, user),
                ticket.claim_until.as_deref().map(|u| format!(", until {}", date(u))).unwrap_or_default()
            ),
            None => "—".into(),
        };
        col = col
            .child(group("People"))
            .child(row("inv-reporter", "reporter", format!("{} · {}", who(&ticket.by_agent, user), date(&ticket.created_at)), Some(Editing::Owner), cx));
        if editing == Some(Editing::Owner) {
            let mut list = choices().child(self.search_box());
            let mut people: Vec<&String> = catalog.agents.iter().chain(std::iter::once(&self.aiball.user)).collect();
            people.sort();
            people.dedup();
            for agent in self.searched(people.into_iter(), cx) {
                let name = agent.clone();
                list = list.child(choice(
                    SharedString::from(format!("owner-{agent}")),
                    agent.clone(),
                    *agent == ticket.by_agent,
                    cx,
                    Box::new(move |panel, window, cx| {
                        let name = name.clone();
                        panel.change(format!("reporter now {name}"), move |aiball, ticket| aiball.set_owner(ticket, &name), window, cx)
                    }),
                ));
            }
            col = col.child(list);
        }
        col = col.child(row("inv-claim", "claimed by", claim, None, cx)).child(row(
            "inv-assignee",
            "assigned to",
            ticket.assignee.as_deref().map_or("—".into(), |a| who(a, user)),
            Some(Editing::Assignee),
            cx,
        ));
        if editing == Some(Editing::Assignee) {
            let mut list = choices().child(self.search_box());
            if ticket.holder().is_some() {
                list = list.child(choice(
                    "assign-release".into(),
                    "release".into(),
                    false,
                    cx,
                    Box::new(|panel, window, cx| panel.change("released", |aiball, ticket| aiball.assign(ticket, None), window, cx)),
                ));
            }
            for agent in self.searched(catalog.agents.iter(), cx) {
                let name = agent.clone();
                list = list.child(choice(
                    SharedString::from(format!("assign-{agent}")),
                    agent.clone(),
                    ticket.assignee.as_deref() == Some(agent.as_str()),
                    cx,
                    Box::new(move |panel, window, cx| {
                        let name = name.clone();
                        panel.change(format!("assigned to {name}"), move |aiball, ticket| aiball.assign(ticket, Some(&name)), window, cx)
                    }),
                ));
            }
            col = col.child(list);
        }

        // ── Links ──
        let (parent, parent_project) = (ticket.id, self.project());
        col = col.child(group("Links")).child(
            div()
                .id("inv-sub-ticket")
                .flex()
                .gap_2()
                .py_0p5()
                .px_1()
                .rounded_sm()
                .cursor_pointer()
                .hover(|d| d.bg(p().hover))
                .child(label("sub-ticket"))
                .child(div().flex_1().text_color(p().accent).child("+ a new one"))
                .on_click(cx.listener(move |_, _, _, cx| {
                    crate::bus::emit(cx, crate::bus::Signal::AskNewTicket { project: parent_project.clone(), parent: Some(parent) })
                })),
        );
        // A row of references: its label, then each one a link.
        let refs = |text: &'static str, ids: Vec<u64>| {
            div()
                .flex()
                .gap_2()
                .py_0p5()
                .px_1()
                .child(label(text))
                .child(div().flex_1().min_w_0().flex().flex_wrap().gap_2().children(
                    ids.into_iter().map(|id| ticketref::link(SharedString::from(format!("inv-ref-{text}-{id}")), id, None)),
                ))
        };
        if let Some(parent) = ticket.parent_ticket_id {
            col = col.child(refs("sub-ticket of", vec![parent]));
        }
        if !ticket.sub_tickets.is_empty() {
            let subs: Vec<u64> = ticket
                .sub_tickets
                .iter()
                .filter_map(|t| t.get("id").and_then(|id| id.as_u64()).or_else(|| t.as_u64()))
                .collect();
            col = col.child(refs("sub-tickets", subs));
        }
        for relation in &ticket.relations {
            // As seen from this ticket (a reciprocal's kind already turned
            // round by aiball); only "duplicates" keeps its word both ways.
            let verb = match (relation.kind.as_str(), relation.reciprocal) {
                ("depends_on", _) => "depends on",
                ("blocks", _) => "blocks",
                ("relates_to", _) => "relates to",
                ("duplicates", false) => "duplicates",
                ("duplicates", true) => "duplicated by",
                ("parent_of", _) => "parent of",
                ("child_of", _) => "child of",
                _ => continue,
            };
            let target = relation.target_ticket_id;
            let stage = relation.target_stage.clone().map(|s| format!(" ({s})")).unwrap_or_default();
            // Only a relation made on this ticket can be undone from here.
            let remove = (!relation.reciprocal).then(|| {
                buttons::remove(SharedString::from(format!("unrelate-{target}")), "✕", "remove this relation")
                    .text_xs()
                    .on_click(cx.listener(move |panel, _, window, cx| {
                        panel.change(format!("relation to #{target} removed"), move |aiball, ticket| aiball.relate(ticket, target, "ignored"), window, cx)
                    }))
            });
            col = col.child(
                div()
                    .flex()
                    .gap_2()
                    .py_0p5()
                    .px_1()
                    .child(label(verb))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .gap_1()
                            .child(ticketref::link(SharedString::from(format!("inv-rel-{target}")), target, None))
                            .child(stage.trim().to_string()),
                    )
                    .children(remove),
            );
        }
        col = col.child(row(
            "inv-relate",
            "relate",
            if editing == Some(Editing::Relation) { "to…".into() } else { "+ a ticket".into() },
            Some(Editing::Relation),
            cx,
        ));
        if editing == Some(Editing::Relation) {
            let mut list = choices().child(div().w(px(90.)).child(Input::new(&self.relation_target)));
            for (id, kind) in [("rel-depends", "depends_on"), ("rel-blocks", "blocks"), ("rel-relates", "relates_to"), ("rel-dup", "duplicates")] {
                list = list.child(choice(
                    id.into(),
                    kind.replace('_', " "),
                    false,
                    cx,
                    Box::new(move |panel, window, cx| panel.add_relation(kind, window, cx)),
                ));
            }
            col = col.child(list);
        }

        // ── Elsewhere ──
        let project = self.project().unwrap_or_default();
        col = col.child(group("Project")).child(row("inv-project", "project", project.clone(), Some(Editing::Project), cx));
        if editing == Some(Editing::Project) {
            let mut list = choices().child(self.search_box());
            for other in self.searched(catalog.projects.iter().filter(|p| **p != project), cx) {
                let name = other.clone();
                list = list.child(choice(
                    SharedString::from(format!("move-{other}")),
                    format!("move to {other}"),
                    false,
                    cx,
                    Box::new(move |panel, window, cx| {
                        let name = name.clone();
                        panel.change(format!("moved to {name}"), move |aiball, ticket| aiball.move_ticket(ticket, &name), window, cx)
                    }),
                ));
            }
            col = col.child(list);
        }

        if let Some(usage) = &ticket.token_usage {
            col = col
                .child(group("Tokens"))
                .child(row("inv-tokens", "in · out", format!("{} · {}", count(usage.tokens_in), count(usage.tokens_out)), None, cx))
                .child(row("inv-cache", "cache", format!("{} written · {} read", count(usage.cache_w), count(usage.cache_r)), None, cx));
        }
        if ticket.has_payload {
            col = col.child(group("Payload")).child(div().text_color(p().muted).child("this ticket carries a payload (see the web UI)"));
        }

        // As dragged, or its default: on a wide screen the rest goes to
        // the talk (see crate::sidecol).
        crate::sidecol::column(cx)
            .px_4()
            .pb_4()
            .bg(p().surface)
            .child(col.overflow_y_scrollbar())
    }

    /// One entry of the thread: an event line, a folded comment, or a whole
    /// one.
    fn entry(&self, entry: &Entry, comment: &Comment, detail: &Detail, user: &str, cx: &mut Context<Self>) -> AnyElement {
        let (unfolded, busy) = (&detail.unfolded, detail.busy);
        if let Shape::Event { verb, count } = &entry.shape {
            let times = if *count > 1 { format!(" ×{count}") } else { String::new() };
            let line = format!(
                "{} {verb}{times}{}",
                who(&comment.by_agent, user),
                ago(&comment.created_at).map(|a| format!(" · {a}")).unwrap_or_default()
            );
            return ticketref::inline(format!("event-{}", entry.id), &line).text_xs().text_color(p().muted).into_any_element();
        }
        // Full screen, everything shows whole unless the user folds it.
        let foldable = matches!(entry.shape, Shape::Folded(_)) && (!self.full || self.full_folded);
        let open = !foldable || unfolded.contains(&entry.id);
        let moderation = entry.pending.then(|| {
            let id = entry.id;
            div()
                .flex()
                .gap_1()
                .child(
                    buttons::answer(("comment-reject", id), "Reject")
                        .danger()
                        .disabled(busy)
                        .on_click(cx.listener(move |panel, _, window, cx| panel.moderate(id, false, window, cx))),
                )
                .child(
                    buttons::answer(("comment-approve", id), "Approve")
                        .success()
                        .disabled(busy)
                        .on_click(cx.listener(move |panel, _, window, cx| panel.moderate(id, true, window, cx))),
                )
        });
        let commits = comment.commits();
        // Its open questions: a click quotes one into the reply, which ticks
        // it once sent.
        let questions = thread::questions(comment.body.as_deref());
        let mut questions_row = div().flex().flex_col().gap_1();
        for question in &questions {
            let queued = detail.answers.iter().any(|(m, q)| *m == comment.id && *q == question.id);
            let (message, pick) = (comment.id, question.clone());
            questions_row = questions_row.child(
                buttons::chip(
                    SharedString::from(format!("question-{}-{}", comment.id, question.id)),
                    div().flex_none().text_color(p().accent).child(if queued { "✓ in the reply" } else { "Answer" }),
                )
                .gap_2()
                .py_0p5()
                .text_xs()
                .chosen(queued)
                    .child(div().flex_1().min_w_0().truncate().child(question.text.clone()))
                    .on_click(cx.listener(move |panel, _, window, cx| panel.answer(message, pick.clone(), window, cx))),
            );
        }
        div()
            .flex()
            .flex_col()
            .gap_1()
            .pt_2()
            .border_t_1()
            .border_color(p().border)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(div().flex_1().min_w_0().child(self.entry_head(
                        entry.id,
                        &comment.by_agent,
                        &comment.created_at,
                        entry.decision.clone(),
                        entry.step.map(|latest| (latest, comment.step_resume())),
                        entry.pending,
                        foldable.then_some(open),
                        user,
                        cx,
                    )))
                    .children(self.full.then(|| self.comment_tools(comment, cx))),
            )
            .children((self.full && self.comment_menu == Some(comment.id)).then(|| self.comment_menu_row(entry, comment, user, cx)))
            .map(|d| match (&entry.shape, open, &comment.body) {
                _ if self.comment_editing == Some(comment.id) => d.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(Textarea::new(&self.edit_comment))
                        .child(
                            div()
                                .flex()
                                .gap_2()
                                .justify_end()
                                .child(buttons::secondary(("comment-cancel", comment.id), "Cancel").on_click(
                                    cx.listener(|panel, _, _, cx| {
                                        panel.comment_editing = None;
                                        cx.notify();
                                    }),
                                ))
                                .child({
                                    let id = comment.id;
                                    buttons::primary(("comment-save", comment.id), "Save")
                                        .loading(busy)
                                        .on_click(cx.listener(move |panel, _, window, cx| panel.edit_comment_save(id, window, cx)))
                                }),
                        ),
                ),
                (Shape::Folded(line), false, _) => d.child(folded_line(entry.id, line.clone(), cx)),
                (_, _, Some(text)) => d.child(self.rich_text(format!("comment-{}", comment.id), text, cx)),
                _ => d,
            })
            .children(moderation)
            .when(open && !questions.is_empty(), |d| d.child(questions_row))
            .when(open && !commits.is_empty(), |d| {
                d.child(div().text_xs().text_color(p().muted).child(commits.join(" · ")))
            })
            .into_any_element()
    }

    /// Full screen, at a comment's right: its votes, and ⋯ for its menu.
    fn comment_tools(&self, comment: &Comment, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let votes = comment.votes_summary.clone().unwrap_or_default();
        let id = comment.id;
        div()
            .flex()
            .flex_none()
            .items_center()
            .gap_2()
            .text_xs()
            .text_color(p().muted)
            .when(votes.up > 0, |d| d.child(format!("+{}", votes.up)))
            .when(votes.down > 0, |d| d.child(format!("−{}", votes.down)))
            .child(
                buttons::icon(("comment-menu", id), "⋯", "the comment's actions")
                    .on_click(cx.listener(move |panel, _, _, cx| {
                        panel.comment_menu = if panel.comment_menu == Some(id) { None } else { Some(id) };
                        panel.confirm_delete = None;
                        cx.notify();
                    })),
            )
    }

    /// A comment's gestures: edit, delete, classify, step, vote, resurface,
    /// copy its reference.
    fn comment_menu_row(&self, entry: &Entry, comment: &Comment, user: &str, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let id = comment.id;
        let busy = self.detail.as_ref().is_some_and(|d| d.busy);
        let item = |key: &str, label: String, colour: Hsla| {
            buttons::chip(SharedString::from(format!("{key}-{id}")), label)
                .py_0p5()
                .text_xs()
                .text_color(colour)
        };
        let mut row = div().flex().flex_wrap().gap_1().pb_1();
        if let Some(body) = comment.body.clone() {
            row = row.child(item("edit", "Edit".into(), p().text).on_click(cx.listener(move |panel, _, window, cx| {
                panel.edit_comment_start(id, body.clone(), window, cx)
            })));
        }
        let confirming = self.confirm_delete == Some(id);
        row = row.child(
            item("delete", if confirming { "Really delete?".into() } else { "Delete".into() }, p().danger).on_click(cx.listener(
                move |panel, _, window, cx| {
                    if busy {
                        return;
                    }
                    if panel.confirm_delete == Some(id) {
                        panel.on_comment("comment deleted", move |aiball| aiball.delete_comment(id), window, cx);
                    } else {
                        panel.confirm_delete = Some(id);
                        cx.notify();
                    }
                },
            )),
        );
        // One way to classify, the four kinds; accepting stays with the
        // decision card.
        let current = entry.decision.as_ref().filter(|(_, s)| *s == DecisionState::Pending).map(|(k, _)| k.clone());
        for kind in ["plan", "resolution", "wontfix", "escalation"] {
            if current.as_deref() == Some(kind) {
                continue;
            }
            row = row.child(
                item(&format!("classify-{kind}"), format!("as {}", reading::kind_noun(kind)), p().text)
                    .on_click(cx.listener(move |panel, _, window, cx| panel.on_comment(&format!("comment made a {}", reading::kind_noun(kind)), move |aiball| aiball.classify(id, kind), window, cx))),
            );
        }
        if current.is_some() {
            row = row.child(
                item("untag", "no decision".into(), p().text)
                    .on_click(cx.listener(move |panel, _, window, cx| panel.on_comment("comment made plain", move |aiball| aiball.untag(id), window, cx))),
            );
        }
        if comment.by_agent != user && entry.decision.is_none() {
            let step = entry.step.is_some();
            row = row.child(
                item("step", if step { "not a step".into() } else { "a step".into() }, p().text)
                    .on_click(cx.listener(move |panel, _, window, cx| panel.on_comment(if step { "step unmarked" } else { "step marked" }, move |aiball| aiball.set_step(id, !step), window, cx))),
            );
        }
        let mine = comment.votes_summary.as_ref().and_then(|v| v.mine).unwrap_or(0);
        for (key, label, value) in [("up", "👍", 1), ("down", "👎", -1)] {
            let on = mine == value;
            row = row.child(
                item(key, if on { format!("{label} ✓") } else { label.to_string() }, p().text).on_click(cx.listener(move |panel, _, window, cx| {
                    let value = if on { 0 } else { value };
                    panel.on_comment("voted", move |aiball| aiball.vote(id, value), window, cx)
                })),
            );
        }
        row = row.child(
            item("resurface", "Resurface".into(), p().text)
                .on_click(cx.listener(move |panel, _, window, cx| panel.on_comment("comment resurfaced", move |aiball| aiball.resurface(id), window, cx))),
        );
        if let Some(hashid) = comment.hashid.clone() {
            row = row.child(item("copy", format!("#C.{hashid}"), p().muted).on_click(cx.listener(move |panel, _, _, cx| {
                cx.write_to_clipboard(ClipboardItem::new_string(format!("#C.{hashid}")));
                crate::bus::emit(cx, crate::bus::Signal::Copied);
                panel.comment_menu = None;
                cx.notify();
            })));
        }
        row
    }

    /// Who, when, and the chips of an entry; `fold`, when it folds: whether
    /// it is open — then ▾ or ▸ leads, as the accordions' sections, and a
    /// click on the head folds or unfolds it.
    #[allow(clippy::too_many_arguments)]
    fn entry_head(
        &self,
        id: u64,
        by: &str,
        when: &str,
        decision: Option<(String, DecisionState)>,
        step: Option<(bool, (Option<String>, Option<u64>))>,
        pending: bool,
        fold: Option<bool>,
        user: &str,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        div()
            .id(("entry", id))
            .flex()
            .flex_wrap()
            .items_center()
            .gap_2()
            .text_xs()
            .text_color(p().muted)
            .when_some(fold, |d, open| d.child(chevron(open)))
            .child(who(by, user))
            .when_some(ago(when), |d, when| d.child(when))
            .when_some(decision, |d, (kind, state)| d.child(decision_chip(&kind, state)))
            .when_some(step, |d, (latest, (at, on_ticket))| {
                let colour = if latest { p().accent } else { p().muted };
                // When its agent resumes, as it said: a time still to come,
                // and (the live step only) another ticket's move.
                let short = at.as_deref().and_then(crate::status::resume_short);
                let on_ticket = on_ticket.filter(|_| latest);
                let tip = [
                    short.is_some().then(|| at.as_deref().and_then(crate::status::resume_full)).flatten().map(|t| format!("the agent resumes at {t}")),
                    on_ticket.map(|n| format!("{} when #{n} moves (a reply, a decision, a close)", if short.is_some() { "or" } else { "the agent resumes" })),
                ]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join(", ");
                d.child(div().text_color(colour).child(icons::labelled(Icon::Step, colour, 12., "step"))).when(
                    short.is_some() || on_ticket.is_some(),
                    |d| {
                        d.child(
                            div()
                                .id(("resume", id))
                                .flex()
                                .gap_1()
                                .text_color(colour)
                                .child("resumes")
                                .children(short.clone())
                                .children(on_ticket.map(|n| {
                                    div()
                                        .id(("resume-on", id))
                                        .flex()
                                        .gap_1()
                                        .child(if short.is_some() { "or on" } else { "on" })
                                        .child(
                                            ticketref::link(("resume-ticket", id), n, None),
                                        )
                                }))
                                .tip(tip),
                        )
                    },
                )
            })
            .when(pending, |d| d.child(pill("to moderate", p().warning)))
            .when(fold.is_some(), |d| {
                d.rounded_sm()
                    .cursor_pointer()
                    .hover(|d| d.bg(p().hover))
                    .tip(if fold == Some(true) { "fold" } else { "unfold" })
                    .on_click(cx.listener(move |panel, _, _, cx| panel.toggle_fold(id, cx)))
            })
    }
}

impl Render for TicketPanel {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let _timing = crate::stats::Timing::new("panel");
        let header = div()
            .flex()
            .items_center()
            .gap_2()
            .h(px(36.))
            .px_3()
            .border_b_1()
            .border_color(p().border)
            // First, the chevron that folds the panel towards the window's edge.
            .child(
                buttons::icon("collapse", "›", buttons::hint(cx, "Fold the ticket panel", "panel.toggle"))
                    .on_click(cx.listener(|_, _, _, cx| cx.emit(CollapsePanel))),
            )
            // A ticket open: its project's name and badges, as the list has
            // them (the way back is above its title).
            .when(self.detail.is_some(), |d| {
                d.children(self.scope.as_ref().map(|s| div().font_weight(FontWeight::BOLD).child(s.project.clone())))
                    .children(self.scope.is_some().then(|| crate::shell::Alerts::of(self.tickets.iter(), self.critical).badges("panel-title")))
            })
            .when(self.detail.is_none(), |d| {
                d.child(div().font_weight(FontWeight::BOLD).child(
                    self.scope
                        .as_ref()
                        .map(|s| s.project.clone())
                        .unwrap_or_else(|| "Tickets".into()),
                ))
                .when(self.scope.as_ref().is_some_and(|s| s.sessionless), |d| {
                    d.child(div().text_xs().text_color(p().muted).child("no session open"))
                })
                // What the project's tickets ask of you, as its row in the
                // sessions list counts it: the critical one, decisions, unread.
                .when(self.scope.is_some(), |d| {
                    d.child(crate::shell::Alerts::of(self.tickets.iter(), self.critical).badges("panel-title"))
                })
            })
            .child(div().flex_1())
            .child(
                div()
                    .text_xs()
                    .text_color(p().muted)
                    .child(format!("as {}", self.aiball.user)),
            )
            .child(
                buttons::chip("new-ticket", "+ New")
                    .text_xs()
                    .text_color(p().accent)
                    .tip(buttons::hint(cx, "A new ticket", "ticket.new"))
                    .on_click(cx.listener(|panel, _, _, cx| {
                        let project = panel.scope.as_ref().map(|s| s.project.clone());
                        crate::bus::emit(cx, crate::bus::Signal::AskNewTicket { project, parent: None })
                    })),
            )
            .child(buttons::separator())
            .child(
                buttons::icon("full-list", "⤢", buttons::hint(cx, "The ticket list, full screen", "list.full"))
                    .on_click(cx.listener(|_, _, _, cx| cx.emit(OpenFullList))),
            );
        let content = match &self.detail {
            Some(detail) => self.detail(detail, cx),
            None => self.list(cx),
        };
        let full = self.is_full();
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(if full { p().bg } else { p().surface })
            .text_sm()
            .when(!full, |d| d.child(header))
            .child(content)
    }
}

fn hint(text: impl Into<SharedString>) -> impl IntoElement {
    div().p_3().text_color(p().muted).child(text.into())
}

/// "you" for the user, else the name.
fn who(name: &str, user: &str) -> String {
    if name == user { "you".into() } else { name.to_string() }
}

/// A folded comment's one line; a click unfolds it, as its head does.
fn folded_line(id: u64, text: String, cx: &mut Context<TicketPanel>) -> impl IntoElement {
    div()
        .id(("folded", id))
        .rounded_sm()
        .text_color(p().muted)
        .truncate()
        .cursor_pointer()
        .hover(|d| d.bg(p().hover))
        .tip("unfold")
        .child(text)
        .on_click(cx.listener(move |panel, _, _, cx| panel.toggle_fold(id, cx)))
}

/// What folds leads with ▾ (open) or ▸ (folded), in the accordions'
/// style: the same glyphs, colour and width.
fn chevron(open: bool) -> impl IntoElement {
    div().w(px(10.)).flex_none().text_color(p().accent).child(if open { "▾" } else { "▸" })
}

/// Coloured when it waits on you, muted otherwise; a step always coloured.
pub(crate) fn glyph_colour(glyph: Glyph, yours: bool) -> Hsla {
    // A rejection is news for everyone, not only for whose turn it is.
    if glyph == Glyph::Rejected {
        return p().danger;
    }
    // A step is the agent's own marker, never the reader's turn: blue, or
    // amber once it went quiet — as aiball's web UI shows them.
    match glyph {
        Glyph::Step => return p().accent,
        Glyph::StalledStep => return p().warning,
        _ => {}
    }
    if !yours {
        return p().muted;
    }
    match glyph {
        Glyph::Escalation | Glyph::Rejected => p().danger,
        Glyph::Plan | Glyph::StalledStep => p().warning,
        Glyph::Resolution => p().success,
        Glyph::Step => p().accent,
        Glyph::Wontfix | Glyph::ClosedResolved | Glyph::Closed => p().muted,
    }
}

pub(crate) fn stripe_colour(state: &RowState) -> Hsla {
    state
        .glyph
        .filter(|_| state.stripe != Stripe::Neutral)
        .map(|g| glyph_colour(g, state.turn == Turn::You))
        .unwrap_or(p().border)
}

/// A row's left edge: the construction tape of a ticket waiting for
/// moderation — nothing goes on until it is let through —, else whose
/// turn it is ([`stripe`]).
pub(crate) fn row_edge(state: &RowState) -> AnyElement {
    if state.band == Band::Moderate {
        hazard().into_any_element()
    } else {
        stripe(state.stripe, stripe_colour(state)).into_any_element()
    }
}

/// Yellow and black diagonal bands, as a construction site's tape.
pub(crate) fn hazard() -> impl IntoElement {
    const WIDTH: f32 = 5.;
    const BAND: f32 = 4.;
    canvas(
        |_, _, _| {},
        |bounds, _, window, _| {
            window.with_content_mask(Some(ContentMask { bounds }), |window| {
                window.paint_quad(fill(bounds, gpui_kit::rgb(0x1c1c1c)));
                let (left, right) = (bounds.left(), bounds.right());
                let mut y = bounds.top() - px(WIDTH);
                while y < bounds.bottom() + px(WIDTH) {
                    let mut path = PathBuilder::fill();
                    path.move_to(point(left, y + px(WIDTH)));
                    path.line_to(point(right, y));
                    path.line_to(point(right, y + px(BAND)));
                    path.line_to(point(left, y + px(WIDTH + BAND)));
                    path.close();
                    if let Ok(path) = path.build() {
                        window.paint_path(path, gpui_kit::rgb(0xf2c230));
                    }
                    y += px(2. * BAND);
                }
            });
        },
    )
    .w(px(WIDTH))
    // Wider than a stripe, which it stands for: the row's text stays put.
    .mr(px(3. - WIDTH))
    .flex_none()
    .rounded_sm()
}

/// Whose turn: coloured when a decision waits on you (solid when it is the
/// last message, dashed when the talk went on), neutral when an agent
/// answered you, nothing when the ball is theirs.
pub(crate) fn stripe(stripe: Stripe, colour: Hsla) -> impl IntoElement {
    div()
        .w(px(3.))
        .flex_none()
        .rounded_sm()
        .map(|d| match stripe {
            Stripe::Solid | Stripe::Neutral => d.bg(colour),
            Stripe::Dashed => d.border_l_3().border_dashed().border_color(colour),
            // You spoke last: a thin dotted line, so waiting shows too.
            Stripe::Waiting => d.border_l_1().border_dashed().border_color(p().muted.opacity(0.6)),
            Stripe::None => d,
        })
}

/// The tallest a thread's image is drawn full screen.
const FULL_PICTURE_HEIGHT: f32 = 900.;

impl TicketPanel {
    /// A text of the thread: its markdown, and its images alone on their
    /// line drawn by tvty — a thumbnail in the panel, the column's width full
    /// screen —,
    /// a click opening the viewer.
    /// The images of the reply being written, read for its preview (a
    /// capture just pasted is not in the thread's yet).
    fn load_preview_images(&mut self, cx: &mut Context<Self>) {
        let text = self.reply.read(cx).value().to_string();
        let (aiball, images) = (self.aiball.clone(), self.images.clone());
        cx.spawn(async move |this, cx| {
            let came = cx.background_executor().spawn(async move { crate::images::load(&text, &aiball, &images) }).await;
            if came {
                let _ = this.update(cx, |_, cx| cx.notify());
            }
        })
        .detach();
    }

    fn rich_text(&self, id: String, text: &str, cx: &mut Context<Self>) -> Div {
        let mut col = div().flex().flex_col().gap_1();
        for (i, segment) in crate::images::segments(text, &self.images).into_iter().enumerate() {
            col = match segment {
                crate::images::Segment::Text(md) => col.child(
                    TextView::markdown(SharedString::from(format!("{id}-{i}")), crate::ui::ticketref::linkify(&md))
                        .selectable(true)
                        .on_link_click(crate::ui::ticketref::on_link),
                ),
                crate::images::Segment::Note(why) => col.child(div().text_xs().italic().text_color(p().muted).child(format!("({why})"))),
                crate::images::Segment::Pictures(pictures) => {
                    let mut row = div().w_full().flex().flex_wrap().gap_2();
                    for (j, picture) in pictures.into_iter().enumerate() {
                        let (width, height) = (picture.width.max(1) as f32, picture.height.max(1) as f32);
                        let reference = picture.reference.clone();
                        let frame = div()
                            .id(SharedString::from(format!("{id}-pic-{i}-{j}")))
                            .relative()
                            .flex_none()
                            .rounded_sm()
                            .overflow_hidden()
                            .border_1()
                            .border_color(p().border)
                            .cursor_pointer()
                            .hover(|d| d.border_color(p().accent));
                        let frame = if self.full {
                            // Full screen: the column's width, never beyond
                            // the real size nor taller than FULL_PICTURE_HEIGHT.
                            let max_w = width.min(FULL_PICTURE_HEIGHT * width / height);
                            frame
                                .w_full()
                                .max_w(px(max_w))
                                .aspect_ratio(width / height)
                                .child(img(ImageSource::Image(picture.image.clone())).size_full())
                        } else {
                            // A thumbnail keeps the image's proportions.
                            let scale = (160. / width).min(100. / height).min(1.);
                            frame.child(img(ImageSource::Image(picture.image.clone())).w(px(width * scale)).h(px(height * scale)))
                        };
                        row = row.child(
                            frame
                                .when(!self.full, |d| {
                                    d.child(
                                        div()
                                            .absolute()
                                            .bottom_0()
                                            .right_0()
                                            .px_1()
                                            .rounded_tl_sm()
                                            .bg(p().bg.opacity(0.7))
                                            .text_xs()
                                            .text_color(p().text)
                                            .child("⤢"),
                                    )
                                })
                                .on_click(cx.listener(move |panel, _, _, cx| panel.open_picture(&reference, cx))),
                        );
                    }
                    col.child(row)
                }
            };
        }
        col
    }

    /// Opens the viewer on an image, with the thread's others to go through.
    fn open_picture(&mut self, reference: &str, cx: &mut Context<Self>) {
        let Some(thread) = self.detail.as_ref().and_then(|d| d.thread.as_ref()) else { return };
        let texts = thread.ticket.body.iter().chain(thread.comments.iter().filter_map(|c| c.body.as_ref()));
        let mut pictures: Vec<crate::images::Picture> = Vec::new();
        for text in texts {
            for picture in crate::images::pictures(text, &self.images) {
                if !pictures.iter().any(|p| p.reference == picture.reference) {
                    pictures.push(picture);
                }
            }
        }
        let index = pictures.iter().position(|p| p.reference == reference).unwrap_or(0);
        crate::bus::emit(cx, crate::bus::Signal::OpenPictures { pictures, index });
    }
}

/// aiball's comment count: a bubble and the number, as a chat shows who
/// spoke — its point on the left and bright when someone else spoke last
/// (blue while unread), on the right and discreet when the user did; a
/// clock and the number while comments wait for moderation.
pub(crate) fn comment_count(ticket: &TicketRow, user: &str) -> Option<Stateful<Div>> {
    if ticket.pending_comment_count > 0 {
        let n = ticket.pending_comment_count;
        return Some(
            icons::labelled(Icon::PendingComments, p().warning, 12., n.to_string())
                .text_color(p().warning)
                .id("comments")
                .tip(format!("{n} comment{} waiting for moderation", if n == 1 { "" } else { "s" })),
        );
    }
    if ticket.comment_count == 0 && ticket.last_speaker.is_none() {
        return None;
    }
    let mine = ticket.last_speaker.as_deref() == Some(user);
    let (colour, tip) = if ticket.unread {
        (p().accent, "unread: something new on it for you")
    } else if mine {
        (p().muted.opacity(0.55), "you spoke last")
    } else {
        (p().text, "someone else spoke last")
    };
    let n = ticket.comment_count;
    Some(
        icons::labelled(if mine && !ticket.unread { Icon::CommentsMine } else { Icon::Comments }, colour, 12., n.to_string())
            .text_color(colour)
            .id("comments")
            .tip(format!("{n} comment{} — {tip}", if n == 1 { "" } else { "s" })),
    )
}

/// Who holds the ticket, and the flame when it was active lately.
pub(crate) fn holder_chip(ticket: &TicketRow) -> Option<Stateful<Div>> {
    let holder = ticket.holder()?.to_string();
    let tip = if ticket.hot { format!("held by {holder}, active on it lately") } else { format!("held by {holder}") };
    Some(if ticket.hot {
        icons::labelled(Icon::Hot, p().warning, 12., holder).id("holder").tip(tip)
    } else {
        div().child(holder).id("holder").tip(tip)
    })
}

/// The project's critical ticket: how many open tickets it holds.
pub(crate) fn critical_chip(ticket: &TicketRow) -> Option<Stateful<Div>> {
    let critical = ticket.critical.as_ref()?;
    let holds = critical.holds;
    let quiet = critical.quiet();
    Some(
        icons::pill(Icon::Critical, quiet.map_or(holds.to_string(), |q| format!("{holds} · {q}")), p().danger)
            .id("critical")
            .tip(format!(
                "the project's critical ticket: it holds {holds} open ticket{}{}",
                if holds == 1 { "" } else { "s" },
                quiet.map(|q| format!(", quiet for {q}")).unwrap_or_default()
            )),
    )
}

/// The priority's glyph, when it is not normal.
pub(crate) fn priority_chip(ticket: &TicketRow, size: f32) -> Option<Stateful<Div>> {
    let priority = ticket.priority.as_deref()?;
    let icon = icons::priority(priority)?;
    Some(div().child(icons::icon(icon, icons::priority_colour(priority), size)).id("priority").tip(format!("priority: {priority}")))
}

/// The state glyph, saying what it means — a step, when its agent resumes.
pub(crate) fn glyph_chip(glyph: Glyph, colour: Hsla, size: f32, ticket: &TicketRow) -> Stateful<Div> {
    let resume = ticket
        .step_resume_at
        .as_deref()
        .filter(|at| glyph == Glyph::Step && crate::status::resume_short(at).is_some())
        .and_then(crate::status::resume_full);
    let tip = match resume {
        Some(at) => format!("step — the agent resumes at {at}"),
        None => glyph.meaning().to_string(),
    };
    div().child(icons::icon(icons::of_glyph(glyph), colour, size)).id("glyph").tip(tip)
}

/// A decision, as a chip: the list's glyphs, and "superseded" when a newer
/// decision replaced it.
fn decision_chip(kind: &str, state: DecisionState) -> AnyElement {
    let noun = reading::kind_noun(kind);
    match state {
        DecisionState::Pending => icons::pill(icons::of_kind(kind), format!("{noun} · pending"), p().warning).into_any_element(),
        DecisionState::Accepted => icons::pill(Icon::Resolution, format!("{noun} accepted"), p().success).into_any_element(),
        DecisionState::Rejected => icons::pill(Icon::Rejected, format!("{noun} rejected"), p().danger).into_any_element(),
        DecisionState::Superseded => div()
            .flex_none()
            .px_1p5()
            .rounded_sm()
            .text_xs()
            .border_1()
            .border_color(p().border)
            .text_color(p().muted)
            .child(format!("{noun} · superseded"))
            .into_any_element(),
    }
}


/// `12345` → `12.3k`.
pub(crate) fn count(n: u64) -> String {
    match n {
        0..1000 => n.to_string(),
        1000..1_000_000 => format!("{:.1}k", n as f64 / 1e3),
        _ => format!("{:.1}M", n as f64 / 1e6),
    }
}

pub fn pill(text: impl Into<SharedString>, color: Hsla) -> impl IntoElement {
    div()
        .flex_none()
        .px_1p5()
        .rounded_sm()
        .text_xs()
        .bg(color)
        .text_color(crate::theme::on(color))
        .child(text.into())
}

pub fn dot(color: Hsla) -> impl IntoElement {
    div().flex_none().mt_1p5().size(px(7.)).rounded_full().bg(color)
}

/// `2026-09-24T13:01:55.681Z` → `3m`, `2h`, `5d` ago.
pub(crate) fn ago(when: &str) -> Option<String> {
    let then = crate::status::parse_time(when)?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();
    let seconds = now.saturating_sub(then);
    Some(match seconds {
        0..60 => "now".into(),
        60..3600 => format!("{}m", seconds / 60),
        3600..86400 => format!("{}h", seconds / 3600),
        _ => format!("{}d", seconds / 86400),
    })
}

/// The tickets sunk in an agent's backlog at `now` (seconds since the
/// epoch), and until when.
fn sunk_of(backlog: &crate::aiball::AgentBacklog, now: u64) -> HashMap<u64, String> {
    backlog
        .rows
        .iter()
        .filter_map(|r| {
            let until = r.backlog_cooled_until.clone()?;
            (crate::status::parse_time(&until)? > now).then_some((r.id, until))
        })
        .collect()
}

#[cfg(test)]
mod sunk_tests {
    use super::sunk_of;
    use crate::aiball::AgentBacklog;
    use serde_json::json;

    #[test]
    fn only_a_pause_still_running_marks_a_ticket_sunk() {
        let backlog: AgentBacklog = serde_json::from_value(json!({ "rows": [
            { "id": 1, "project": "p", "title": "a", "backlog_tier": 1, "backlog_cooled_until": "2026-09-29T15:00:00.000Z" },
            { "id": 2, "project": "p", "title": "b", "backlog_tier": 1, "backlog_cooled_until": "2026-09-29T13:00:00.000Z" },
            { "id": 3, "project": "p", "title": "c", "backlog_tier": 3, "backlog_cooled_until": null },
        ] }))
        .unwrap();
        let now = crate::status::parse_time("2026-09-29T14:00:00.000Z").unwrap();
        let sunk = sunk_of(&backlog, now);
        assert_eq!(sunk.keys().copied().collect::<Vec<_>>(), vec![1]);
    }
}
