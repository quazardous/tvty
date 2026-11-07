//! tvty's menu: the one its icon opens at the title bar's start (F1), as
//! the GPUI build has it (`help.rs` there). egui-pop's: each entry a
//! command, in groups parted by a line, with its keys at the end.

use egui_pop::egui;
use egui_pop::model::ids::{CommandId, MenuId};
use egui_pop::model::menus::MenuItem;
use egui_pop::prelude::*;
use serde_json::json;

pub const MENU: MenuId = MenuId::from_static("tvty.menu");

/// Its entries: the command, its title, the group it is in, its key.
const ENTRIES: [(&str, &str, &str, Option<&str>); 15] = [
    ("tvty.about", "About Terminal Velocity", "1", None),
    ("tvty.settings", "Settings…", "1", Some("Ctrl+,")),
    ("tvty.new-project", "New project…", "1", None),
    ("tvty.message-all", "Message every agent…", "1", None),
    ("tvty.full-screen", "Full screen", "1", Some("F11")),
    ("tvty.documentation", "Documentation", "1", None),
    ("tvty.shortcuts", "Keyboard shortcuts", "1", None),
    ("tvty.tips", "Tips…", "1", None),
    ("tvty.whats-new", "What's new", "1", None),
    ("tvty.github", "GitHub", "2", None),
    ("tvty.issue", "Report an issue", "2", None),
    ("tvty.aiball", "aiball", "2", None),
    ("tvty.updates", "Updates…", "2", None),
    ("tvty.restart", "Restart tvty", "3", None),
    ("tvty.quit", "Quit tvty", "3", Some("Ctrl+Q")),
];

/// Declares the commands and the menu holding them.
pub fn declare(ctx: &mut Ctx<'_>) -> Result<(), PluginError> {
    for (order, (id, title, group, key)) in ENTRIES.into_iter().enumerate() {
        let command = CommandId::new(id).expect("a command id");
        ctx.command(CommandSpec::new(command.clone(), title, "tvty"), move |call| {
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

/// GPUI's menus have no icons, nor the column egui-pop keeps for them:
/// the `menus.icons` setting off, as on GNOME. Written once, at start.
pub fn no_icons(app: &AppHandle) {
    use egui_pop::model::commands::Origin;
    use egui_pop::model::ids::OwnerId;
    use egui_pop::model::values::{Change, Set, Value};
    let path = format!("settings/{}", egui_pop::api::menus::SETTING_ICONS);
    let shared = app.store();
    let mut store = egui_pop::core::lock(&shared);
    static DONE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if !DONE.swap(true, std::sync::atomic::Ordering::Relaxed) {
        let set = Set::new(&path, Change::To(Value::Choice("off".into())), Origin::User).by(&OwnerId::FRAMEWORK).untracked();
        if let Err(e) = store.set(set) {
            eprintln!("menus.icons: {e}");
        }
    }
}

/// Opens the menu under `anchor` (tvty's icon).
pub fn open(app: &AppHandle, anchor: egui::Rect) {
    egui_pop::api::menus::popup_at(app, &MENU, anchor);
}
