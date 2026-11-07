//! tvty's menu: the one its icon opens at the title bar's start (F1), as
//! the GPUI build has it (`help.rs` there). egui-pop's: each entry a
//! command, in groups parted by a line, with its keys at the end.

use egui_pop::egui;
use egui_pop::model::ids::{CommandId, MenuId};
use egui_pop::model::menus::MenuItem;
use egui_pop::prelude::*;
use serde_json::json;

pub const MENU: MenuId = MenuId::from_static("tvty.menu");

/// Its entries: the command, its title, the group it is in, its key, the
/// note at its end, whether it opens a page in the browser (a ↗).
type Entry = (&'static str, &'static str, &'static str, Option<&'static str>, Option<&'static str>, bool);
const ENTRIES: [Entry; 15] = [
    ("tvty.about", "About Terminal Velocity", "1", None, Some(concat!("v", env!("CARGO_PKG_VERSION"))), false),
    ("tvty.settings", "Settings…", "1", Some("Ctrl+,"), None, false),
    ("tvty.new-project", "New project…", "1", None, None, false),
    ("tvty.message-all", "Message every agent…", "1", None, None, false),
    ("tvty.full-screen", "Full screen", "1", Some("F11"), None, false),
    ("tvty.documentation", "Documentation", "1", None, None, true),
    ("tvty.shortcuts", "Keyboard shortcuts", "1", None, None, false),
    ("tvty.tips", "Tips…", "1", None, None, false),
    ("tvty.whats-new", "What's new", "1", None, None, true),
    ("tvty.github", "GitHub", "2", None, None, true),
    ("tvty.issue", "Report an issue", "2", None, None, true),
    ("tvty.aiball", "aiball", "2", None, Some("the board"), true),
    ("tvty.updates", "Updates…", "2", None, None, false),
    ("tvty.restart", "Restart tvty", "3", None, Some("sessions kept"), false),
    ("tvty.quit", "Quit tvty", "3", Some("Ctrl+Q"), None, false),
];

/// Declares the commands and the menu holding them; F1 opens it.
pub fn declare(ctx: &mut Ctx<'_>) -> Result<(), PluginError> {
    let open_menu = CommandId::new("tvty.menu").expect("a command id");
    ctx.command(CommandSpec::new(open_menu.clone(), "Menu", "tvty"), |call| {
        // Under tvty's icon, where a click opens it.
        open(&call.app);
        Ok(json!(null))
    })?;
    ctx.keybinding("F1", open_menu, "", egui_pop::model::keymap::Layer::Default)?;
    for (order, (id, title, group, key, note, link)) in ENTRIES.into_iter().enumerate() {
        let command = CommandId::new(id).expect("a command id");
        let mut spec = CommandSpec::new(command.clone(), title, "tvty").opens_link(link);
        if let Some(note) = note {
            spec = spec.menu_note(note);
        }
        ctx.command(spec, move |call| {
            let egui = call.app.egui();
            match id {
                "tvty.message-all" => crate::message::toggle(&egui),
                "tvty.settings" => crate::settings::set_open(&egui, true),
                "tvty.full-screen" => {
                    let full = egui.input(|i| i.viewport().fullscreen.unwrap_or(false));
                    egui.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!full));
                }
                "tvty.quit" => egui.send_viewport_cmd(egui::ViewportCommand::Close),
                // The rest are tvty's own pages and links: not in this clone.
                _ => {}
            }
            Ok(json!(null))
        })?;
        ctx.menu_item(MenuItem::command(MENU.clone(), group, order as i32, command.clone()))?;
        if let Some(key) = key {
            ctx.keybinding(key, command, "", egui_pop::model::keymap::Layer::Default)?;
        }
    }
    Ok(())
}

/// Opens the menu under tvty's icon, where GPUI puts it: its edge 8 inside
/// the window, its top on 38.
pub fn open(app: &AppHandle) {
    let anchor = egui::Rect::from_min_max(egui::pos2(8.0, 10.0), egui::pos2(37.0, 38.0));
    egui_pop::api::menus::popup_at(app, &MENU, anchor);
}
