//! The ticket list, full screen: over the window, the scope and the counters
//! on the left third — projects, bands, filters — and the list on the rest,
//! each row with all it has to say. The panel beside the terminal stays the
//! compact list; this one is for looking over the board.

use crate::ui::Named as _;
use std::collections::{BTreeSet, HashMap};

use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::scroll::ScrollableElement as _;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::component::button::ButtonVariants as _;
use gpui_kit::*;

use crate::aiball::{Aiball, TicketRow};
use crate::icons::{self, Icon};
use crate::ui::buttons::{self, Look as _};
use crate::panel::{ago, count, glyph_colour};
use crate::rowstate::{self, Band, RowState, Turn};
use crate::shell::Alerts;
use crate::tip::Tip as _;
use crate::theme::p;

/// Closed tickets read per project when the list shows them too.
const ALL_LIMIT: usize = 300;

const BANDS: [Band; 5] = [Band::AgentOnIt, Band::Moderate, Band::Decide, Band::Open, Band::Closed];

/// How the list is ordered: one list, not bands — the bands filter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Sort {
    Activity,
    Turn,
    Priority,
    Created,
    Number,
}

impl Sort {
    const ALL: [Sort; 5] = [Sort::Activity, Sort::Turn, Sort::Priority, Sort::Created, Sort::Number];

    fn title(self) -> &'static str {
        match self {
            Sort::Activity => "Last activity",
            Sort::Turn => "Whose turn (bands)",
            Sort::Priority => "Priority",
            Sort::Created => "Created",
            Sort::Number => "Number",
        }
    }
}

fn priority_rank(priority: Option<&str>) -> u8 {
    match priority {
        Some("urgent") => 3,
        Some("high") => 2,
        Some("low") => 0,
        _ => 1,
    }
}

/// The user leaves the list.
pub struct CloseFullList;


pub struct FullList {
    aiball: Aiball,
    /// The projects on the board, their open tickets and critical ticket.
    projects: Vec<String>,
    open: HashMap<String, Vec<TicketRow>>,
    critical: HashMap<String, u64>,
    /// `None`: every project.
    scope: Option<String>,
    band: Option<Band>,
    sort: Sort,
    /// The sort's natural order (most recent, most pressing, highest
    /// first), or its reverse.
    reversed: bool,
    /// Closed tickets too, read on demand.
    with_closed: bool,
    /// Only what has something new for the user.
    unread_only: bool,
    all: HashMap<String, Vec<TicketRow>>,
    loading: bool,
    tags: BTreeSet<String>,
    search: Entity<InputState>,
    error: Option<String>,
    /// Rows chosen for a bulk action (ctrl+click, shift+click), in order;
    /// shift+click extends from the anchor.
    selected: Vec<(String, u64)>,
    anchor: Option<(String, u64)>,
    /// The rows shown, in order, as last drawn: what shift+click and
    /// ctrl+a range over.
    shown: Vec<(String, u64)>,
    /// A bulk action asked, waiting for its confirmation.
    confirm: Option<crate::bulk::Action>,
    /// A bulk action on its way.
    working: bool,
}

impl EventEmitter<CloseFullList> for FullList {}


impl FullList {
    pub fn new(aiball: Aiball, scope: Option<String>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        // The side column's width, dragged here or on another full page.
        cx.observe_global::<crate::sidecol::SideWidth>(|_, cx| cx.notify()).detach();
        let search = cx.new(|cx| InputState::new(window, cx).placeholder("Search titles…"));
        // The rows a notification is about shine while it is up.
        cx.subscribe(&crate::bus::bus(cx), |_, _, signal: &crate::bus::Signal, cx| {
            if matches!(signal, crate::bus::Signal::Notices) {
                cx.notify();
            }
        })
        .detach();
        cx.subscribe(&search, |_, _, event: &InputEvent, cx| {
            if matches!(event, InputEvent::Change) {
                cx.notify();
            }
        })
        .detach();
        Self {
            aiball,
            projects: Vec::new(),
            open: HashMap::new(),
            critical: HashMap::new(),
            scope,
            band: None,
            sort: Sort::Activity,
            reversed: false,
            with_closed: false,
            unread_only: false,
            all: HashMap::new(),
            loading: false,
            tags: BTreeSet::new(),
            search,
            error: None,
            selected: Vec::new(),
            anchor: None,
            shown: Vec::new(),
            confirm: None,
            working: false,
        }
    }

    /// The board as last read.
    pub fn set_board(
        &mut self,
        aiball: &Aiball,
        projects: Vec<String>,
        open: HashMap<String, Vec<TicketRow>>,
        critical: HashMap<String, u64>,
        cx: &mut Context<Self>,
    ) {
        self.aiball = aiball.clone();
        if self.projects != projects || self.open != open || self.critical != critical {
            self.projects = projects;
            self.open = open;
            self.critical = critical;
            if self.with_closed {
                self.read_all(cx);
            }
            cx.notify();
        }
    }

    fn scoped(&self) -> Vec<String> {
        match &self.scope {
            Some(project) => vec![project.clone()],
            None => self.projects.clone(),
        }
    }

    // ── Selection and bulk actions ─────────────────────────────────────

    pub fn selecting(&self) -> bool {
        !self.selected.is_empty()
    }

    pub fn clear_selection(&mut self, cx: &mut Context<Self>) {
        self.selected.clear();
        self.anchor = None;
        self.confirm = None;
        cx.notify();
    }

    /// Every row shown (its filters applied).
    pub fn select_all(&mut self, cx: &mut Context<Self>) {
        self.selected = self.shown.clone();
        cx.notify();
    }

    pub fn search_focused(&self, window: &Window, cx: &App) -> bool {
        self.search.read(cx).focus_handle(cx).is_focused(window)
    }

    fn toggle_selected(&mut self, key: (String, u64), cx: &mut Context<Self>) {
        match self.selected.iter().position(|k| *k == key) {
            Some(at) => {
                self.selected.remove(at);
            }
            None => self.selected.push(key.clone()),
        }
        self.anchor = Some(key);
        self.confirm = None;
        cx.notify();
    }

    /// From the anchor to `key`, as the rows are shown.
    fn select_to(&mut self, key: (String, u64), cx: &mut Context<Self>) {
        let at = |k: &(String, u64)| self.shown.iter().position(|s| s == k);
        let (Some(from), Some(to)) = (self.anchor.as_ref().and_then(at), at(&key)) else {
            return self.toggle_selected(key, cx);
        };
        let (low, high) = (from.min(to), from.max(to));
        for k in &self.shown[low..=high] {
            if !self.selected.contains(k) {
                self.selected.push(k.clone());
            }
        }
        self.confirm = None;
        cx.notify();
    }

    /// The selected rows, as last read.
    fn selected_rows(&self) -> Vec<TicketRow> {
        let source = if self.with_closed { &self.all } else { &self.open };
        self.selected
            .iter()
            .filter_map(|(project, id)| source.get(project)?.iter().find(|t| t.id == *id).cloned())
            .collect()
    }

    /// Does `action` on the selection, off the UI thread; one notification
    /// says how it went.
    fn run_bulk(&mut self, action: crate::bulk::Action, cx: &mut Context<Self>) {
        let rows = self.selected_rows();
        let aiball = self.aiball.clone();
        self.confirm = None;
        self.working = true;
        cx.notify();
        cx.spawn(async move |this, cx| {
            let (line, failed) = cx.background_executor().spawn(async move { crate::bulk::run(&aiball, action, &rows) }).await;
            let _ = this.update(cx, |list, cx| {
                list.working = false;
                list.selected.clear();
                list.anchor = None;
                let activity = if failed {
                    crate::activity::Activity::failed(None, action.label(), line)
                } else {
                    crate::activity::Activity::done(None, line)
                };
                crate::activity::publish(cx, activity);
                cx.notify();
            });
        })
        .detach();
    }

    /// The left column while rows are selected: what can be done to them.
    fn bulk_side(&self, cx: &mut Context<Self>) -> Stateful<Div> {
        use crate::bulk::Action;
        let rows = self.selected_rows();
        let refs: Vec<&TicketRow> = rows.iter().collect();
        let link = |id: &'static str, label: &'static str| buttons::link(id, label).text_xs();
        let mut side = div()
            .named("bulk-side")
            .flex()
            .flex_col()
            .gap_1()
            .p_4()
            .child(div().text_lg().font_weight(FontWeight::BOLD).child(format!("{} selected", self.selected.len())))
            .child(
                div()
                    .flex()
                    .gap_2()
                    .pb_2()
                    .child(link("bulk-all", "Select all shown").on_click(cx.listener(|list, _, _, cx| list.select_all(cx))))
                    .child(link("bulk-clear", "Clear · Esc").on_click(cx.listener(|list, _, _, cx| list.clear_selection(cx)))),
            )
            .child(group("Actions"))
            .child(
                div()
                    .pb_1()
                    .text_xs()
                    .text_color(p().muted)
                    .child("Each acts on the chosen tickets it fits: how many, on the right."),
            );
        if self.working {
            return side.child(div().text_color(p().muted).child("Working…"));
        }
        for action in Action::ALL {
            let count = action.count(&refs);
            let asked = self.confirm == Some(action);
            let row = div()
                .named(SharedString::from(format!("bulk-{}", action.label())))
                .flex()
                .items_center()
                .gap_2()
                .px_2()
                .py_1()
                .rounded_md()
                .tip(action.about())
                .child(div().flex_1().child(action.label()))
                .child(div().text_xs().text_color(p().muted).child(format!("{count} of {}", refs.len())));
            let row = if count == 0 {
                row.text_color(p().muted.opacity(0.6))
            } else {
                row.text_color(p().text).cursor_pointer().hover(|d| d.bg(p().hover)).on_click(cx.listener(move |list, _, _, cx| {
                    if action.confirmed() {
                        list.confirm = Some(action);
                        cx.notify();
                    } else {
                        list.run_bulk(action, cx);
                    }
                }))
            };
            side = side.child(row);
            if asked {
                side = side.child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .px_2()
                        .pb_2()
                        .text_sm()
                        .child(format!("{} {count} ticket{}?", action.label(), if count == 1 { "" } else { "s" }))
                        .child(
                            buttons::answer("bulk-confirm", action.label())
                                .warning()
                                .on_click(cx.listener(move |list, _, _, cx| list.run_bulk(action, cx))),
                        )
                        .child(buttons::secondary("bulk-cancel", "Cancel").on_click(cx.listener(|list, _, _, cx| {
                            list.confirm = None;
                            cx.notify();
                        }))),
                );
            }
        }
        side
    }

    /// Shows only what has something new for the user (or everything).
    pub fn set_unread_only(&mut self, unread_only: bool, cx: &mut Context<Self>) {
        self.unread_only = unread_only;
        cx.notify();
    }

    pub fn set_scope(&mut self, scope: Option<String>, cx: &mut Context<Self>) {
        self.scope = scope;
        self.band = None;
        if self.with_closed {
            self.read_all(cx);
        }
        cx.notify();
    }

    fn set_with_closed(&mut self, with_closed: bool, cx: &mut Context<Self>) {
        self.with_closed = with_closed;
        if with_closed {
            self.read_all(cx);
        }
        cx.notify();
    }

    /// Reads the scope's tickets, closed ones too, off the UI thread.
    fn read_all(&mut self, cx: &mut Context<Self>) {
        let aiball = self.aiball.clone();
        let projects = self.scoped();
        self.loading = true;
        cx.spawn(async move |this, cx| {
            let read = cx
                .background_executor()
                .spawn(async move {
                    let mut all = HashMap::new();
                    for project in projects {
                        all.insert(project.clone(), aiball.all_tickets(&project, ALL_LIMIT)?);
                    }
                    anyhow::Ok(all)
                })
                .await;
            let _ = this.update(cx, |list, cx| {
                list.loading = false;
                match read {
                    Ok(all) => {
                        list.all.extend(all);
                        list.error = None;
                    }
                    Err(error) => list.error = Some(format!("{error:#}")),
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// The scope's rows, with their reading, before the band and the text
    /// and tag filters.
    fn rows(&self) -> Vec<(RowState, &TicketRow)> {
        let user = self.aiball.user.as_str();
        let source = if self.with_closed { &self.all } else { &self.open };
        let mut rows: Vec<(RowState, &TicketRow)> = self
            .scoped()
            .iter()
            .filter_map(|project| source.get(project))
            .flatten()
            .map(|t| (rowstate::of(t, user), t))
            .collect();
        let recent = |a: &TicketRow, b: &TicketRow| b.last_activity.cmp(&a.last_activity);
        rows.sort_by(|(sa, a), (sb, b)| {
            let order = match self.sort {
                Sort::Activity => recent(a, b),
                Sort::Turn => sa.band.rank().cmp(&sb.band.rank()).then_with(|| recent(a, b)),
                Sort::Priority => priority_rank(b.priority.as_deref())
                    .cmp(&priority_rank(a.priority.as_deref()))
                    .then_with(|| recent(a, b)),
                Sort::Created => b.created_at.cmp(&a.created_at),
                Sort::Number => b.id.cmp(&a.id),
            };
            if self.reversed { order.reverse() } else { order }
        });
        rows
    }

    /// The critical ticket of each of `projects`, what holds back the most
    /// first, then what has been quiet the longest.
    fn criticals(&self, projects: &[String]) -> Vec<&TicketRow> {
        let mut found: Vec<&TicketRow> = projects
            .iter()
            .filter_map(|project| {
                let id = *self.critical.get(project)?;
                self.open.get(project)?.iter().find(|t| t.id == id)
            })
            .collect();
        let rank = |t: &TicketRow| t.critical.as_ref().map_or((0, 0), |c| (c.holds, c.quiet_minutes()));
        found.sort_by(|a, b| rank(b).cmp(&rank(a)));
        found
    }

    fn matches(&self, ticket: &TicketRow, query: &str) -> bool {
        (query.is_empty() || ticket.title.to_lowercase().contains(query))
            && self.tags.iter().all(|tag| ticket.tags.iter().any(|t| &t.name == tag))
    }

    // ── The left third: scope, counters, filters ────────────────────────

    fn side(&self, rows: &[(RowState, &TicketRow)], query: &str, cx: &mut Context<Self>) -> Stateful<Div> {
        let mut side = div()
            .named("full-list-side")
            .flex()
            .flex_col()
            .gap_0p5()
            .p_3()
            .text_sm()
            .child(group("Projects"));
        let mut all_alerts = Alerts::of(
            self.projects.iter().filter_map(|p| self.open.get(p)).flatten(),
            None,
        );
        all_alerts.critical = self.criticals(&self.projects).first().map(|t| t.id);
        side = side.child(self.choice(
            "scope-all",
            "All projects".into(),
            self.scope.is_none(),
            Some(all_alerts.badges("full-all").into_any_element()),
            cx.listener(|list, _, _, cx| list.set_scope(None, cx)),
        ));
        for project in &self.projects {
            let alerts = Alerts::of(
                self.open.get(project).into_iter().flatten(),
                self.critical.get(project).copied(),
            );
            let pick = project.clone();
            side = side.child(self.choice(
                SharedString::from(format!("scope-{project}")),
                project.clone().into(),
                self.scope.as_ref() == Some(project),
                Some(alerts.badges(format!("full-{project}")).into_any_element()),
                cx.listener(move |list, _, _, cx| list.set_scope(Some(pick.clone()), cx)),
            ));
        }

        side = side.child(group("Bands"));
        for band in BANDS {
            if band == Band::Closed && !self.with_closed {
                continue;
            }
            let n = rows.iter().filter(|(s, t)| s.band == band && self.matches(t, query)).count();
            side = side.child(self.choice(
                SharedString::from(format!("band-{band:?}")),
                band.title().into(),
                self.band == Some(band),
                Some(div().text_xs().text_color(p().muted).child(n.to_string()).into_any_element()),
                cx.listener(move |list, _, _, cx| {
                    list.band = if list.band == Some(band) { None } else { Some(band) };
                    cx.notify();
                }),
            ));
        }

        side = side.child(group("Sort"));
        for sort in Sort::ALL {
            let chosen = self.sort == sort;
            let arrow = if !chosen { "" } else if self.reversed { "↑" } else { "↓" };
            side = side.child(self.choice(
                SharedString::from(format!("sort-{sort:?}")),
                sort.title().into(),
                chosen,
                Some(div().text_xs().text_color(p().muted).child(arrow).into_any_element()),
                cx.listener(move |list, _, _, cx| {
                    if list.sort == sort {
                        list.reversed = !list.reversed;
                    } else {
                        list.sort = sort;
                        list.reversed = false;
                    }
                    cx.notify();
                }),
            ));
        }

        side = side
            .child(group("Filters"))
            .child(
                div()
                    .flex()
                    .gap_1()
                    .child(self.toggle("open-only", "Open", !self.with_closed, cx.listener(|list, _, _, cx| {
                        list.set_with_closed(false, cx)
                    })))
                    .child(self.toggle("with-closed", "All", self.with_closed, cx.listener(|list, _, _, cx| {
                        list.set_with_closed(true, cx)
                    })))
                    .child(self.toggle("unread-only", "Unread", self.unread_only, cx.listener(|list, _, _, cx| {
                        list.set_unread_only(!list.unread_only, cx)
                    })))
                    .when(self.loading, |d| d.child(div().text_xs().text_color(p().muted).child("reading…"))),
            )
            .child(div().pt_1().child(Input::new(&self.search)));

        // The tags of the scope, to narrow to those carrying them all.
        let tags: BTreeSet<&str> = rows.iter().flat_map(|(_, t)| t.tags.iter().map(|t| t.name.as_str())).collect();
        if !tags.is_empty() {
            let mut chips = div().flex().flex_wrap().gap_1().pt_1();
            for tag in tags {
                let on = self.tags.contains(tag);
                let name = tag.to_string();
                chips = chips.child(
                    buttons::chip(SharedString::from(format!("tag-{tag}")), tag.to_string())
                        .text_xs()
                        .when(!on, |d| d.text_color(p().muted))
                        .chosen(on)
                        .on_click(cx.listener(move |list, _, _, cx| {
                            if !list.tags.remove(&name) {
                                list.tags.insert(name.clone());
                            }
                            cx.notify();
                        })),
                );
            }
            side = side.child(chips);
        }
        side.when_some(self.error.clone(), |d, error| d.child(div().pt_2().text_color(p().danger).child(error)))
    }

    fn choice(
        &self,
        id: impl Into<ElementId>,
        label: SharedString,
        chosen: bool,
        trailing: Option<AnyElement>,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Stateful<Div> {
        div()
            .named(id)
            .flex()
            .items_center()
            .gap_2()
            .px_2()
            .py_1()
            .rounded_md()
            .cursor_pointer()
            .when(chosen, |d| d.bg(p().active))
            .hover(|d| d.bg(p().hover))
            .child(div().flex_1().min_w_0().truncate().child(label))
            .children(trailing)
            .on_click(on_click)
    }

    fn toggle(
        &self,
        id: &'static str,
        label: &'static str,
        on: bool,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Stateful<Div> {
        buttons::chip(id, label)
            .py_0p5()
            .text_xs()
            .when(!on, |d| d.text_color(p().muted))
            .chosen(on)
            .on_click(on_click)
    }

    // ── The list ────────────────────────────────────────────────────────

    fn row(&self, ticket: &TicketRow, state: RowState, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let yours = state.turn == Turn::You;
        let user = &self.aiball.user;
        let who = |name: &str| if name == user { "you".to_string() } else { name.to_string() };

        let title = div()
            .flex()
            .items_center()
            .gap_2()
            .items_center()
            .child(div().flex_none().text_color(p().muted).child(format!("#{}", ticket.id)))
            .when(self.scope.is_none(), |d| {
                d.child(div().flex_none().text_xs().text_color(p().accent).child(ticket.project.clone()))
            })
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .map(|d| {
                        if ticket.unread {
                            d.font_weight(FontWeight::BOLD).text_color(p().text)
                        } else {
                            d.text_color(p().text.opacity(0.72))
                        }
                    })
                    .child(ticket.title.clone()),
            )
            .when_some(ticket.priority.as_deref().and_then(icons::priority), |d, icon| {
                let priority = ticket.priority.clone().unwrap_or_default();
                d.child(
                    icons::labelled(icon, icons::priority_colour(&priority), 14., priority.clone())
                        .text_xs()
                        .text_color(p().muted)
                        .named("priority")
                        .tip(format!("priority: {priority}")),
                )
            })
            .children(crate::panel::comment_count(ticket, &self.aiball.user).map(|c| c.flex_none().text_xs()))
            .when_some(ticket.last_activity.as_deref().and_then(ago), |d, when| {
                d.child(div().flex_none().text_xs().text_color(p().muted).child(when))
            });

        let mut facts: Vec<String> = Vec::new();
        if state.glyph == Some(crate::rowstate::Glyph::Rejected) {
            facts.push("rejected".into());
        }
        if let Some(intent) = &ticket.intent {
            facts.push(intent.clone());
        }
        if let Some(level) = ticket.level.as_deref().filter(|l| *l != "task") {
            facts.push(level.to_string());
        }
        if let Some(title) = ticket.milestone.as_ref().and_then(|m| m.title.clone()) {
            facts.push(format!("milestone {title}"));
        }
        if let Some(holder) = ticket.holder() {
            let held = if ticket.held.assigned() { "assigned to" } else { "claimed by" };
            facts.push(format!("{held} {}", who(holder)));
        }
        if let Some(speaker) = &ticket.last_speaker {
            facts.push(format!("{} spoke last", who(speaker)));
        }
        if let Some(usage) = &ticket.token_usage {
            let total = usage.tokens_in + usage.tokens_out + usage.cache_w;
            if total > 0 {
                facts.push(format!("{} tok", count(total)));
            }
        }
        if ticket.blocked {
            facts.push("blocked".into());
        }
        if let Some(until) = ticket.postponed_until.as_deref() {
            facts.push(format!("snoozed until {}", until.get(..10).unwrap_or(until)));
        }
        if ticket.has_payload {
            facts.push("payload".into());
        }
        if ticket.scope.as_deref().is_some_and(|s| s != "default") {
            facts.push(ticket.scope.clone().unwrap_or_default());
        }
        facts.push(format!("by {} · {}", who(&ticket.by_agent), ticket.created_at.get(..10).unwrap_or("")));

        let meta = div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap_x_2()
            .text_xs()
            .text_color(p().muted)
            .children(crate::panel::critical_chip(ticket))
            .when(ticket.hot, |d| {
                d.child(div().child(icons::icon(Icon::Hot, p().warning, 12.)).named("hot").tip("an agent was active on it lately"))
            })
            .children(ticket.tags.iter().map(|tag| {
                div().px_1().rounded_sm().border_1().border_color(p().border).child(tag.name.clone())
            }))
            .child(facts.join(" · "));

        let (project, id) = (ticket.project.clone(), ticket.id);
        let lit = crate::notify::lit(cx, id);
        let chosen = self.selected.iter().any(|(p, i)| *p == project && *i == id);
        div()
            .named(SharedString::from(format!("full-{}-{}", ticket.project, ticket.id)))
            // A notification about it is up: it shines.
            .map(|d| crate::notify::halo(d, lit))
            .flex()
            .gap_2()
            .px_3()
            .py_2()
            .border_b_1()
            .border_color(p().border.opacity(0.5))
            .cursor_pointer()
            .hover(|d| d.bg(p().hover))
            .when(chosen, |d| d.bg(p().active).border_l_2().border_color(p().accent))
            .child(crate::panel::row_edge(&state))
            .child(
                div()
                    .w(px(18.))
                    .pt_0p5()
                    .flex_none()
                    .when_some(state.glyph, |d, glyph| {
                        d.child(crate::panel::glyph_chip(glyph, glyph_colour(glyph, yours), 18., ticket))
                    }),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_0p5()
                    .flex_1()
                    .min_w_0()
                    .child(title)
                    .when_some(ticket.snippet.clone().filter(|s| !s.trim().is_empty()), |d, snippet| {
                        d.child(div().text_xs().text_color(p().muted).truncate().child(snippet.replace('\n', " ")))
                    })
                    .child(meta),
            )
            // ctrl+click chooses it for a bulk action, shift+click the rows
            // up to it; a click opens it.
            .on_click(cx.listener(move |list, event: &ClickEvent, _, cx| {
                let m = event.modifiers();
                if m.control || m.platform {
                    list.toggle_selected((project.clone(), id), cx);
                } else if m.shift {
                    list.select_to((project.clone(), id), cx);
                } else {
                    crate::bus::emit(cx, crate::bus::Signal::OpenTicket { project: project.clone(), ticket: id })
                }
            }))
    }
}

impl Render for FullList {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let _timing = crate::stats::Timing::new("fulllist");
        let query = self.search.read(cx).value().trim().to_lowercase();
        let rows = self.rows();

        let shown: Vec<&(RowState, &TicketRow)> = rows
            .iter()
            .filter(|(s, t)| self.band.is_none_or(|b| s.band == b) && (!self.unread_only || t.unread) && self.matches(t, &query))
            .collect();
        let order: Vec<(String, u64)> = shown.iter().map(|(_, t)| (t.project.clone(), t.id)).collect();
        // Rows chosen, the left column says what can be done to them.
        let side = if self.selected.is_empty() { self.side(&rows, &query, cx) } else { self.bulk_side(cx) };
        let mut list = div().named("full-list-rows").flex().flex_col().pb_4();
        // What holds the most back, project by project, heads the whole list
        // (no band, no filter): the tickets to move first.
        let criticals = self.criticals(&self.scoped());
        if self.band.is_none() && !self.unread_only && query.is_empty() && !criticals.is_empty() {
            let user = self.aiball.user.clone();
            list = list.child(div().px_4().child(group("Critical — what holds the most back")));
            for ticket in criticals {
                list = list.child(self.row(ticket, rowstate::of(ticket, &user), cx));
            }
            list = list.child(div().px_4().child(group("All")));
        }
        for (state, ticket) in &shown {
            list = list.child(self.row(ticket, *state, cx));
        }
        if shown.is_empty() {
            list = list.child(div().p_4().text_color(p().muted).child(if self.loading {
                "Reading…"
            } else {
                "No ticket here."
            }));
        }

        let title = match &self.scope {
            Some(project) => format!("Tickets — {project}"),
            None => "Tickets — all projects".to_string(),
        };
        // What the scope's tickets ask of you, as the side lists it.
        let alerts = match &self.scope {
            Some(project) => Alerts::of(self.open.get(project).into_iter().flatten(), self.critical.get(project).copied()),
            None => {
                let mut all = Alerts::of(self.projects.iter().filter_map(|p| self.open.get(p)).flatten(), None);
                all.critical = self.criticals(&self.projects).first().map(|t| t.id);
                all
            }
        };
        let view = div()
            .named("full-list")
            .absolute()
            .inset_0()
            .occlude()
            .flex()
            .flex_col()
            .bg(p().bg)
            .text_color(p().text)
            .child(
                div()
                    .flex()
                    .items_center()
                    .h(px(40.))
                    .px_4()
                    .border_b_1()
                    .border_color(p().border)
                    .child(div().text_lg().font_weight(FontWeight::BOLD).child(title))
                    .child(div().pl_3().child(alerts.badges("full-title")))
                    .child(div().flex_1())
                    .child(div().pr_4().text_xs().text_color(p().muted).child(format!("{} shown", shown.len())))
                    .child(
                        buttons::link("full-list-new", "+ New ticket")
                            .text_sm()
                            .tip(buttons::hint(cx, "A new ticket", "ticket.new"))
                            .on_click(cx.listener(|list, _, _, cx| {
                                let project = list.scope.clone();
                                crate::bus::emit(cx, crate::bus::Signal::AskNewTicket { project, parent: None })
                            })),
                    )
                    .child(buttons::separator())
                    .child(
                        buttons::link("full-list-close", "✕  Esc")
                            .text_sm()
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(CloseFullList))),
                    ),
            )
            .child(
                crate::sidecol::row()
                    .child(crate::sidecol::column(cx).bg(p().surface).child(side.overflow_y_scrollbar()))
                    .child(crate::sidecol::edge("full-list-edge"))
                    .child(div().flex_1().min_w_0().h_full().child(list.overflow_y_scrollbar())),
            );
        self.shown = order;
        view
    }
}

fn group(title: &'static str) -> Div {
    div()
        .pt_3()
        .pb_1()
        .text_xs()
        .font_weight(FontWeight::BOLD)
        .text_color(p().muted)
        .child(title.to_uppercase())
}
