//! A new ticket, full screen, laid out as a ticket's full detail: what it is
//! (project, intent, priority, scope, tags, assignee, milestone, parent) on
//! the left third, its words on the rest. One call files it; what aiball
//! takes on routes of their own (tags, assignee, milestone) follows, best
//! effort, as aiball's web UI does: a ticket that exists is worth more than
//! one perfectly labelled, so a failure there is said, and the ticket stays.
//!
//! The form lives on while hidden: Esc puts it away, the draft kept for the
//! next time, until it is sent.

use gpui_kit::component::input::{Input, InputEvent, InputState, Paste, Textarea, TextareaState};
use gpui_kit::component::text::TextView;
use gpui_kit::component::scroll::ScrollableElement as _;
use gpui_kit::component::Disableable as _;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::aiball::{Aiball, NewTicket};
use crate::{composer, field};
use crate::theme::p;
use crate::ui::buttons::{self, Look as _};

/// The form puts itself away (Esc, ✕); the draft stays.
pub struct CloseNewTicket;

/// A ticket was filed: the shell opens it.
pub struct Created {
    pub project: String,
    pub ticket: u64,
    /// Its title and body, as filed: its notification says them.
    pub title: String,
    pub body: String,
}

const INTENTS: &[&str] = &["request", "question", "fyi", "feature", "panic"];
const PRIORITIES: &[&str] = &["urgent", "high", "normal", "low"];
const SCOPES: &[&str] = &["internal", "default", "broadcast"];
const LEVELS: &[&str] = &["task", "milestone", "roadmap"];

/// The field whose choices are open in the left column.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Pick {
    Project,
    Tag,
    Milestone,
    Assignee,
}

/// The most suggestions a field shows.
const SUGGESTED: usize = 8;

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

/// The tallest a picture of the preview is drawn.
const PREVIEW_PICTURE_HEIGHT: f32 = 600.;

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
    /// What is typed in each picker, to search its choices.
    pickers: [(Pick, Entity<InputState>); 4],
    busy: bool,
    error: Option<String>,
    mentions: Vec<String>,
    /// The body shown as it will read (the Preview tab).
    preview: bool,
    /// The images of the body, read for its preview.
    images: crate::images::Cache,
}

impl EventEmitter<CloseNewTicket> for NewTicketForm {}
impl EventEmitter<Created> for NewTicketForm {}

impl NewTicketForm {
    /// The body as it will read: its words, and its pictures alone on their
    /// line drawn at the column's width (never beyond their size), as the
    /// thread draws them; those not read yet say so.
    fn preview_view(&self, text: &str) -> Div {
        use crate::images::Segment;
        let text = crate::images::preview(text, &self.images, false);
        let mut col = div().flex().flex_col().gap_2();
        for (i, segment) in crate::images::segments(&text, &self.images).into_iter().enumerate() {
            col = match segment {
                Segment::Text(md) => col.child(TextView::markdown(SharedString::from(format!("new-body-preview-{i}")), md).selectable(true)),
                Segment::Note(why) => col.child(div().text_xs().italic().text_color(p().muted).child(format!("({why})"))),
                Segment::Pictures(pictures) => col.children(pictures.into_iter().map(|picture| {
                    let (width, height) = (picture.width.max(1) as f32, picture.height.max(1) as f32);
                    div()
                        .w_full()
                        .max_w(px(width.min(PREVIEW_PICTURE_HEIGHT * width / height)))
                        .aspect_ratio(width / height)
                        .rounded_sm()
                        .overflow_hidden()
                        .border_1()
                        .border_color(p().border)
                        .child(img(ImageSource::Image(picture.image.clone())).size_full())
                })),
            };
        }
        col
    }

    pub fn new(aiball: Aiball, project: String, window: &mut Window, cx: &mut Context<Self>) -> Self {
        // The fields column's width, dragged here or on another full page.
        cx.observe_global::<crate::sidecol::SideWidth>(|_, cx| cx.notify()).detach();
        let title = cx.new(|cx| InputState::new(window, cx).placeholder("Title"));
        let summary = cx.new(|cx| InputState::new(window, cx).placeholder("Summary, one line (optional)"));
        let body = cx.new(|cx| {
            TextareaState::new(window, cx)
                .placeholder("What it is about… (@ to mention, ctrl+v pastes an image)")
                .auto_grow(10, 40)
        });
        // @-mentions are offered as the body is typed; Ctrl+Enter files.
        cx.subscribe_in(&body, window, |form: &mut Self, _, event: &InputEvent, window, cx| match event {
            InputEvent::Change => cx.notify(),
            InputEvent::PressEnter { secondary: true, .. } => form.submit(window, cx),
            _ => {}
        })
        .detach();
        // Ctrl+Enter files from the title and the summary too.
        for input in [&title, &summary] {
            cx.subscribe_in(input, window, |form: &mut Self, _, event: &InputEvent, window, cx| {
                if matches!(event, InputEvent::PressEnter { secondary: true, .. }) {
                    form.submit(window, cx);
                }
            })
            .detach();
        }
        let parent_input = cx.new(|cx| InputState::new(window, cx).placeholder("#ticket"));
        let pickers = [
            (Pick::Project, "another project…"),
            (Pick::Tag, "add a tag…"),
            (Pick::Milestone, "a milestone…"),
            (Pick::Assignee, "an agent…"),
        ]
        .map(|(pick, hint)| {
            let input = cx.new(|cx| InputState::new(window, cx).placeholder(hint));
            // Typing searches; Enter takes the first match.
            cx.subscribe_in(&input, window, move |form: &mut Self, input, event: &InputEvent, window, cx| match event {
                InputEvent::Change => cx.notify(),
                InputEvent::PressEnter { .. } => {
                    let query = input.read(cx).value().to_string();
                    if let Some((value, _, _)) = form.suggestions(pick, &query).into_iter().next() {
                        form.pick(pick, value, window, cx);
                    }
                }
                _ => {}
            })
            .detach();
            (pick, input)
        });
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
            pickers,
            busy: false,
            error: None,
            mentions: Vec::new(),
            preview: false,
            images: Default::default(),
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

    fn picker(&self, pick: Pick) -> &Entity<InputState> {
        &self.pickers.iter().find(|(p, _)| *p == pick).expect("every pick has its input").1
    }

    /// A picker's choices that match what is typed, as (value, label, one
    /// of the project's own): nothing typed, the likeliest few (the
    /// project's agents, the first tags and milestones).
    fn suggestions(&self, pick: Pick, query: &str) -> Vec<(String, String, bool)> {
        let catalog = &self.catalog;
        let all: Vec<(String, String, bool)> = match pick {
            Pick::Project => catalog.projects.iter().filter(|p| **p != self.project).map(|p| (p.clone(), p.clone(), true)).collect(),
            Pick::Tag => catalog.tags.iter().filter(|t| !self.tags.contains(t)).map(|t| (t.clone(), t.clone(), true)).collect(),
            Pick::Milestone => catalog
                .milestones
                .iter()
                .filter(|(id, _)| self.milestone.as_ref().is_none_or(|m| m.0 != *id))
                .map(|(id, title)| (id.to_string(), title.clone(), true))
                .collect(),
            Pick::Assignee => catalog
                .agents_by_project()
                .into_iter()
                .filter(|(a, _)| self.assignee.as_deref() != Some(a.as_str()))
                .map(|(a, own)| (a.clone(), a, own))
                .collect(),
        };
        let query = query.trim().to_lowercase();
        // One value, already chosen: nothing to offer until one is sought.
        let chosen = match pick {
            Pick::Milestone => self.milestone.is_some(),
            Pick::Assignee => self.assignee.is_some(),
            _ => false,
        };
        if query.is_empty() && chosen {
            return Vec::new();
        }
        if query.is_empty() {
            return match pick {
                // A project is searched for: the list is long.
                Pick::Project => Vec::new(),
                Pick::Assignee => all.into_iter().filter(|(_, _, own)| *own).take(SUGGESTED).collect(),
                _ => all.into_iter().take(SUGGESTED).collect(),
            };
        }
        // Names that start with it first, then those that contain it.
        let (mut first, rest): (Vec<_>, Vec<_>) = all
            .into_iter()
            .filter(|(_, label, _)| label.to_lowercase().contains(&query))
            .partition(|(_, label, _)| label.to_lowercase().starts_with(&query));
        first.extend(rest);
        first.truncate(SUGGESTED);
        first
    }

    /// A suggestion taken: the field set (a tag added), the search cleared.
    fn pick(&mut self, pick: Pick, value: String, window: &mut Window, cx: &mut Context<Self>) {
        match pick {
            Pick::Project => self.set_project(value, cx),
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
        self.picker(pick).clone().update(cx, |input, cx| input.set_value("", window, cx));
        cx.notify();
    }

    fn complete_mention(&mut self, name: String, window: &mut Window, cx: &mut Context<Self>) {
        let (before, _) = field::around_cursor(&self.body, cx);
        let Some(start) = composer::mention_start(&before) else { return };
        field::replace_range(&self.body, start..before.len(), &format!("@{name} "), window, cx);
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
                            let (before, after) = field::around_cursor(&form.body, cx);
                            field::insert_at_cursor(&form.body, &composer::image_snippet(&before, &after, &url), window, cx);
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
            self.picker(Pick::Project).clone().update(cx, |input, cx| input.focus(window, cx));
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
            tags: self.tags.iter().cloned().collect(),
            assignee: self.assignee.clone(),
            milestone: self.milestone.as_ref().map(|(id, _)| *id),
            level: self.level.into(),
        };
        let aiball = self.aiball.clone();
        let window_handle = window.window_handle();
        self.busy = true;
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
                    form.busy = false;
                    match filed {
                        Ok(ticket) => {
                            form.clear(window, cx);
                            cx.emit(Created { project, ticket, title, body });
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
        self.preview = false;
        (self.intent, self.priority, self.level, self.scope) = ("request", "normal", "task", "default");
        self.typed_parent = None;
        self.parent_input.update(cx, |input, cx| input.set_value("", window, cx));
        self.tags.clear();
        self.assignee = None;
        self.milestone = None;
        self.parent = None;
        for (_, input) in &self.pickers {
            input.update(cx, |input, cx| input.set_value("", window, cx));
        }
        self.error = None;
    }

    fn on_key(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let keystroke = &event.keystroke;
        if keystroke.modifiers.control && keystroke.key == "enter" {
            self.submit(window, cx);
            cx.stop_propagation();
        }
    }

    /// A choice dropped: a tag, the milestone, the assignee.
    fn unpick(&mut self, pick: Pick, value: &str, cx: &mut Context<Self>) {
        match pick {
            Pick::Project => {}
            Pick::Tag => self.tags.retain(|t| t != value),
            Pick::Milestone => self.milestone = None,
            Pick::Assignee => self.assignee = None,
        }
        cx.notify();
    }

    /// A picker: what is chosen (a click on ✕ drops it), where to type, and
    /// under it what matches.
    fn picker_field(&self, pick: Pick, chosen: Vec<(String, String)>, removable: bool, cx: &mut Context<Self>) -> Div {
        let input = self.picker(pick).clone();
        let query = input.read(cx).value().to_string();
        let id = format!("{pick:?}").to_lowercase();
        let mut row = div().flex().flex_wrap().items_center().gap_1();
        for (value, label) in chosen {
            row = row.child(
                div()
                    .id(SharedString::from(format!("new-{id}-chosen-{value}")))
                    .flex()
                    .items_center()
                    .gap_1()
                    .px_1p5()
                    .py_0p5()
                    .rounded_sm()
                    .text_xs()
                    .border_1()
                    .border_color(p().accent)
                    .bg(p().accent.opacity(0.15))
                    .child(label)
                    .when(removable, |d| {
                        d.child(
                            buttons::remove(SharedString::from(format!("new-{id}-drop-{value}")), "✕", "take it out")
                                .min_w(px(0.))
                                .on_click(cx.listener(move |form, _, _, cx| form.unpick(pick, &value, cx))),
                        )
                    }),
            );
        }
        row = row.child(div().w(px(160.)).child(Input::new(&input)));
        let mut suggested = div().flex().flex_wrap().gap_1().pt_1();
        let found = self.suggestions(pick, &query);
        let none = found.is_empty() && !query.trim().is_empty();
        for (value, label, own) in found {
            suggested = suggested.child(
                buttons::chip(SharedString::from(format!("new-{id}-{value}")), label)
                    .py_0p5()
                    .text_xs()
                    .when(!own, |d| d.text_color(p().muted))
                    .on_click(cx.listener(move |form, _, window, cx| form.pick(pick, value.clone(), window, cx))),
            );
        }
        if none {
            suggested = suggested.child(div().text_xs().text_color(p().muted).child("nothing matches"));
        }
        div().flex().flex_col().child(row).child(suggested)
    }

    /// The left third: what the ticket is. The short fields show their
    /// choices at once, the chosen one lit; the long ones (project, tags,
    /// milestone, assignee) are searched as they are typed.
    fn fields(&self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let label = |text: &'static str| div().w(px(80.)).flex_none().pt_0p5().text_color(p().muted).child(text);
        let choice = |id: String, text: String, on: bool, cx: &mut Context<Self>, set: Box<dyn Fn(&mut Self, &mut Context<Self>)>| {
            buttons::chip(SharedString::from(id), text)
                .py_0p5()
                .text_xs()
                .when(!on, |d| d.text_color(p().muted))
                .chosen(on)
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
        let mut col = div().id("new-ticket-fields").flex().flex_col().pr_3().text_sm();

        // ── Where ──
        let project = (!self.project.is_empty()).then(|| (self.project.clone(), self.project.clone()));
        col = col.child(group("Where")).child(field("project", self.picker_field(Pick::Project, project.into_iter().collect(), false, cx)));

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
        let tags = self.tags.iter().map(|t| (t.clone(), t.clone())).collect();
        col = col.child(field("tags", self.picker_field(Pick::Tag, tags, true, cx)));
        let milestone = self.milestone.iter().map(|(id, title)| (id.to_string(), title.clone())).collect();
        col = col.child(field("milestone", self.picker_field(Pick::Milestone, milestone, true, cx)));

        // ── People: the project's agents first ──
        let assignee = self.assignee.iter().map(|a| (a.clone(), a.clone())).collect();
        col = col.child(group("People")).child(field("assign to", self.picker_field(Pick::Assignee, assignee, true, cx)));

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
            None => div().w(px(120.)).child(Input::new(&self.parent_input)),
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
        let busy = self.busy;
        let mentions = composer::typed_mention(&field::around_cursor(&self.body, cx).0).map(|typed| {
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
            // The body, written or previewed; the names that fit an `@`
            // right under the words. (No summary: aiball's agents write
            // one; a person's title says enough.)
            .child(composer::write_tabs("new-body", self.preview, cx, |form: &mut Self, on, cx| {
                form.preview = on;
                if on {
                    // Its pictures, read once; the preview draws them when they came.
                    let text = form.body.read(cx).value().to_string();
                    let (aiball, images) = (form.aiball.clone(), form.images.clone());
                    cx.spawn(async move |this, cx| {
                        let came = cx.background_executor().spawn(async move { crate::images::load(&text, &aiball, &images) }).await;
                        if came {
                            let _ = this.update(cx, |_, cx| cx.notify());
                        }
                    })
                    .detach();
                }
                cx.notify();
            }))
            .child(if self.preview {
                let text = self.body.read(cx).value().to_string();
                div()
                    .id("new-body-preview")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .p_3()
                    .rounded_md()
                    .border_1()
                    .border_color(p().border)
                    .child(if text.trim().is_empty() {
                        div().text_color(p().muted).child("Nothing to preview yet.").into_any_element()
                    } else {
                        self.preview_view(&text).into_any_element()
                    })
                    .into_any_element()
            } else {
                div().flex_1().min_h_0().flex().flex_col().gap_1().child(Textarea::new(&self.body)).children(mentions).into_any_element()
            })
            .when_some(self.error.clone(), |d, error| d.child(div().text_color(p().danger).child(error)))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(div().flex_1().text_xs().text_color(p().muted).child("ctrl+enter files it · esc keeps the draft"))
                    .child(
                        buttons::primary("new-ticket-file", "File the ticket")
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
