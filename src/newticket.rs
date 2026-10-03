//! A new ticket, full screen, laid out as a ticket's full detail: what it is
//! (project, intent, priority, scope, tags, assignee, milestone, parent) on
//! the left third, its words on the rest. One call files it; what aiball
//! takes on routes of their own (tags, assignee, milestone) follows, best
//! effort, as aiball's web UI does: a ticket that exists is worth more than
//! one perfectly labelled, so a failure there is said, and the ticket stays.
//!
//! The form lives on while hidden: Esc puts it away, the draft kept for the
//! next time, until it is sent.

use crate::ui::Named as _;
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::scroll::ScrollableElement as _;
use gpui_kit::component::Disableable as _;
use gpui_kit::component::button::ButtonVariants as _;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::aiball::{Aiball, NewTicket};
use crate::theme::p;
use crate::ui::buttons;
use crate::kernel::catalog::{self, Catalog};
use crate::ui::combo::{self, Choice, ComboEvent, ComboState};
use crate::ui::ticket_text::{TicketText, TicketTextEvent};

/// The form puts itself away (Esc, ✕); the draft stays.
pub struct CloseNewTicket;

/// A ticket was filed: the shell opens it.
pub struct Created {
    pub project: String,
    pub ticket: u64,
    /// Its title and body, as filed: its notification says them.
    pub title: String,
    pub body: String,
    /// Whether it is opened once filed; else one is back where one was.
    pub open: bool,
}

use crate::ui::fields::{self, INTENTS, LEVELS, PRIORITIES, SCOPES};

/// The field whose choices are open in the left column.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Pick {
    Project,
    Intent,
    Priority,
    Level,
    Scope,
    Tag,
    Milestone,
    Assignee,
}


pub struct NewTicketForm {
    aiball: Aiball,
    project: String,
    intent: &'static str,
    priority: &'static str,
    level: &'static str,
    scope: &'static str,
    tags: Vec<String>,
    assignee: Option<String>,
    milestone: Option<(u64, String)>,
    parent: Option<u64>,
    /// The parent typed by hand, when no sub-ticket gesture gave one.
    typed_parent: Option<u64>,
    /// Its title and body, as they are written.
    text: Entity<TicketText>,
    summary: Entity<InputState>,
    /// A parent typed by hand: its number, with or without a hash.
    parent_input: Entity<InputState>,
    catalog: Catalog,
    /// Each long field's combo: a dropdown searched as it is typed.
    combos: [(Pick, Entity<ComboState>); 8],
    /// Being filed: then opened (`true`), or back where one was. The button
    /// of that gesture shows it at work, whatever started it (Ctrl+Enter).
    filing: Option<bool>,
    error: Option<String>,
}

/// What Ctrl+Enter does in a new ticket (Settings > Ticket list > New
/// ticket): files it and opens it, or files it and exits (the default).
fn ctrl_enter_opens(cx: &App) -> bool {
    crate::config::get::<crate::settings::Preferences>(cx).tickets.ctrl_enter_opens
}

impl EventEmitter<CloseNewTicket> for NewTicketForm {}
impl EventEmitter<Created> for NewTicketForm {}

impl NewTicketForm {
    pub fn new(aiball: Aiball, project: String, window: &mut Window, cx: &mut Context<Self>) -> Self {
        // The fields column's width, dragged here or on another full page.
        cx.observe_global::<crate::sidecol::SideWidth>(|_, cx| cx.notify()).detach();
        // A catalog read in: the combos told.
        cx.observe_in(&catalog::store(cx), window, |form: &mut Self, _, window, cx| form.take_catalog(window, cx)).detach();
        let text = cx.new(|cx| TicketText::new(aiball.clone(), "new", (10, 40), window, cx));
        cx.subscribe_in(&text, window, |form: &mut Self, _, event: &TicketTextEvent, window, cx| match event {
            TicketTextEvent::Submit => form.submit(ctrl_enter_opens(cx), window, cx),
            TicketTextEvent::Failed(error) => {
                form.error = Some(error.clone());
                cx.notify();
            }
        })
        .detach();
        let summary = cx.new(|cx| InputState::new(window, cx).placeholder("Summary, one line (optional)"));
        // Ctrl+Enter files from the summary too.
        cx.subscribe_in(&summary, window, |form: &mut Self, _, event: &InputEvent, window, cx| {
            if matches!(event, InputEvent::PressEnter { secondary: true, .. }) {
                form.submit(ctrl_enter_opens(cx), window, cx);
            }
        })
        .detach();
        let parent_input = cx.new(|cx| InputState::new(window, cx).placeholder("#ticket"));
        let combos = [Pick::Project, Pick::Intent, Pick::Priority, Pick::Level, Pick::Scope, Pick::Tag, Pick::Milestone, Pick::Assignee].map(|pick| {
            let state = combo::new(Vec::new(), None, window, cx);
            // A choice takes it; a milestone or an assignee cleared drops it.
            cx.subscribe_in(&state, window, move |form: &mut Self, _, event: &ComboEvent<combo::Choices>, window, cx| {
                let ComboEvent::Confirm(value) = event;
                match value {
                    Some(value) => form.pick(pick, value.clone(), window, cx),
                    None => form.unpick(pick, "", window, cx),
                }
            })
            .detach();
            (pick, state)
        });
        // A parent typed by hand counts once it reads as a number.
        cx.subscribe(&parent_input, |form: &mut Self, input, event: &InputEvent, cx| {
            if matches!(event, InputEvent::Change) {
                let text = input.read(cx).value().to_string();
                form.typed_parent = text.trim().trim_start_matches('#').parse::<u64>().ok();
            }
        })
        .detach();
        let mut form = Self {
            aiball,
            project: String::new(),
            intent: "request",
            priority: "normal",
            level: "task",
            scope: "default",
            tags: Vec::new(),
            assignee: None,
            milestone: None,
            parent: None,
            typed_parent: None,
            text,
            summary,
            parent_input,
            catalog: Catalog::default(),
            combos,
            filing: None,
            error: None,
        };
        form.set_project(project, window, cx);
        form
    }

    /// Opens on a project and a parent: a new draft takes them; one already
    /// begun keeps its own, save a parent asked for (a sub-ticket).
    pub fn prefill(&mut self, project: Option<String>, parent: Option<u64>, window: &mut Window, cx: &mut Context<Self>) {
        let begun = self.text.read(cx).begun(cx);
        if let Some(project) = project.filter(|_| !begun || parent.is_some()) {
            self.set_project(project, window, cx);
        }
        if parent.is_some() || !begun {
            self.parent = parent;
        }
        self.error = None;
        self.text.update(cx, |text, cx| text.focus_title(window, cx));
        cx.notify();
    }

    /// The project the ticket goes to: its tags and milestones are read
    /// again, and a milestone of the other project dropped.
    fn set_project(&mut self, project: String, window: &mut Window, cx: &mut Context<Self>) {
        if project == self.project && self.catalog.project == project && !self.catalog.projects.is_empty() {
            return;
        }
        if project != self.project {
            self.milestone = None;
        }
        self.project = project.clone();
        // The store reads it once for every view; its read comes in by
        // `take_catalog`.
        catalog::request(cx, &project);
        self.take_catalog(window, cx);
        cx.notify();
    }

    /// The project's catalog, as the store has it now: the combos told.
    fn take_catalog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let read = catalog::get(cx, &self.project).unwrap_or_else(|| Catalog { project: self.project.clone(), ..Default::default() });
        if read != self.catalog {
            self.catalog = read;
            self.fill_combos(window, cx);
            cx.notify();
        }
    }

    fn combo(&self, pick: Pick) -> &Entity<ComboState> {
        &self.combos.iter().find(|(p, _)| *p == pick).expect("every pick has its combo").1
    }

    /// A field's choices: the project's own agents first, the others greyed.
    fn choices(&self, pick: Pick) -> Vec<Choice> {
        let catalog = &self.catalog;
        match pick {
            Pick::Project => catalog.projects.iter().map(Choice::plain).collect(),
            Pick::Intent => INTENTS.iter().map(|v| Choice::plain(*v)).collect(),
            Pick::Priority => PRIORITIES.iter().map(|v| Choice::plain(*v)).collect(),
            Pick::Level => LEVELS.iter().map(|v| Choice::plain(*v)).collect(),
            Pick::Scope => SCOPES.iter().map(|v| Choice::plain(*v)).collect(),
            Pick::Tag => catalog.tags.iter().filter(|t| !self.tags.contains(t)).map(Choice::plain).collect(),
            Pick::Milestone => catalog.milestones.iter().map(|(id, title)| Choice::new(id.to_string(), title.clone())).collect(),
            Pick::Assignee => catalog
                .agents_by_project()
                .into_iter()
                .map(|(a, own)| if own { Choice::plain(a) } else { Choice::plain(a).muted() })
                .collect(),
        }
    }

    /// What a field's combo shows chosen: none for tags, which it adds to.
    fn selected(&self, pick: Pick) -> Option<String> {
        match pick {
            Pick::Project => (!self.project.is_empty()).then(|| self.project.clone()),
            Pick::Intent => Some(self.intent.to_string()),
            Pick::Priority => Some(self.priority.to_string()),
            Pick::Level => Some(self.level.to_string()),
            Pick::Scope => Some(self.scope.to_string()),
            Pick::Tag => None,
            Pick::Milestone => self.milestone.as_ref().map(|(id, _)| id.to_string()),
            Pick::Assignee => self.assignee.clone(),
        }
    }

    /// Every combo told its choices and what is chosen.
    fn fill_combos(&self, window: &mut Window, cx: &mut Context<Self>) {
        for (pick, state) in &self.combos {
            combo::set(state, self.choices(*pick), self.selected(*pick).as_deref(), window, cx);
        }
    }

    /// A choice taken: the field set (a tag added).
    fn pick(&mut self, pick: Pick, value: String, window: &mut Window, cx: &mut Context<Self>) {
        match pick {
            Pick::Project => self.set_project(value, window, cx),
            Pick::Intent => self.intent = INTENTS.iter().copied().find(|v| *v == value).unwrap_or(self.intent),
            Pick::Priority => self.priority = PRIORITIES.iter().copied().find(|v| *v == value).unwrap_or(self.priority),
            Pick::Level => self.level = LEVELS.iter().copied().find(|v| *v == value).unwrap_or(self.level),
            Pick::Scope => self.scope = SCOPES.iter().copied().find(|v| *v == value).unwrap_or(self.scope),
            Pick::Tag => {
                if !self.tags.contains(&value) {
                    self.tags.push(value);
                }
            }
            Pick::Milestone => {
                self.milestone = value
                    .parse::<u64>()
                    .ok()
                    .and_then(|id| self.catalog.milestones.iter().find(|(m, _)| *m == id).cloned());
            }
            Pick::Assignee => self.assignee = Some(value),
        }
        // A tag added leaves the combo empty, for the next one.
        self.fill_combos(window, cx);
        cx.notify();
    }

    /// Files the ticket, then what follows it; once filed it is opened
    /// (`open`), or one is back where one was.
    fn submit(&mut self, open: bool, window: &mut Window, cx: &mut Context<Self>) {
        if self.filing.is_some() || self.text.read(cx).uploading() {
            return;
        }
        let title = self.text.read(cx).title(cx).trim().to_string();
        if title.is_empty() {
            self.error = Some("A title first.".into());
            self.text.update(cx, |text, cx| text.focus_title(window, cx));
            cx.notify();
            return;
        }
        if self.project.is_empty() {
            self.error = Some("Which project?".into());
            self.combo(Pick::Project).clone().update(cx, |state, cx| state.focus(window, cx));
            cx.notify();
            return;
        }
        let ticket = NewTicket {
            project: self.project.clone(),
            title,
            summary: self.summary.read(cx).value().to_string(),
            body: self.text.read(cx).body(cx),
            intent: self.intent.into(),
            priority: self.priority.into(),
            scope: self.scope.into(),
            parent: self.parent.or(self.typed_parent),
            tags: self.tags.iter().cloned().collect(),
            assignee: self.assignee.clone(),
            milestone: self.milestone.as_ref().map(|(id, _)| *id),
            level: self.level.into(),
        };
        let aiball = self.aiball.clone();
        let window_handle = window.window_handle();
        self.filing = Some(open);
        self.error = None;
        cx.notify();
        cx.spawn(async move |this, cx| {
            let (project, title, body) = (ticket.project.clone(), ticket.title.clone(), ticket.body.clone());
            let filed = cx
                .background_executor()
                .spawn(async move {
                    aiball.create(&ticket)
                })
                .await;
            let _ = cx.update_window(window_handle, |_, window, cx| {
                let _ = this.update(cx, |form, cx| {
                    form.filing = None;
                    match filed {
                        Ok(ticket) => {
                            form.clear(window, cx);
                            cx.emit(Created { project, ticket, title, body, open });
                        }
                        Err(error) => form.error = Some(format!("{error:#}")),
                    }
                    cx.notify();
                });
            });
        })
        .detach();
    }

    /// A fresh draft, on the same project.
    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.summary.update(cx, |input, cx| input.set_value("", window, cx));
        self.text.update(cx, |text, cx| text.set("", "", window, cx));
        (self.intent, self.priority, self.level, self.scope) = ("request", "normal", "task", "default");
        self.typed_parent = None;
        self.parent_input.update(cx, |input, cx| input.set_value("", window, cx));
        self.tags.clear();
        self.assignee = None;
        self.milestone = None;
        self.parent = None;
        self.fill_combos(window, cx);
        self.error = None;
    }

    /// Ctrl+Enter files it, from the title too (the body is a composer).
    fn on_key(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let keystroke = &event.keystroke;
        if keystroke.modifiers.control && keystroke.key == "enter" {
            self.submit(ctrl_enter_opens(cx), window, cx);
            cx.stop_propagation();
        }
    }

    /// A choice dropped: a tag, the milestone, the assignee.
    fn unpick(&mut self, pick: Pick, value: &str, window: &mut Window, cx: &mut Context<Self>) {
        match pick {
            Pick::Project | Pick::Intent | Pick::Priority | Pick::Level | Pick::Scope => {}
            Pick::Tag => self.tags.retain(|t| t != value),
            Pick::Milestone => self.milestone = None,
            Pick::Assignee => self.assignee = None,
        }
        self.fill_combos(window, cx);
        cx.notify();
    }

    /// A long field: its combo, searched as it is typed. Tags, which are
    /// several, stand beside it (a click on ✕ drops one); the combo adds.
    fn picker_field(&self, pick: Pick, cx: &mut Context<Self>) -> Div {
        let id = format!("{pick:?}").to_lowercase();
        let (placeholder, search) = match pick {
            Pick::Project => ("a project", "a project…"),
            Pick::Intent => ("intent", "an intent…"),
            Pick::Priority => ("priority", "a priority…"),
            Pick::Level => ("level", "a level…"),
            Pick::Scope => ("scope", "a scope…"),
            Pick::Tag => ("add a tag", "a tag…"),
            Pick::Milestone => ("none", "a milestone…"),
            Pick::Assignee => ("nobody", "an agent…"),
        };
        let clearable = matches!(pick, Pick::Milestone | Pick::Assignee);
        let picker = fields::picker(self.combo(pick), format!("new-{id}"), placeholder, search, clearable);
        if pick != Pick::Tag {
            return picker;
        }
        let chips = self
            .tags
            .clone()
            .into_iter()
            .map(|tag| {
                let drop = fields::tag_drop(format!("new-tag-drop-{tag}"))
                    .on_click(cx.listener({
                        let tag = tag.clone();
                        move |form, _, window, cx| form.unpick(Pick::Tag, &tag, window, cx)
                    }));
                fields::tag(format!("new-tag-chosen-{tag}"), &tag, drop)
            })
            .collect();
        fields::tags(chips, picker)
    }

    /// The left third: what the ticket is. The short fields show their
    /// choices at once, the chosen one lit; the long ones (project, tags,
    /// milestone, assignee) are searched as they are typed.
    fn fields(&self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        use fields::{field, group};
        let mut col = div().named("new-ticket-fields").flex().flex_col().pr_3().text_sm();

        // ── Where ──
        col = col.child(group("Where")).child(field("project", self.picker_field(Pick::Project, cx)));

        // ── Fields ──
        col = col
            .child(group("Fields"))
            .child(field("intent", self.picker_field(Pick::Intent, cx)))
            .child(field("priority", self.picker_field(Pick::Priority, cx)))
            .child(field("level", self.picker_field(Pick::Level, cx)))
            .child(field("scope", self.picker_field(Pick::Scope, cx)))
            .child(fields::scope_note(self.scope))
            .child(field("tags", self.picker_field(Pick::Tag, cx)))
            .child(field("milestone", self.picker_field(Pick::Milestone, cx)));

        // ── People: the project's agents first ──
        col = col.child(group("People")).child(field("assign to", self.picker_field(Pick::Assignee, cx)));

        // ── Links ──
        let parent = match self.parent {
            Some(parent) => div()
                .flex()
                .items_center()
                .gap_2()
                .child(crate::ui::ticketref::link("new-parent", parent, None))
                .child(
                    buttons::remove("new-parent-drop", "✕", "no parent")
                        .text_xs()
                        .on_click(cx.listener(|form, _, _, cx| {
                            form.parent = None;
                            cx.notify();
                        })),
                ),
            None => crate::focusmode::on_hover(div().w(px(120.)).child(Input::new(&self.parent_input)), self.parent_input.read(cx).focus_handle(cx)),
        };
        col = col.child(group("Links")).child(field("sub-ticket of", parent));

        crate::sidecol::column(cx)
            .px_4()
            .pb_4()
            .bg(p().surface)
            .child(col.overflow_y_scrollbar())
    }
}

impl Render for NewTicketForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let busy = self.filing.is_some() || self.text.read(cx).uploading();
        let words = div()
            .flex_1()
            .min_w_0()
            .h_full()
            .flex()
            .flex_col()
            .gap_2()
            .p_4()
            // Its words; no summary: aiball's agents write one, a person's
            // title says enough.
            .child(self.text.clone())
            .when_some(self.error.clone(), |d, error| d.child(div().text_color(p().danger).child(error)))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(div().flex_1().text_xs().text_color(p().muted).child(if ctrl_enter_opens(cx) {
                        "ctrl+enter files it and opens it · esc keeps the draft"
                    } else {
                        "ctrl+enter files it and exits · esc keeps the draft"
                    }))
                    .child(crate::tips::target(
                        "new-ticket.file-exit",
                        buttons::answer("new-ticket-file-exit", "File and exit")
                            // Filled in the warning colour, beside the accent of "File the ticket".
                            .warning()
                            // Filled, unless greyed while the other files it.
                            .when(!(busy && self.filing != Some(false)), |b| b.bg(p().warning).text_color(p().bg))
                            // At work: its spinner (the kit spins an icon only).
                            .when(self.filing == Some(false), |b| b.icon(gpui_kit::component::IconName::LoaderCircle).loading(true))
                            .disabled(busy && self.filing != Some(false))
                            .tooltip("Files it without opening it: back to where you were")
                            .on_click(cx.listener(|form, _, window, cx| form.submit(false, window, cx))),
                    ))
                    .child(
                        buttons::primary("new-ticket-file", "File the ticket")
                            .when(self.filing == Some(true), |b| b.icon(gpui_kit::component::IconName::LoaderCircle).loading(true))
                            .disabled(busy && self.filing != Some(true))
                            .on_click(cx.listener(|form, _, window, cx| form.submit(true, window, cx))),
                    ),
            );
        let title = match self.parent {
            Some(parent) => format!("New sub-ticket of #{parent}"),
            None => "New ticket".into(),
        };
        div()
            .named("new-ticket")
            .absolute()
            .inset_0()
            .occlude()
            .flex()
            .flex_col()
            .bg(p().bg)
            .text_color(p().text)
            .capture_key_down(cx.listener(Self::on_key))
            .child(
                div()
                    .flex()
                    .items_center()
                    .h(px(40.))
                    .px_4()
                    .border_b_1()
                    .border_color(p().border)
                    .child(div().flex_1().text_lg().font_weight(FontWeight::BOLD).child(title))
                    .child(
                        buttons::link("new-ticket-close", "✕  Esc")
                            .text_sm()
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(CloseNewTicket))),
                    ),
            )
            .child(crate::sidecol::row().child(self.fields(cx)).child(crate::sidecol::edge("new-ticket-edge")).child(words))
    }
}

/// The project a new ticket goes to, first found: the one the panel shows,
/// the active terminal's, the last one used, the first on the board.
pub fn default_project(panel: Option<&str>, terminal: Option<&str>, last: Option<&str>, board: &[String]) -> Option<String> {
    let on_board = |p: &&str| board.iter().any(|b| b == p);
    panel
        .filter(on_board)
        .or(terminal.filter(on_board))
        .or(last.filter(on_board))
        .map(str::to_string)
        .or_else(|| board.first().cloned())
}

#[cfg(test)]
mod tests {
    use super::default_project;

    #[test]
    fn the_project_comes_from_what_is_shown() {
        let board = vec!["aiball".to_string(), "tvty".to_string()];
        assert_eq!(default_project(Some("tvty"), Some("aiball"), None, &board).as_deref(), Some("tvty"));
        assert_eq!(default_project(None, Some("aiball"), Some("tvty"), &board).as_deref(), Some("aiball"));
        assert_eq!(default_project(None, Some("tmux"), Some("tvty"), &board).as_deref(), Some("tvty"));
        assert_eq!(default_project(None, None, Some("gone"), &board).as_deref(), Some("aiball"));
        assert_eq!(default_project(None, None, None, &[]), None);
    }
}
