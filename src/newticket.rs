//! A new ticket, full screen, laid out as a ticket's full detail: what it is
//! (project, intent, priority, scope, tags, assignee, milestone, parent) on
//! the left third, its words on the rest. One call files it; what aiball
//! takes on routes of their own (tags, assignee, milestone) follows, best
//! effort, as aiball's web UI does: a ticket that exists is worth more than
//! one perfectly labelled, so a failure there is said, and the ticket stays.
//!
//! The form lives on while hidden: Esc puts it away, the draft kept for the
//! next time, until it is sent.

use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{Input, InputEvent, InputState, Paste, Textarea, TextareaState};
use gpui_kit::component::scroll::ScrollableElement as _;
use gpui_kit::component::Disableable as _;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::aiball::{Aiball, NewTicket};
use crate::composer;
use crate::theme::p;

/// The form puts itself away (Esc, ✕); the draft stays.
pub struct CloseNewTicket;

/// A ticket was filed: the shell opens it. `warnings`: what did not follow.
pub struct Created {
    pub project: String,
    pub ticket: u64,
    pub warnings: Vec<String>,
}

const INTENTS: &[&str] = &["request", "question", "fyi", "feature", "panic"];
const PRIORITIES: &[&str] = &["urgent", "high", "normal", "low"];
const SCOPES: &[&str] = &["internal", "default", "broadcast"];
const LEVELS: &[&str] = &["task", "milestone", "roadmap"];

/// The field whose choices are open in the left column.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Field {
    Project,
}

/// What the left column offers: the board's projects and agents, and the
/// chosen project's tags and milestones.
#[derive(Clone, Default)]
struct Catalog {
    project: String,
    projects: Vec<String>,
    agents: Vec<String>,
    /// The agents working on this project.
    own: Vec<String>,
    tags: Vec<String>,
    milestones: Vec<(u64, String)>,
}

impl Catalog {
    /// Every agent, the project's own first: (name, whether it is).
    fn agents_by_project(&self) -> Vec<(String, bool)> {
        let mut all: Vec<(String, bool)> = self.own.iter().map(|a| (a.clone(), true)).collect();
        all.extend(self.agents.iter().filter(|a| !self.own.contains(a)).map(|a| (a.clone(), false)));
        all
    }
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
    title: Entity<InputState>,
    summary: Entity<InputState>,
    body: Entity<TextareaState>,
    /// A parent typed by hand: its number, with or without a hash.
    parent_input: Entity<InputState>,
    catalog: Catalog,
    editing: Option<Field>,
    busy: bool,
    error: Option<String>,
    mentions: Vec<String>,
}

impl EventEmitter<CloseNewTicket> for NewTicketForm {}
impl EventEmitter<Created> for NewTicketForm {}

impl NewTicketForm {
    pub fn new(aiball: Aiball, project: String, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let title = cx.new(|cx| InputState::new(window, cx).placeholder("Title"));
        let summary = cx.new(|cx| InputState::new(window, cx).placeholder("Summary, one line (optional)"));
        let body = cx.new(|cx| {
            TextareaState::new(window, cx)
                .placeholder("What it is about… (@ to mention, ctrl+v pastes an image)")
                .auto_grow(10, 40)
        });
        // @-mentions are offered as the body is typed.
        cx.subscribe(&body, |_, _, event: &InputEvent, cx| {
            if matches!(event, InputEvent::Change) {
                cx.notify();
            }
        })
        .detach();
        let parent_input = cx.new(|cx| InputState::new(window, cx).placeholder("#ticket"));
        // A parent typed by hand counts once it reads as a number.
        cx.subscribe(&parent_input, |form: &mut Self, input, event: &InputEvent, cx| {
            if matches!(event, InputEvent::Change) {
                let text = input.read(cx).value().to_string();
                form.typed_parent = text.trim().trim_start_matches('#').parse::<u64>().ok();
            }
        })
        .detach();
        let reader = aiball.clone();
        cx.spawn(async move |this, cx| {
            let mentions = cx.background_executor().spawn(async move { reader.mention_suggestions() }).await;
            if let Ok(mentions) = mentions {
                let _ = this.update(cx, |form, _| form.mentions = mentions);
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
            title,
            summary,
            body,
            parent_input,
            catalog: Catalog::default(),
            editing: None,
            busy: false,
            error: None,
            mentions: Vec::new(),
        };
        form.set_project(project, cx);
        form
    }

    /// Opens on a project and a parent: a new draft takes them; one already
    /// begun keeps its own, save a parent asked for (a sub-ticket).
    pub fn prefill(&mut self, project: Option<String>, parent: Option<u64>, window: &mut Window, cx: &mut Context<Self>) {
        let begun = !self.title.read(cx).value().trim().is_empty() || !self.body.read(cx).value().trim().is_empty();
        if let Some(project) = project.filter(|_| !begun || parent.is_some()) {
            self.set_project(project, cx);
        }
        if parent.is_some() || !begun {
            self.parent = parent;
        }
        self.error = None;
        self.title.update(cx, |input, cx| input.focus(window, cx));
        cx.notify();
    }

    /// The project the ticket goes to: its tags and milestones are read
    /// again, and a milestone of the other project dropped.
    fn set_project(&mut self, project: String, cx: &mut Context<Self>) {
        if project == self.project && self.catalog.project == project {
            return;
        }
        if project != self.project {
            self.milestone = None;
        }
        self.project = project.clone();
        let aiball = self.aiball.clone();
        cx.spawn(async move |this, cx| {
            let catalog = cx
                .background_executor()
                .spawn(async move {
                    let (projects, agents) = aiball.projects_and_agents().unwrap_or_default();
                    let own = aiball
                        .consumers()
                        .unwrap_or_default()
                        .into_iter()
                        .filter(|c| c.kind != "human" && c.project.as_deref() == Some(project.as_str()))
                        .map(|c| c.consumer_id)
                        .collect();
                    Catalog {
                        own,
                        tags: aiball.tag_catalog(&project).unwrap_or_default(),
                        milestones: aiball.milestones(&project).unwrap_or_default(),
                        projects,
                        agents,
                        project,
                    }
                })
                .await;
            let _ = this.update(cx, |form, cx| {
                if catalog.project == form.project {
                    form.catalog = catalog;
                    cx.notify();
                }
            });
        })
        .detach();
        cx.notify();
    }

    fn toggle(&mut self, field: Field, cx: &mut Context<Self>) {
        self.editing = if self.editing == Some(field) { None } else { Some(field) };
        cx.notify();
    }

    fn complete_mention(&mut self, name: String, window: &mut Window, cx: &mut Context<Self>) {
        let text = self.body.read(cx).value().to_string();
        let Some(completed) = composer::complete_mention(&text, &name) else { return };
        self.body.update(cx, |body, cx| body.set_value(completed, window, cx));
        cx.notify();
    }

    /// Ctrl+V with an image: uploaded, its link put in the body.
    fn paste_image(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let Some((bytes, content_type, name)) = composer::clipboard_image(cx) else {
            return false;
        };
        let aiball = self.aiball.clone();
        let window_handle = window.window_handle();
        self.busy = true;
        cx.spawn(async move |this, cx| {
            let uploaded = cx
                .background_executor()
                .spawn(async move { aiball.upload(&bytes, &content_type, &name) })
                .await;
            let _ = cx.update_window(window_handle, |_, window, cx| {
                let _ = this.update(cx, |form, cx| {
                    form.busy = false;
                    match uploaded {
                        Ok(url) => {
                            let text = composer::with_image(&form.body.read(cx).value(), &url);
                            form.body.update(cx, |body, cx| body.set_value(text, window, cx));
                        }
                        Err(error) => form.error = Some(format!("{error:#}")),
                    }
                    cx.notify();
                });
            });
        })
        .detach();
        true
    }

    /// Files the ticket, then what follows it; opens it once filed.
    fn submit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        let title = self.title.read(cx).value().trim().to_string();
        if title.is_empty() {
            self.error = Some("A title first.".into());
            self.title.update(cx, |input, cx| input.focus(window, cx));
            cx.notify();
            return;
        }
        if self.project.is_empty() {
            self.error = Some("Which project?".into());
            self.editing = Some(Field::Project);
            cx.notify();
            return;
        }
        let ticket = NewTicket {
            project: self.project.clone(),
            title,
            summary: self.summary.read(cx).value().to_string(),
            body: self.body.read(cx).value().to_string(),
            intent: self.intent.into(),
            priority: self.priority.into(),
            scope: self.scope.into(),
            parent: self.parent.or(self.typed_parent),
        };
        let (tags, assignee, milestone) = (self.tags.clone(), self.assignee.clone(), self.milestone.clone());
        // aiball takes the level after the ticket exists (a human's gesture).
        let level = (self.level != "task").then_some(self.level);
        let aiball = self.aiball.clone();
        let window_handle = window.window_handle();
        self.busy = true;
        self.error = None;
        cx.notify();
        cx.spawn(async move |this, cx| {
            let project = ticket.project.clone();
            let filed = cx
                .background_executor()
                .spawn(async move {
                    let id = aiball.create(&ticket)?;
                    let mut warnings = Vec::new();
                    for tag in &tags {
                        if let Err(error) = aiball.add_tag(id, tag) {
                            warnings.push(format!("tag {tag}: {error:#}"));
                        }
                    }
                    if let Some(who) = &assignee {
                        if let Err(error) = aiball.assign(id, Some(who)) {
                            warnings.push(format!("assign to {who}: {error:#}"));
                        }
                    }
                    if let Some(level) = level {
                        if let Err(error) = aiball.edit(id, serde_json::json!({ "level": level })) {
                            warnings.push(format!("level {level}: {error:#}"));
                        }
                    }
                    if let Some((milestone, name)) = &milestone {
                        if let Err(error) = aiball.set_milestone(id, Some(*milestone)) {
                            warnings.push(format!("milestone {name}: {error:#}"));
                        }
                    }
                    anyhow::Ok((id, warnings))
                })
                .await;
            let _ = cx.update_window(window_handle, |_, window, cx| {
                let _ = this.update(cx, |form, cx| {
                    form.busy = false;
                    match filed {
                        Ok((ticket, warnings)) => {
                            form.clear(window, cx);
                            cx.emit(Created { project, ticket, warnings });
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
        for input in [&self.title, &self.summary] {
            input.update(cx, |input, cx| input.set_value("", window, cx));
        }
        self.body.update(cx, |body, cx| body.set_value("", window, cx));
        (self.intent, self.priority, self.level, self.scope) = ("request", "normal", "task", "default");
        self.typed_parent = None;
        self.parent_input.update(cx, |input, cx| input.set_value("", window, cx));
        self.tags.clear();
        self.assignee = None;
        self.milestone = None;
        self.parent = None;
        self.editing = None;
        self.error = None;
    }

    fn on_key(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let keystroke = &event.keystroke;
        if keystroke.modifiers.control && keystroke.key == "enter" {
            self.submit(window, cx);
            cx.stop_propagation();
        }
    }

    /// The left third: what the ticket is. Every field shows its choices at
    /// once, the chosen one lit; only the project, a long list, opens.
    fn fields(&self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let label = |text: &'static str| div().w(px(80.)).flex_none().pt_0p5().text_color(p().muted).child(text);
        let choice = |id: String, text: String, on: bool, cx: &mut Context<Self>, set: Box<dyn Fn(&mut Self, &mut Context<Self>)>| {
            div()
                .id(SharedString::from(id))
                .px_1p5()
                .py_0p5()
                .rounded_sm()
                .text_xs()
                .border_1()
                .border_color(if on { p().accent } else { p().border })
                .bg(if on { p().accent.opacity(0.15) } else { p().bg.opacity(0.) })
                .text_color(if on { p().text } else { p().muted })
                .cursor_pointer()
                .hover(|d| d.bg(p().hover))
                .child(text)
                .on_click(cx.listener(move |form, _, _, cx| {
                    set(form, cx);
                    cx.notify();
                }))
        };
        // A field: its name, and its choices beside it, wrapping.
        let field = |text: &'static str, choices: Div| {
            div().flex().gap_2().py_1().child(label(text)).child(choices.flex_1().min_w_0())
        };
        let chips = || div().flex().flex_wrap().gap_1();
        let group = |title: &'static str| {
            div()
                .pt_3()
                .pb_1()
                .text_xs()
                .font_weight(FontWeight::BOLD)
                .text_color(p().muted)
                .child(title.to_uppercase())
        };
        let catalog = &self.catalog;

        let mut col = div().id("new-ticket-fields").flex().flex_col().pr_3().text_sm();

        // ── Where ──
        let open = self.editing == Some(Field::Project);
        col = col.child(group("Where")).child(
            div()
                .id("new-Project")
                .flex()
                .gap_2()
                .py_1()
                .px_1()
                .rounded_sm()
                .cursor_pointer()
                .when(open, |d| d.bg(p().active))
                .hover(|d| d.bg(p().hover))
                .on_click(cx.listener(|form, _, _, cx| form.toggle(Field::Project, cx)))
                .child(label("project"))
                .child(div().flex_1().child(if self.project.is_empty() { "—".to_string() } else { self.project.clone() }))
                .child(div().text_color(p().muted).child(if open { "▴" } else { "▾" })),
        );
        if open {
            let mut list = chips().pl(px(88.)).pb_1();
            for project in &catalog.projects {
                let name = project.clone();
                list = list.child(choice(
                    format!("new-project-{project}"),
                    project.clone(),
                    *project == self.project,
                    cx,
                    Box::new(move |form, cx| {
                        form.set_project(name.clone(), cx);
                        form.editing = None;
                    }),
                ));
            }
            col = col.child(list);
        }

        // ── Fields: every choice in sight ──
        col = col.child(group("Fields"));
        let fixed: [(&'static str, &'static str, &'static [&'static str], fn(&mut Self, &'static str)); 4] = [
            ("intent", self.intent, INTENTS, |f, v| f.intent = v),
            ("priority", self.priority, PRIORITIES, |f, v| f.priority = v),
            ("level", self.level, LEVELS, |f, v| f.level = v),
            ("scope", self.scope, SCOPES, |f, v| f.scope = v),
        ];
        for (name, value, values, set) in fixed {
            let mut list = chips();
            for v in values {
                let v: &'static str = v;
                list = list.child(choice(format!("new-{name}-{v}"), v.into(), value == v, cx, Box::new(move |form, _| set(form, v))));
            }
            col = col.child(field(name, list));
            if name == "scope" {
                // Two lines kept whatever it says: the fields below do not move.
                col = col.child(div().pl(px(88.)).pb_1().h(px(36.)).text_xs().text_color(p().muted).child(match value {
                    "internal" => "notifies nobody but who is mentioned",
                    "broadcast" => "notifies the project's followers too",
                    _ => "notifies the ticket's subscribers and the project's owners",
                }));
            }
        }
        let mut tags = chips();
        for tag in &catalog.tags {
            let (name, on) = (tag.clone(), self.tags.contains(tag));
            tags = tags.child(choice(
                format!("new-tag-{tag}"),
                tag.clone(),
                on,
                cx,
                Box::new(move |form, _| {
                    if on {
                        form.tags.retain(|t| *t != name);
                    } else {
                        form.tags.push(name.clone());
                    }
                }),
            ));
        }
        if catalog.tags.is_empty() {
            tags = tags.child(div().text_xs().text_color(p().muted).child("no tag yet"));
        }
        col = col.child(field("tags", tags));
        let mut milestones = chips().child(choice(
            "new-milestone-none".into(),
            "none".into(),
            self.milestone.is_none(),
            cx,
            Box::new(|form, _| form.milestone = None),
        ));
        for (id, title) in &catalog.milestones {
            let chosen = (*id, title.clone());
            milestones = milestones.child(choice(
                format!("new-milestone-{id}"),
                title.clone(),
                self.milestone.as_ref().is_some_and(|m| m.0 == *id),
                cx,
                Box::new(move |form, _| form.milestone = Some(chosen.clone())),
            ));
        }
        col = col.child(field("milestone", milestones));

        // ── People: the project's agents first ──
        let mut people = chips().child(choice(
            "new-assign-none".into(),
            "nobody".into(),
            self.assignee.is_none(),
            cx,
            Box::new(|form, _| form.assignee = None),
        ));
        for (agent, own) in catalog.agents_by_project() {
            let name = agent.clone();
            people = people.child(
                choice(
                    format!("new-assign-{agent}"),
                    agent.clone(),
                    self.assignee.as_deref() == Some(agent.as_str()),
                    cx,
                    Box::new(move |form, _| form.assignee = Some(name.clone())),
                )
                .when(!own, |d| d.opacity(0.7)),
            );
        }
        col = col.child(group("People")).child(field("assign to", people));

        // ── Links ──
        let parent = match self.parent {
            Some(parent) => div()
                .flex()
                .items_center()
                .gap_2()
                .child(format!("#{parent}"))
                .child(
                    div()
                        .id("new-parent-drop")
                        .px_1()
                        .text_xs()
                        .text_color(p().muted)
                        .cursor_pointer()
                        .hover(|d| d.text_color(p().danger))
                        .child("✕")
                        .on_click(cx.listener(|form, _, _, cx| {
                            form.parent = None;
                            cx.notify();
                        })),
                ),
            None => div().w(px(120.)).child(Input::new(&self.parent_input)),
        };
        col = col.child(group("Links")).child(field("sub-ticket of", parent));

        div()
            .w_1_3()
            .flex_none()
            .h_full()
            .px_4()
            .pb_4()
            .bg(p().surface)
            .border_r_1()
            .border_color(p().border)
            .child(col.overflow_y_scrollbar())
    }
}

impl Render for NewTicketForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let busy = self.busy;
        let mentions = composer::typed_mention(&self.body.read(cx).value()).map(|typed| {
            composer::mention_chips(&self.mentions, &typed, "new-mention", cx, |form: &mut Self, name, window, cx| {
                form.complete_mention(name, window, cx)
            })
        });
        let words = div()
            .flex_1()
            .min_w_0()
            .h_full()
            .flex()
            .flex_col()
            .gap_2()
            .p_4()
            // A paste with an image goes to aiball, its link to the body.
            .capture_action(cx.listener(|form, _: &Paste, window, cx| {
                if form.paste_image(window, cx) {
                    cx.stop_propagation();
                }
            }))
            .child(Input::new(&self.title))
            .child(Input::new(&self.summary))
            // The names that fit an `@` right under the words.
            .child(div().flex_1().min_h_0().flex().flex_col().gap_1().child(Textarea::new(&self.body)).children(mentions))
            .when_some(self.error.clone(), |d, error| d.child(div().text_color(p().danger).child(error)))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(div().flex_1().text_xs().text_color(p().muted).child("ctrl+enter files it · esc keeps the draft"))
                    .child(
                        Button::new("new-ticket-file")
                            .primary()
                            .label("File the ticket")
                            .loading(busy)
                            .disabled(busy)
                            .on_click(cx.listener(|form, _, window, cx| form.submit(window, cx))),
                    ),
            );
        let title = match self.parent {
            Some(parent) => format!("New sub-ticket of #{parent}"),
            None => "New ticket".into(),
        };
        div()
            .id("new-ticket")
            .absolute()
            .inset_0()
            .occlude()
            .flex()
            .flex_col()
            .bg(p().bg)
            .text_color(p().text)
            .on_key_down(cx.listener(Self::on_key))
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
                        div()
                            .id("new-ticket-close")
                            .px_2()
                            .rounded_sm()
                            .text_sm()
                            .text_color(p().muted)
                            .cursor_pointer()
                            .hover(|d| d.bg(p().hover).text_color(p().text))
                            .child("✕  Esc")
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(CloseNewTicket))),
                    ),
            )
            .child(div().flex().flex_1().min_h_0().child(self.fields(cx)).child(words))
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
