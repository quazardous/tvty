//! tvty — Terminal Velocity. For now, only the empty frame of the window:
//! projects on the left, the terminal in the middle, the ticket panel on the
//! right (see docs/UX.md).

use gpui_kit::component::*;
use gpui_kit::*;

struct Shell;

impl Render for Shell {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .h_flex()
            .size_full()
            .bg(rgb(0x1e1e1e))
            .text_color(rgb(0xd4d4d4))
            .child(
                div()
                    .v_flex()
                    .w(px(200.))
                    .h_full()
                    .p_2()
                    .bg(rgb(0x252526))
                    .child("Projects"),
            )
            .child(div().flex_1().h_full().p_2().child("Terminal"))
            .child(
                div()
                    .v_flex()
                    .w(relative(1. / 3.))
                    .h_full()
                    .p_2()
                    .bg(rgb(0x252526))
                    .child("Tickets"),
            )
    }
}

fn main() {
    gpui_kit::application().run(|cx| {
        gpui_kit::init(cx);
        cx.spawn(async move |cx| {
            cx.open_window(
                WindowOptions {
                    titlebar: Some(TitlebarOptions {
                        title: Some("tvty".into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |window, cx| {
                    let view = cx.new(|_| Shell);
                    cx.new(|cx| Root::new(view, window, cx))
                },
            )
            .expect("failed to open the window");
        })
        .detach();
    });
}
