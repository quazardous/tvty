//! A ticket's words as they are written: its title, and its body written or
//! previewed (Write / Preview), with the names that fit an `@` offered under
//! the words and an image pasted (Ctrl+V) uploaded to aiball, its link put
//! in the body. One editor for a new ticket and a ticket edited in place;
//! what surrounds it (the button that files or saves, the error) is the
//! view's own.

use crate::ui::Named as _;
use gpui_kit::component::input::{Input, InputEvent, InputState, Paste, Textarea, TextareaState};
use gpui_kit::component::text::TextView;
use gpui_kit::*;

use crate::aiball::Aiball;
use crate::theme::p;
use crate::{composer, field};

/// The tallest a picture of the preview is drawn.
const PREVIEW_PICTURE_HEIGHT: f32 = 600.;

/// What the editor says to its view.
pub enum TicketTextEvent {
    /// Ctrl+Enter, from the title or the body: file it, or save it.
    Submit,
    /// A pasted image could not be uploaded.
    Failed(String),
}

pub struct TicketText {
    aiball: Aiball,
    /// Its elements' ids start with it.
    id: &'static str,
    title: Entity<InputState>,
    body: Entity<TextareaState>,
    /// The body shown as it will read (the Preview tab).
    preview: bool,
    /// The images of the body, read for its preview.
    images: crate::images::Cache,
    mentions: Vec<String>,
    /// An image being uploaded.
    uploading: bool,
}

impl EventEmitter<TicketTextEvent> for TicketText {}

impl TicketText {
    /// `lines`: the body's height, least and most, as it grows.
    pub fn new(aiball: Aiball, id: &'static str, lines: (usize, usize), window: &mut Window, cx: &mut Context<Self>) -> Self {
        let title = cx.new(|cx| InputState::new(window, cx).placeholder(crate::t!("fields-title")));
        let body = cx.new(|cx| {
            TextareaState::new(window, cx)
                .placeholder(crate::t!("fields-body"))
                .auto_grow(lines.0, lines.1)
        });
        // @-mentions are offered as the body is typed; Ctrl+Enter submits.
        cx.subscribe_in(&body, window, |_: &mut Self, _, event: &InputEvent, _, cx| match event {
            InputEvent::Change => cx.notify(),
            InputEvent::PressEnter { secondary: true, .. } => cx.emit(TicketTextEvent::Submit),
            _ => {}
        })
        .detach();
        cx.subscribe_in(&title, window, |_: &mut Self, _, event: &InputEvent, _, cx| {
            if matches!(event, InputEvent::PressEnter { secondary: true, .. }) {
                cx.emit(TicketTextEvent::Submit);
            }
        })
        .detach();
        let reader = aiball.clone();
        cx.spawn(async move |this, cx| {
            let mentions = cx.background_executor().spawn(async move { reader.mention_suggestions() }).await;
            if let Ok(mentions) = mentions {
                let _ = this.update(cx, |text, _| text.mentions = mentions);
            }
        })
        .detach();
        Self { aiball, id, title, body, preview: false, images: Default::default(), mentions: Vec::new(), uploading: false }
    }

    pub fn title(&self, cx: &App) -> String {
        self.title.read(cx).value().to_string()
    }

    pub fn body(&self, cx: &App) -> String {
        self.body.read(cx).value().to_string()
    }

    /// Something written in it.
    pub fn begun(&self, cx: &App) -> bool {
        !self.title(cx).trim().is_empty() || !self.body(cx).trim().is_empty()
    }

    /// An image is being uploaded.
    pub fn uploading(&self) -> bool {
        self.uploading
    }

    /// Filled with `title` and `body`, written (not previewed).
    pub fn set(&mut self, title: &str, body: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.title.update(cx, |input, cx| input.set_value(title.to_string(), window, cx));
        self.body.update(cx, |input, cx| input.set_value(body.to_string(), window, cx));
        self.preview = false;
        cx.notify();
    }

    pub fn focus_title(&self, window: &mut Window, cx: &mut App) {
        self.title.update(cx, |input, cx| input.focus(window, cx));
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
        self.uploading = true;
        cx.notify();
        cx.spawn(async move |this, cx| {
            let uploaded = cx
                .background_executor()
                .spawn(async move { aiball.upload(&bytes, &content_type, &name) })
                .await;
            let _ = cx.update_window(window_handle, |_, window, cx| {
                let _ = this.update(cx, |text, cx| {
                    text.uploading = false;
                    match uploaded {
                        Ok(url) => {
                            let (before, after) = field::around_cursor(&text.body, cx);
                            field::insert_at_cursor(&text.body, &composer::image_snippet(&before, &after, &url), window, cx);
                        }
                        Err(error) => cx.emit(TicketTextEvent::Failed(format!("{error:#}"))),
                    }
                    cx.notify();
                });
            });
        })
        .detach();
        true
    }

    fn set_preview(&mut self, on: bool, cx: &mut Context<Self>) {
        self.preview = on;
        if on {
            // Its pictures, read once; the preview draws them when they came.
            let text = self.body(cx);
            let (aiball, images) = (self.aiball.clone(), self.images.clone());
            cx.spawn(async move |this, cx| {
                let came = cx.background_executor().spawn(async move { crate::images::load(&text, &aiball, &images) }).await;
                if came {
                    let _ = this.update(cx, |_, cx| cx.notify());
                }
            })
            .detach();
        }
        cx.notify();
    }

    /// The body as it will read: its words, and its pictures alone on their
    /// line drawn at the column's width (never beyond their size), as the
    /// thread draws them; those not read yet say so.
    fn preview_view(&self, text: &str) -> Div {
        use crate::images::Segment;
        let text = crate::images::preview(text, &self.images, false);
        let mut col = div().flex().flex_col().gap_2();
        for (i, segment) in crate::images::segments(&text, &self.images).into_iter().enumerate() {
            col = match segment {
                Segment::Text(md) => col.child(TextView::markdown(SharedString::from(format!("{}-preview-{i}", self.id)), md).selectable(true)),
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
}

impl Render for TicketText {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let id = self.id;
        let mentions = composer::typed_mention(&field::around_cursor(&self.body, cx).0).map(|typed| {
            composer::mention_chips(&self.mentions, &typed, &format!("{id}-mention"), cx, |text: &mut Self, name, window, cx| {
                text.complete_mention(name, window, cx)
            })
        });
        div()
            .flex()
            .flex_col()
            .gap_2()
            .min_h_0()
            // Ctrl+Enter submits, from the title too.
            .capture_key_down(cx.listener(|_, event: &KeyDownEvent, _, cx| {
                let keystroke = &event.keystroke;
                if keystroke.modifiers.control && keystroke.key == "enter" {
                    cx.emit(TicketTextEvent::Submit);
                    cx.stop_propagation();
                }
            }))
            // A paste with an image goes to aiball, its link to the body.
            .capture_action(cx.listener(|text, _: &Paste, window, cx| {
                if text.paste_image(window, cx) {
                    cx.stop_propagation();
                }
            }))
            .child(crate::focusmode::on_hover(div().child(Input::new(&self.title)), self.title.read(cx).focus_handle(cx)))
            .child(composer::write_tabs(&format!("{id}-body"), self.preview, cx, |text: &mut Self, on, cx| text.set_preview(on, cx)))
            .child(if self.preview {
                let text = self.body(cx);
                // Its own height; shrunk, and scrolled, when the page is short.
                div()
                    .named(SharedString::from(format!("{id}-body-preview")))
                    .min_h_0()
                    .overflow_y_scroll()
                    .p_3()
                    .rounded_md()
                    .border_1()
                    .border_color(p().border)
                    .child(if text.trim().is_empty() {
                        div().text_color(p().muted).child(crate::t!("tickets-nothing-to-preview")).into_any_element()
                    } else {
                        self.preview_view(&text).into_any_element()
                    })
                    .into_any_element()
            } else {
                // A composer: Ctrl+Enter submits (keymap's ComposerSend), no
                // new line put in.
                div()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .key_context(crate::keymap::COMPOSER)
                    .on_action(cx.listener(|_, _: &crate::keymap::ComposerSend, _, cx| cx.emit(TicketTextEvent::Submit)))
                    .child(crate::focusmode::on_hover(div().child(Textarea::new(&self.body)), self.body.read(cx).focus_handle(cx)))
                    .children(mentions)
                    .into_any_element()
            })
    }
}
