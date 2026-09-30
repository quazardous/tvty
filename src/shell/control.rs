//! The debug control's requests, carried out on the UI thread
//! (`crate::control` receives them): what is on screen, by id
//! (`crate::inspect`); clicks, pointer moves and keys put in through the
//! window, as the user's would come; and what the shell holds.
//!
//! `{"cmd": "tree"}` · `{"cmd": "query", "id": "assign"}` (ids starting so)
//! · `{"cmd": "where", "id": …}` · `{"cmd": "click", "id": …}` ·
//! `{"cmd": "hover", "id": …}` · `{"cmd": "press", "id": …}` (a press never
//! released) · `{"cmd": "selection"}` (the text selected) · `{"cmd": "key", "keys": "ctrl-enter"}` ·
//! `{"cmd": "type", "text": …}` · `{"cmd": "wait", "id": …, "ms": 5000}` ·
//! `{"cmd": "text", "id": …}` (what it says) · `{"cmd": "wait-text", "id":
//! …, "text": …, "ms": 5000}` (until it says so) ·
//! `{"cmd": "state"}` · `{"cmd": "inspector"}` · and failure paths,
//! provoked: `{"cmd": "bus-reconnect"}`, `{"cmd": "fault", "subscribe":
//! "Tickets"}` (its next subscribing fails once).

use std::time::Duration;

use futures::StreamExt as _;
use gpui_kit::*;
use serde_json::{Value, json};

use super::Shell;
use crate::control::Request;
use crate::inspect;

/// How often `wait` looks again.
const WAIT_STEP: Duration = Duration::from_millis(50);

impl Shell {
    /// Takes the debug control's requests, while the shell lives.
    pub(super) fn serve_control(mut requests: futures::channel::mpsc::UnboundedReceiver<Request>, window: &mut Window, cx: &mut Context<Self>) {
        cx.spawn_in(window, async move |this, cx| {
            while let Some(request) = requests.next().await {
                let answer = carry_out(&request.command, &this, cx).await;
                let _ = request.reply.send(answer);
            }
        })
        .detach();
    }

    /// What the shell holds, for `state`.
    fn said(&self, cx: &App) -> Value {
        json!({
            "page": self.page_shown(cx),
            "terminal": self.selected,
            "copies": self.copies.iter().collect::<Vec<_>>(),
            "open_terminals": self.terminals.keys().collect::<Vec<_>>(),
            "panel": self.panel.read(cx).said(),
            "bus": self.wire.as_ref().and_then(|w| w.hello()).is_some(),
            "subscriptions": self.live.subscriptions_said(),
            "link": self.bus_state.said(),
            "catalogs": crate::kernel::catalog::store(cx).read(cx).said(),
            "backlogs": crate::kernel::backlog::store(cx).read(cx).said(),
            "standing": crate::kernel::standing::store(cx).read(cx).said(),
            "place_bar": self.move_asked,
        })
    }
}

async fn carry_out(command: &Value, this: &WeakEntity<Shell>, cx: &mut AsyncWindowContext) -> Value {
    let cmd = command.get("cmd").and_then(Value::as_str).unwrap_or_default();
    let id = command.get("id").and_then(Value::as_str).unwrap_or_default();
    let answer = match cmd {
        "tree" | "query" => {
            let marks: Vec<Value> = inspect::marks()
                .into_iter()
                .filter(|(m, _, _)| m.starts_with(id))
                .map(|(m, b, text)| {
                    let mut element = json!({ "id": m, "x": f32::from(b.origin.x), "y": f32::from(b.origin.y), "w": f32::from(b.size.width), "h": f32::from(b.size.height) });
                    if let Some(text) = text {
                        element["text"] = json!(text);
                    }
                    element
                })
                .collect();
            Ok(json!({ "elements": marks }))
        }
        "where" => center(id).map(|p| json!({ "x": f32::from(p.x), "y": f32::from(p.y) })),
        "click" | "hover" | "press" => match center(id) {
            Ok(at) => cx
                .update(|window, cx| {
                    let moved = MouseMoveEvent { position: at, pressed_button: None, modifiers: Modifiers::default() };
                    window.dispatch_event(PlatformInput::MouseMove(moved), cx);
                    if cmd != "hover" {
                        let down = MouseDownEvent { button: MouseButton::Left, position: at, modifiers: Modifiers::default(), click_count: 1, first_mouse: false };
                        window.dispatch_event(PlatformInput::MouseDown(down), cx);
                    }
                    // A press: the button goes down and its release never
                    // comes (it was let go outside the window).
                    if cmd == "click" {
                        let up = MouseUpEvent { button: MouseButton::Left, position: at, modifiers: Modifiers::default(), click_count: 1 };
                        window.dispatch_event(PlatformInput::MouseUp(up), cx);
                    }
                    json!({ "x": f32::from(at.x), "y": f32::from(at.y) })
                })
                .map_err(|e| format!("{e:#}")),
            Err(error) => Err(error),
        },
        // The text selected in the window (the panel's, a page's), and
        // whether any is.
        "selection" => cx
            .update(|window, cx| {
                let text = gpui_kit::base::TextSelection::selected_text(window, cx);
                json!({ "selected": !text.is_empty(), "text": text })
            })
            .map_err(|e| format!("{e:#}")),
        "key" => {
            let keys = command.get("keys").and_then(Value::as_str).unwrap_or_default().to_string();
            match Keystroke::parse(&keys) {
                Ok(keystroke) => cx.update(|window, cx| json!({ "handled": window.dispatch_keystroke(keystroke, cx) })).map_err(|e| format!("{e:#}")),
                Err(error) => Err(format!("{keys}: {error:#}")),
            }
        }
        "type" => {
            let text = command.get("text").and_then(Value::as_str).unwrap_or_default().to_string();
            cx.update(|window, cx| {
                for c in text.chars() {
                    let key = if c == ' ' { "space".to_string() } else { c.to_lowercase().to_string() };
                    let keystroke = Keystroke { modifiers: Modifiers { shift: c.is_uppercase(), ..Default::default() }, key, key_char: Some(c.to_string()) };
                    window.dispatch_keystroke(keystroke, cx);
                }
                json!({ "typed": text.chars().count() })
            })
            .map_err(|e| format!("{e:#}"))
        }
        // What an element says (a button's label, a row's title), when known.
        "text" => match inspect::text_of(id) {
            Some(Some(text)) => Ok(json!({ "text": text })),
            Some(None) => Err(format!("{id}: on screen, its text is not known (give it one with `.saying`)")),
            None => Err(format!("{id}: not on screen")),
        },
        // Until an element says `text` (contains it).
        "wait-text" => {
            let wanted = command.get("text").and_then(Value::as_str).unwrap_or_default().to_string();
            let ms = command.get("ms").and_then(Value::as_u64).unwrap_or(5000);
            let until = std::time::Instant::now() + Duration::from_millis(ms);
            loop {
                let said = inspect::text_of(id).flatten();
                if said.as_deref().is_some_and(|t| t.contains(&wanted)) {
                    break Ok(json!({ "text": said }));
                }
                if std::time::Instant::now() >= until {
                    break Err(match said {
                        Some(said) => format!("{id}: says {said:?}, not {wanted:?}, after {ms} ms"),
                        None => format!("{id}: says nothing known after {ms} ms"),
                    });
                }
                cx.background_executor().timer(WAIT_STEP).await;
            }
        }
        "wait" => {
            let ms = command.get("ms").and_then(Value::as_u64).unwrap_or(5000);
            let until = std::time::Instant::now() + Duration::from_millis(ms);
            loop {
                if inspect::bounds_of(id).is_some() {
                    break Ok(json!({ "found": id }));
                }
                if std::time::Instant::now() >= until {
                    break Err(format!("{id}: not on screen after {ms} ms"));
                }
                cx.background_executor().timer(WAIT_STEP).await;
            }
        }
        "state" => this.update(cx, |shell, cx| shell.said(cx)).map_err(|e| format!("{e:#}")),
        // Failure paths, provoked: the bus dropped, a subscription refused.
        "bus-reconnect" => this
            .update(cx, |shell, _| match &shell.wire {
                Some(wire) => {
                    wire.reconnect();
                    Ok(json!({ "reconnect": "asked" }))
                }
                None => Err("no bus".to_string()),
            })
            .map_err(|e| format!("{e:#}"))
            .and_then(|r| r),
        "fault" => {
            let kind = command.get("subscribe").and_then(Value::as_str).unwrap_or_default();
            if crate::live::fail_next(kind) {
                Ok(json!({ "fails_once": kind }))
            } else {
                Err(format!("{kind:?}: no such subscription (Tickets, State, Bar, Pings, Sessions, Config, Board)"))
            }
        }
        "inspector" => toggle_inspector(cx),
        other => Err(format!("{other:?}: no such command (tree, query, text, where, click, hover, press, selection, key, type, wait, wait-text, state, inspector, bus-reconnect, fault)")),
    };
    match answer {
        Ok(value) => json!({ "ok": value }),
        Err(error) => json!({ "error": error }),
    }
}

/// The middle of `id` on screen.
fn center(id: &str) -> Result<Point<Pixels>, String> {
    let bounds = inspect::bounds_of(id).ok_or_else(|| format!("{id}: not on screen"))?;
    Ok(bounds.center())
}

/// GPUI's own inspector, in debug builds: pick an element, see its id and
/// where it was made.
fn toggle_inspector(cx: &mut AsyncWindowContext) -> Result<Value, String> {
    if !cfg!(debug_assertions) {
        return Err("GPUI's inspector is in debug builds only".into());
    }
    cx.update(|window, cx| {
        toggle_inspector_here(window, cx);
        json!({ "inspector": "toggled" })
    })
    .map_err(|e| format!("{e:#}"))
}

/// The inspector on or off in this window (its key, `debug.inspector`):
/// nothing in a release build.
pub(super) fn toggle_inspector_here(window: &mut Window, cx: &mut App) {
    #[cfg(debug_assertions)]
    window.toggle_inspector(cx);
    #[cfg(not(debug_assertions))]
    let _ = (window, cx);
}
