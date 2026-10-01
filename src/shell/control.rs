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
//! `{"cmd": "frame", "park": true}` (until the window has drawn again) ·
//! `{"cmd": "settle", "ms": 5000}` (until no animation runs) ·
//! `{"cmd": "dblclick", "id": …}` · `{"cmd": "window", "what": "maximize"}`
//! (or `restore`, or `size` with `w`, `h`) · `{"cmd": "hold", "keys":
//! "ctrl"}` (modifiers held until others are said; none: let go) ·
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
/// How long `frame` waits for the window to draw.
const FRAME_WAIT: Duration = Duration::from_millis(3000);

impl Shell {
    /// Takes the debug control's requests, while the shell lives.
    pub(super) fn serve_control(mut requests: futures::channel::mpsc::UnboundedReceiver<Request>, window: &mut Window, cx: &mut Context<Self>) {
        cx.spawn_in(window, async move |this, cx| {
            while let Some(request) = requests.next().await {
                let answer = if request.gestures { carry_out(&request.command, &this, cx).await } else { read(&request.command, &this, cx).await };
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
            "terminal_frozen": self.selected.as_ref().and_then(|s| self.terminals.get(s)).is_some_and(|t| t.read(cx).frozen()),
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
            // This machine as aiball names it, and the sessions as the panel
            // placed them: what a missing row is read from.
            "machine": self.machine,
            "projects": self.board.projects.iter().map(|p| json!({
                "name": p.name,
                "terminals": p.terminals.iter().map(|t| json!({
                    "session": t.session,
                    "agent": t.agent,
                    "hosted": t.attach.is_some(),
                    "state": t.status.as_ref().map(|s| s.state.clone()),
                    "online": t.status.as_ref().map(|s| s.online),
                })).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
            // What is open in the views, said rather than guessed from the
            // buttons on screen.
            "views": {
                "sidebar_tab": if self.settings.layout.sidebar_tab == "workspaces" { "workspaces" } else { "sessions" },
                "agent_line": if self.move_asked.is_some() {
                    Some("place")
                } else if self.afk_menu {
                    Some("afk")
                } else if self.restart_asked.is_some() {
                    Some("update")
                } else {
                    None
                },
                "picker": self.picker.as_ref().map(|p| p.said()),
                "dialog": self.picker.as_ref().filter(|p| p.restarting()).map(|_| "restart-loops"),
                "menu": if self.theme_menu {
                    Some("theme")
                } else if self.help_menu {
                    Some("help")
                } else {
                    None
                },
                "megaphone": self.megaphone.is_some(),
                "new_ticket": self.new_ticket.is_some() && self.new_ticket_shown,
                "workspace_renaming": self.workspace_renaming,
                "workspace_deleting": self.workspace_deleting,
            },
            // The compositor this tvty draws on: a test's clipboard is that
            // one's, never the desktop's.
            "wayland_display": std::env::var("WAYLAND_DISPLAY").ok(),
        })
    }
}

/// What the inspection answers: what is read, never a gesture. The marks
/// of what is on screen are taken from its first ask on, after a frame.
async fn read(command: &Value, this: &WeakEntity<Shell>, cx: &mut AsyncWindowContext) -> Value {
    let cmd = command.get("cmd").and_then(Value::as_str).unwrap_or_default();
    match cmd {
        "state" | "focus" => {}
        "tree" | "query" | "where" | "text" => {
            if !inspect::enabled() {
                inspect::enable();
                if let Err(error) = drawn(cx, false).await {
                    return json!({ "error": error });
                }
            }
        }
        other => return json!({ "error": format!("{other:?}: not here, which only reads (state, focus, tree, query, where, text)") }),
    }
    carry_out(command, this, cx).await
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
        "click" | "dblclick" | "hover" | "press" => match center(id) {
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
                    if matches!(cmd, "click" | "dblclick") {
                        let up = MouseUpEvent { button: MouseButton::Left, position: at, modifiers: Modifiers::default(), click_count: 1 };
                        window.dispatch_event(PlatformInput::MouseUp(up), cx);
                    }
                    // The second click of a double one, counted so.
                    if cmd == "dblclick" {
                        let down = MouseDownEvent { button: MouseButton::Left, position: at, modifiers: Modifiers::default(), click_count: 2, first_mouse: false };
                        window.dispatch_event(PlatformInput::MouseDown(down), cx);
                        let up = MouseUpEvent { button: MouseButton::Left, position: at, modifiers: Modifiers::default(), click_count: 2 };
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
        // Until the window has drawn again: what changed so far is on
        // screen, a capture taken now shows it. `park`: the pointer is put
        // outside the window first, so that no tooltip covers anything.
        "frame" => {
            let park = command.get("park").and_then(Value::as_bool).unwrap_or(false);
            drawn(cx, park).await.map(|()| json!({ "drawn": true }))
        }
        // Until the screen holds still: no animation of tvty's running
        // (the terminal sliding in, the slider's stacks, a notice coming),
        // and drawn once more since.
        "settle" => {
            let ms = command.get("ms").and_then(Value::as_u64).unwrap_or(5000);
            let park = command.get("park").and_then(Value::as_bool).unwrap_or(false);
            let until = std::time::Instant::now() + Duration::from_millis(ms);
            let mut waited = 0;
            loop {
                if let Err(error) = drawn(cx, park && waited == 0).await {
                    break Err(error);
                }
                let running = inspect::still_animating();
                if running.is_empty() {
                    break Ok(json!({ "settled": true, "waited_for": waited }));
                }
                if std::time::Instant::now() >= until {
                    break Err(format!("still moving after {ms} ms: {}", running.join(", ")));
                }
                waited += 1;
                cx.background_executor().timer(WAIT_STEP).await;
            }
        }
        // The window itself: maximized, put back, or given a size — asked
        // of the window, not aimed at its title bar.
        "window" => {
            let what = command.get("what").and_then(Value::as_str).unwrap_or_default().to_string();
            let (w, h) = (command.get("w").and_then(Value::as_f64), command.get("h").and_then(Value::as_f64));
            let asked = cx
                .update(|window, _| {
                    match (what.as_str(), w, h) {
                        ("maximize", _, _) if !window.is_maximized() => window.zoom_window(),
                        ("restore", _, _) if window.is_maximized() => window.zoom_window(),
                        ("maximize" | "restore", _, _) => {}
                        ("size", Some(w), Some(h)) => window.resize(size(px(w as f32), px(h as f32))),
                        _ => return Err(format!("{what:?}: maximize, restore, or size W H")),
                    }
                    Ok((window.is_maximized(), window.viewport_size()))
                })
                .map_err(|e| format!("{e:#}"))
                .and_then(|r| r);
            match asked {
                Err(error) => Err(error),
                // The compositor answers a moment later: said once the
                // window is as asked (or as it is after a second).
                Ok(before) => {
                    let until = std::time::Instant::now() + Duration::from_millis(1000);
                    loop {
                        let _ = drawn(cx, false).await;
                        let now = cx.update(|window, _| (window.is_maximized(), window.viewport_size())).map_err(|e| format!("{e:#}"));
                        let Ok(now) = now else { break now.map(|_| json!({})) };
                        let as_asked = match what.as_str() {
                            "maximize" => now.0,
                            "restore" => !now.0,
                            _ => now != before || (Some(f64::from(f32::from(now.1.width))), Some(f64::from(f32::from(now.1.height)))) == (w, h),
                        };
                        if as_asked || std::time::Instant::now() >= until {
                            break Ok(json!({ "maximized": now.0, "w": f32::from(now.1.width), "h": f32::from(now.1.height) }));
                        }
                        cx.background_executor().timer(WAIT_STEP).await;
                    }
                }
            }
        }
        // Modifiers held from now on, until others are said (none: let
        // go): what lives while a key is held — the slider under Ctrl — is
        // there between two commands. `key` then takes them into its own.
        "hold" => {
            let keys = command.get("keys").and_then(Value::as_str).unwrap_or_default().to_string();
            let mut modifiers = Modifiers::default();
            let mut unknown = None;
            for key in keys.split(['-', '+', ' ']).filter(|k| !k.is_empty()) {
                match key {
                    "ctrl" | "control" => modifiers.control = true,
                    "shift" => modifiers.shift = true,
                    "alt" => modifiers.alt = true,
                    "super" | "cmd" | "platform" => modifiers.platform = true,
                    other => unknown = Some(other.to_string()),
                }
            }
            match unknown {
                Some(other) => Err(format!("{other:?}: not a modifier (ctrl, shift, alt, super)")),
                None => cx
                    .update(|window, cx| {
                        let event = ModifiersChangedEvent { modifiers, capslock: Default::default() };
                        window.dispatch_event(PlatformInput::ModifiersChanged(event), cx);
                        json!({ "held": keys })
                    })
                    .map_err(|e| format!("{e:#}")),
            }
        }
        "state" => this.update(cx, |shell, cx| shell.said(cx)).map_err(|e| format!("{e:#}")),
        // Where the keys are, named as the focus trace names it.
        "focus" => this
            .update_in(cx, |shell, window, cx| json!({ "to": shell.focus_owner(window, cx), "window_active": window.is_window_active() }))
            .map_err(|e| format!("{e:#}")),
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
        other => Err(format!("{other:?}: no such command (tree, query, frame, settle, text, where, click, dblclick, hover, press, window, hold, selection, key, type, wait, wait-text, state, focus, inspector, bus-reconnect, fault)")),
    };
    match answer {
        Ok(value) => json!({ "ok": value }),
        Err(error) => json!({ "error": error }),
    }
}

/// Until the window has drawn again: what changed so far is on screen.
/// `park`: the pointer put outside the window first.
async fn drawn(cx: &mut AsyncWindowContext, park: bool) -> Result<(), String> {
    let (drawn, was_drawn) = futures::channel::oneshot::channel::<()>();
    cx.update(|window, cx| {
        if park {
            let away = MouseMoveEvent { position: point(px(-10.), px(-10.)), pressed_button: None, modifiers: Modifiers::default() };
            window.dispatch_event(PlatformInput::MouseMove(away), cx);
        }
        // Two frames: the first draws what changed, the second's start
        // says the first is done.
        window.refresh();
        window.on_next_frame(move |window, _| {
            window.refresh();
            window.on_next_frame(move |_, _| {
                let _ = drawn.send(());
            });
        });
    })
    .map_err(|e| format!("{e:#}"))?;
    let late = cx.background_executor().timer(FRAME_WAIT);
    match futures::future::select(was_drawn, Box::pin(late)).await {
        futures::future::Either::Left(_) => Ok(()),
        futures::future::Either::Right(_) => Err(format!("no frame drawn in {} ms (a window not shown draws none)", FRAME_WAIT.as_millis())),
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
