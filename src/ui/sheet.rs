//! The sheet: a large panel over the dimmed window, for what asks more than
//! a yes or a no — choosing among sessions, a wizard's steps. Always the
//! same: a head (its title, a line saying what is asked, ✕ Esc), the steps
//! when there are some, a body that takes the room and scrolls, and a foot
//! (a setting on the left, the answers on the right). A plain confirmation
//! keeps the small dialog (`shell::quit::dialog`).
//!
//! Its parts are named from its id, for the debug control: `{id}-veil`,
//! `{id}-card`, `{id}-page` (the body), `{id}-close`.

use gpui_kit::*;

use crate::theme::p;
use crate::ui::Named as _;
use crate::ui::buttons;

/// The sheet's size, at most: a share of the window beyond.
const WIDTH: f32 = 760.;
const HEIGHT: f32 = 640.;

type OnClose = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// A sheet, built part by part, then drawn by [`Sheet::render`].
pub struct Sheet {
    id: &'static str,
    title: SharedString,
    summary: Option<SharedString>,
    steps: Option<AnyElement>,
    body: Option<AnyElement>,
    setting: Option<AnyElement>,
    answers: Option<AnyElement>,
    on_close: Option<OnClose>,
}

impl Sheet {
    pub fn new(id: &'static str, title: impl Into<SharedString>) -> Self {
        Self { id, title: title.into(), summary: None, steps: None, body: None, setting: None, answers: None, on_close: None }
    }

    /// The line under the title: what is asked, and what each answer does.
    pub fn summary(mut self, summary: impl Into<SharedString>) -> Self {
        self.summary = Some(summary.into());
        self
    }

    /// A wizard's steps, under the head.
    pub fn steps(mut self, steps: impl IntoElement) -> Self {
        self.steps = Some(steps.into_any_element());
        self
    }

    pub fn body(mut self, body: impl IntoElement) -> Self {
        self.body = Some(body.into_any_element());
        self
    }

    /// A setting beside the answers ("Remember this choice").
    pub fn setting(mut self, setting: impl IntoElement) -> Self {
        self.setting = Some(setting.into_any_element());
        self
    }

    /// The answers, the main one last.
    pub fn answers(mut self, answers: impl IntoElement) -> Self {
        self.answers = Some(answers.into_any_element());
        self
    }

    /// What ✕ Esc does (Esc itself is the window's: it calls the same).
    pub fn on_close(mut self, on_close: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Box::new(on_close));
        self
    }

    pub fn render(self) -> AnyElement {
        let id = self.id;
        let head = div()
            .flex()
            .flex_none()
            .items_start()
            .gap_3()
            .px_5()
            .pt_4()
            .pb_2()
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(div().text_lg().font_weight(FontWeight::BOLD).child(self.title))
                    .children(self.summary.map(|s| div().text_sm().text_color(p().muted).child(s))),
            )
            .children(self.on_close.map(|close| {
                buttons::link(SharedString::from(format!("{id}-close")), "✕  Esc").text_sm().flex_none().on_click(move |e, w, cx| close(e, w, cx))
            }));
        let foot = div()
            .flex()
            .flex_none()
            .flex_wrap()
            .items_center()
            .gap_3()
            .px_5()
            .py_3()
            .border_t_1()
            .border_color(p().border)
            // The setting takes what the answers leave; short of room, it
            // goes on a line of its own above them. Never narrower than its
            // text: a smaller floor let it run under the answers (seen on
            // Windows, where the text is wider).
            .child(div().flex_grow(1.).children(self.setting))
            .children(self.answers.map(|a| div().flex().flex_none().ml_auto().gap_2().child(a)));
        let card = div()
            .named(SharedString::from(format!("{id}-card")))
            .occlude()
            .w(px(WIDTH))
            .h(px(HEIGHT))
            .max_w(relative(0.92))
            .max_h(relative(0.9))
            .flex()
            .flex_col()
            .rounded_lg()
            .border_1()
            .border_color(p().border)
            .bg(p().surface)
            .text_color(p().text)
            .child(head)
            .children(self.steps.map(|s| div().flex_none().px_5().py_2().child(s)))
            .child(
                div()
                    .named(SharedString::from(format!("{id}-page")))
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .px_5()
                    .py_2()
                    .children(self.body),
            )
            .child(foot);
        div()
            .named(SharedString::from(format!("{id}-veil")))
            .absolute()
            .inset_0()
            .occlude()
            .flex()
            .items_center()
            .justify_center()
            .bg(gpui_kit::black().opacity(0.45))
            .child(card)
            .into_any_element()
    }
}
