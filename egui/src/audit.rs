//! What can be clicked, for `scripts/audit.py` to find what cannot:
//! with `TVTY_EGUI_AUDIT=<file>`, each pass that changes it writes there
//! the widgets that take a click (egui's widget rectangles of the pass) and
//! the texts painted in the link colour (the theme's accent), layer by
//! layer from the back. A link no click widget covers is a dead link; a
//! click widget whose click changes nothing on screen is a dead button
//! (the script clicks each one in wbox to tell).

use egui_pop::egui::{self, Shape};
use egui_pop::prelude::Tokens;
use serde_json::json;

struct Audit {
    path: std::path::PathBuf,
    last: String,
}

/// Starts the audit when `TVTY_EGUI_AUDIT` names a file; once.
pub fn start(ctx: &egui::Context) {
    let Some(path) = std::env::var_os("TVTY_EGUI_AUDIT").filter(|p| !p.is_empty()) else {
        return;
    };
    let id = egui::Id::unique("tvty.audit.on");
    if ctx.data(|d| d.get_temp::<bool>(id)).unwrap_or(false) {
        return;
    }
    ctx.data_mut(|d| d.insert_temp(id, true));
    ctx.add_plugin(Audit { path: path.into(), last: String::new() });
}

fn rect(r: egui::Rect) -> serde_json::Value {
    json!([r.min.x.round(), r.min.y.round(), r.max.x.round(), r.max.y.round()])
}

impl egui::plugin::Plugin for Audit {
    fn debug_name(&self) -> &'static str {
        "tvty.audit"
    }

    fn on_end_pass(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let accent = Tokens::of(&ctx).accent.default;
        // Back to front: what a later layer covers is hidden.
        let layers: Vec<egui::LayerId> = ctx.memory(|m| m.layer_ids().collect());
        let order = |l: &egui::LayerId| layers.iter().position(|x| x == l).unwrap_or(0);
        let mut links = Vec::new();
        ctx.graphics(|g| {
            for layer in &layers {
                let Some(list) = g.get(*layer) else { continue };
                for clipped in list.all_entries() {
                    let Shape::Text(text) = &clipped.shape else { continue };
                    let colored = text.override_text_color == Some(accent)
                        || text.fallback_color == accent
                        || text.galley.job.sections.iter().any(|s| s.format.color == accent);
                    if !colored {
                        continue;
                    }
                    let shown = text.visual_bounding_rect().intersect(clipped.clip_rect);
                    if !shown.is_positive() {
                        continue;
                    }
                    links.push(json!({"text": text.galley.text(), "rect": rect(shown), "layer": order(layer)}));
                }
            }
        });
        let mut clicks = Vec::new();
        ctx.viewport(|vp| {
            for (layer, widgets) in vp.this_pass.widgets.layers() {
                for w in widgets {
                    if w.sense.senses_click() && w.interact_rect.is_positive() {
                        clicks.push(json!({"id": format!("{:?}", w.id), "rect": rect(w.interact_rect), "layer": order(layer), "drag": w.sense.senses_drag()}));
                    }
                }
            }
        });
        let screen = ctx.content_rect();
        let out = json!({"screen": rect(screen), "links": links, "clicks": clicks}).to_string();
        if out != self.last {
            if let Err(e) = std::fs::write(&self.path, &out) {
                log::warn!("audit: {}: {e}", self.path.display());
            }
            self.last = out;
        }
    }
}
