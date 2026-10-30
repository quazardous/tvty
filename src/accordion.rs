//! Foldable sections, the one way tvty lists things by group: the tickets
//! by band, the sessions by state. Each section has its title — ▾ or ▸, the
//! name in small capitals, its count; a click folds it — and its own
//! scroll: the sections share the height, a long one never pushes the
//! others out of sight, and none shrinks below its title and a couple of
//! rows. When even those minimums do not fit, the whole list scrolls
//! rather than letting them overlap: put the sections in [`list`].

use crate::ui::Named as _;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use gpui_kit::component::scroll::{Scrollbar, ScrollbarAxis};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::theme::p;

// ── Heights set by hand ──────────────────────────────────────────────
//
// Dragging a section's title moves the border with the section above: that
// one gets a height of its own, kept by `list/section`. The others keep
// their share, and the last section always does: a height it got while
// another lay below would be one nothing could change. A double click on a title gives the list its shares back.

fn heights_store() -> &'static Mutex<HashMap<String, f32>> {
    static HEIGHTS: OnceLock<Mutex<HashMap<String, f32>>> = OnceLock::new();
    HEIGHTS.get_or_init(Default::default)
}

/// Where each section was laid out last: its top, its least height, and
/// the height it was given.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Placed {
    top: f32,
    least: f32,
    height: f32,
}

fn placed() -> &'static Mutex<HashMap<String, Placed>> {
    static PLACED: OnceLock<Mutex<HashMap<String, Placed>>> = OnceLock::new();
    PLACED.get_or_init(Default::default)
}

/// What a drag does for the pointer's height `wanted`: the height of the
/// section `above` (its least at least), and the heights set by hand it
/// takes room from. Dragged down, it pushes: first into the room left under
/// the last section (`empty`), then each section below gives way down to
/// its least, the nearest first — one sized by the list as the layout
/// shrinks it, one set by hand by a new height. It stops only once the
/// whole stack is down to its least. `below`: top first, with its key and
/// whether its height is set by hand.
fn plan_drag(wanted: f32, above: Placed, below: &[(String, Placed, bool)], empty: f32) -> (f32, Vec<(String, f32)>) {
    let from = above.height.max(above.least);
    let mut owed = (wanted - from).max(0.);
    let pushed = owed;
    owed -= owed.min(empty.max(0.));
    let mut taken = Vec::new();
    for (key, b, set) in below {
        if owed <= 0. {
            break;
        }
        let give = (b.height - b.least).max(0.).min(owed);
        owed -= give;
        if *set && give > 0. {
            taken.push((key.clone(), (b.height - give).round()));
        }
    }
    let height = if wanted > from { from + pushed - owed } else { wanted.max(above.least) };
    (height.round(), taken)
}

fn key(list: &str, section: &str) -> String {
    format!("{list}/{section}")
}

/// The heights set by hand, to keep (`list/section` → pixels).
pub fn heights() -> HashMap<String, f32> {
    heights_store().lock().map(|h| h.clone()).unwrap_or_default()
}

/// The heights kept from a previous run.
pub fn set_heights(heights: HashMap<String, f32>) {
    if let Ok(mut h) = heights_store().lock() {
        *h = heights;
    }
}

/// Set when the heights changed for good (a drag ended, a reset): who
/// keeps them observes it.
pub struct HeightsChanged;

impl Global for HeightsChanged {}

/// Says the heights changed for good.
pub fn commit(cx: &mut App) {
    cx.set_global(HeightsChanged);
}

/// What a title carries while dragged: the section whose height it sets.
#[derive(Clone)]
pub struct SectionDrag {
    pub list: SharedString,
    pub above: SharedString,
}

/// A drag shows nothing under the pointer.
struct NoPreview;

impl Render for NoPreview {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

/// The pointer, dragging a title, is at `y`: the section above ends there
/// (its least height kept). Answers whether a height changed.
pub fn drag_to(drag: &SectionDrag, y: Pixels) -> bool {
    let key = key(&drag.list, &drag.above);
    let prefix = format!("{}/", drag.list);
    let Ok(placed) = placed().lock().map(|p| p.clone()) else { return false };
    let Some(above) = placed.get(&key).copied() else { return false };
    let Ok(mut heights) = heights_store().lock() else { return false };
    // The heights set by hand, as they show: one that gave way (a shorter
    // window) is taken at its shown height, so that the list fits.
    for (k, stored) in heights.iter_mut().filter(|(k, _)| k.starts_with(&prefix) && **k != key) {
        if let Some(shown) = placed.get(k).map(|p| p.height.round()) {
            *stored = stored.min(shown);
        }
    }
    let mut below: Vec<(String, Placed, bool)> =
        placed.iter().filter(|(k, p)| k.starts_with(&prefix) && p.top > above.top).map(|(k, p)| (k.clone(), *p, heights.contains_key(k))).collect();
    below.sort_by(|a, b| a.1.top.total_cmp(&b.1.top));
    // The room left under the last section, in a list its sections do not fill.
    let end = placed.iter().filter(|(k, _)| k.starts_with(&prefix)).map(|(_, p)| p.top + p.height).fold(above.top + above.height, f32::max);
    let empty = list_bottoms().lock().ok().and_then(|b| b.get(drag.list.as_ref()).copied()).map_or(0., |bottom| bottom - end);
    // The pointer holds the title about its middle.
    let (height, taken) = plan_drag(f32::from(y) - above.top - TITLE_HEIGHT / 2., above, &below, empty);
    let mut changed = heights.insert(key, height) != Some(height);
    for (k, h) in taken {
        changed |= heights.insert(k, h) != Some(h);
    }
    changed
}

/// Gives a list's sections their shares back.
fn reset(list: &str) {
    let prefix = format!("{list}/");
    if let Ok(mut heights) = heights_store().lock() {
        heights.retain(|k, _| !k.starts_with(&prefix));
    }
}

/// How much more a section sized by the list gives way than one set by
/// hand, when the list is short: as good as all of it, first.
const SIZED_BY_LIST_GIVES: f32 = 10_000.;

/// A section's title line, in pixels.
pub const TITLE_HEIGHT: f32 = 28.;

/// Where each list's box ends, as last laid out: the room left under its
/// last section, which a drag may take.
fn list_bottoms() -> &'static Mutex<HashMap<String, f32>> {
    static BOTTOMS: OnceLock<Mutex<HashMap<String, f32>>> = OnceLock::new();
    BOTTOMS.get_or_init(Default::default)
}

thread_local! {
    /// Each list's scroll, kept across frames: its scrollbar follows it.
    static LIST_SCROLLS: std::cell::RefCell<HashMap<String, ScrollHandle>> = Default::default();
}

fn list_scroll(list: &str) -> ScrollHandle {
    LIST_SCROLLS.with(|s| s.borrow_mut().entry(list.to_string()).or_default().clone())
}

/// The list, its sections in, framed with its scrollbar: the sections that
/// do not all fit (heights set by hand, a short window) scroll as a whole,
/// and say so.
pub fn framed(list: Stateful<Div>, key: &str) -> Div {
    let scroll = list_scroll(key);
    div()
        .relative()
        .flex()
        .flex_col()
        .flex_1()
        .min_h_0()
        .child(list)
        .child(div().absolute().inset_0().child(Scrollbar::new(&scroll).axis(ScrollbarAxis::Vertical).viewport_from_layout()))
}

/// The column that holds the sections of `list` (the sections' `list`);
/// [`framed`] gives it its scrollbar once filled.
pub fn list(id: impl Into<ElementId>, list: &str) -> Stateful<Div> {
    let key = list.to_string();
    let scroll = list_scroll(list);
    div().named(id).relative().flex().flex_col().flex_1().min_h_0().track_scroll(&scroll).overflow_y_scroll().child(
        canvas(
            move |bounds, _, _| {
                if let Ok(mut bottoms) = list_bottoms().lock() {
                    bottoms.insert(key, f32::from(bounds.bottom()));
                }
            },
            |_, _, _, _| {},
        )
        .absolute()
        .inset_0(),
    )
}

/// One section.
pub struct Section {
    /// The list it is in, and the section above it (none for the first):
    /// dragging its title sets that one's height.
    pub list: SharedString,
    pub above: Option<SharedString>,
    /// The list's last section: no title below to drag, so it never keeps
    /// a height of its own — it takes what the others leave.
    pub last: bool,
    pub id: SharedString,
    pub title: String,
    pub count: usize,
    pub folded: bool,
    /// The least height its body keeps when unfolded (a couple of rows, or
    /// all of them when fewer).
    pub keep: f32,
    /// Its scroll, kept by the owner across renders.
    pub scroll: ScrollHandle,
    /// The rows drawn: all of them, or those of [`window`].
    pub body: Vec<AnyElement>,
    /// The height of the rows left out above and below the drawn ones, so
    /// that the scroll keeps the whole list's height.
    pub before: f32,
    pub after: f32,
    /// For a windowed section: what [`window`] drew, and whose view to draw
    /// again when the section's size or scroll turned out different.
    pub windowed: Option<Windowed>,
}

/// What a windowed section drew: the rows, their height, and its view.
pub struct Windowed {
    pub drawn: std::ops::Range<usize>,
    pub row: f32,
    pub owner: EntityId,
}

/// The rows of a section worth drawing when each is `row` pixels high:
/// those its scroll showed at the last frame, and a few around. The others
/// are not laid out at all — with hundreds of rows, that is what keeps a
/// frame cheap.
pub fn window(count: usize, row: f32, scroll: &ScrollHandle) -> std::ops::Range<usize> {
    // GPUI adds a wheel's delta to the offset before it bounds it: at the
    // end, the offset may be past it for a frame. The rows then are those
    // the bounded offset shows, not blank ones.
    let top = (-f32::from(scroll.offset().y)).clamp(0., f32::from(scroll.max_offset().y).max(0.));
    let height = match f32::from(scroll.bounds().size.height) {
        // Not laid out yet: a screenful.
        h if h <= 0. => 1200.,
        h => h,
    };
    let first = ((top / row) as usize).saturating_sub(2).min(count);
    let last = (((top + height) / row).ceil() as usize + 2).min(count);
    first..last
}

impl Section {
    /// The section, its title folding it through `on_toggle`.
    pub fn render(self, on_toggle: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Div {
        let Self { list, above, last, id, title, count, folded, keep, scroll, body, before, after, windowed } = self;
        let least = TITLE_HEIGHT + if folded { 0. } else { keep };
        let height = (!folded && !last)
            .then(|| heights_store().lock().ok()?.get(&key(&list, &id)).copied())
            .flatten()
            .map(|h| h.max(least));
        // Where it lies, for a drag of the title below it.
        let place = {
            let key = key(&list, &id);
            canvas(
                move |bounds, _, _| {
                    if let Ok(mut placed) = placed().lock() {
                        placed.insert(key, Placed { top: f32::from(bounds.origin.y), least, height: f32::from(bounds.size.height) });
                    }
                },
                |_, _, _, _| {},
            )
            .absolute()
            .size_full()
        };
        // The rows were chosen from the last frame's size and scroll. Once
        // this one is laid out, the choice is checked: a list that grew or
        // moved (rows came, a section above folded) would leave blank rows
        // until something else drew it again, so its view draws again now.
        let recheck = windowed.filter(|_| !folded).map(|w| {
            let scroll = scroll.clone();
            canvas(
                move |_, win, _| {
                    if window(count, w.row, &scroll) != w.drawn {
                        let owner = w.owner;
                        win.on_next_frame(move |_, cx| cx.notify(owner));
                    }
                },
                |_, _, _, _| {},
            )
            .absolute()
            .size_0()
        });
        // A band of its own: a line above, a darker ground, the title in
        // full colour and its count in a pill — the break between two
        // groups reads at a glance.
        let header = div()
            .saying(SharedString::from(format!("{id}-title")), format!("{} {count}", title.to_uppercase()))
            .flex()
            .flex_none()
            .items_center()
            .gap_1p5()
            .h(px(TITLE_HEIGHT))
            .px_3()
            .bg(p().surface)
            .border_t_1()
            .border_b_1()
            .border_color(p().border)
            .text_xs()
            .font_weight(FontWeight::BOLD)
            .text_color(p().text)
            .cursor_pointer()
            .hover(|d| d.bg(p().hover))
            // Dragged, it moves the border with the section above.
            .when_some(above, |d, above| {
                d.cursor(CursorStyle::ResizeUpDown)
                    .on_drag(SectionDrag { list: list.clone(), above }, |_, _, _, cx| cx.new(|_| NoPreview))
            })
            .child(div().w(px(10.)).text_color(p().accent).child(if folded { "▸" } else { "▾" }))
            .child(title.to_uppercase())
            .child(
                div()
                    .px_1p5()
                    .rounded_full()
                    .bg(p().border)
                    .font_weight(FontWeight::NORMAL)
                    .text_color(p().text)
                    .child(count.to_string()),
            )
            .on_click({
                let list = list.clone();
                move |event, window, cx| {
                    // A double click: the list's shares back.
                    if event.click_count() == 2 {
                        reset(&list);
                        commit(cx);
                    }
                    on_toggle(event, window, cx)
                }
            });
        // A wheel pushing past an end does nothing: stopped here, it does
        // not move the offset nor draw the section again.
        let ends = {
            let scroll = scroll.clone();
            canvas(
                |bounds, _, _| bounds,
                move |bounds, _, window, _| {
                    let scroll = scroll.clone();
                    window.on_mouse_event(move |event: &ScrollWheelEvent, phase, _, cx| {
                        if phase != DispatchPhase::Capture || !bounds.contains(&event.position) {
                            return;
                        }
                        let push = f32::from(event.delta.pixel_delta(px(16.)).y);
                        let at = -f32::from(scroll.offset().y);
                        let end = f32::from(scroll.max_offset().y);
                        if (push < 0. && at >= end - 0.5) || (push > 0. && at <= 0.5) {
                            cx.stop_propagation();
                        }
                    });
                },
            )
            .absolute()
            .size_full()
        };
        let keep = if folded { 0. } else { keep };
        div()
            .relative()
            .flex()
            .flex_col()
            .map(|d| match height {
                // A height set by hand gives way when the list is shorter (a
                // smaller window): the sections below keep their least
                // height in view, and their scroll, rather than going under
                // the bottom.
                // Set by hand, it keeps its height while the list holds it;
                // in a shorter one (a smaller window) it gives way too, once
                // the sections sized by the list are down to their least —
                // the last section stays in view. A drag never overflows
                // (it stops at the bottom), so nothing gives way under it.
                Some(height) => d.flex_grow(0.).flex_shrink(1.).h(px(height)),
                None => d.flex_initial().flex_shrink(SIZED_BY_LIST_GIVES),
            })
            .min_h(px(TITLE_HEIGHT + keep))
            .child(place)
            .child(header)
            .when(!folded, |d| {
                let rows = div()
                    .named(SharedString::from(format!("{id}-rows")))
                    .flex()
                    .flex_col()
                    .flex_initial()
                    .min_h_0()
                    .track_scroll(&scroll)
                    .overflow_y_scroll()
                    .when(before > 0., |d| d.child(div().flex_none().h(px(before))))
                    .children(body)
                    .when(after > 0., |d| d.child(div().flex_none().h(px(after))));
                d.child(
                    div()
                        .relative()
                        .flex()
                        .flex_col()
                        .flex_initial()
                        .min_h(px(keep))
                        .child(rows)
                        .child(ends)
                        .child(
                            div()
                                .absolute()
                                .inset_0()
                                .child(Scrollbar::new(&scroll).axis(ScrollbarAxis::Vertical).viewport_from_layout()),
                        ),
                )
                // After the rows: by then this frame's size and scroll are known.
                .children(recheck)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::{Placed, plan_drag};

    fn at(top: f32, least: f32, height: f32) -> Placed {
        Placed { top, least, height }
    }

    #[test]
    fn a_drag_down_pushes_and_the_sections_below_give_way_nearest_first() {
        let above = at(0., 56., 300.);
        let below = vec![("l/b".to_string(), at(300., 56., 200.), true), ("l/c".to_string(), at(500., 28., 128.), false)];
        // The one just below gives first, down to its least (144 to give).
        assert_eq!(plan_drag(400., above, &below, 0.), (400., vec![("l/b".to_string(), 100.)]));
        // Then the next (100 more), and no further: all at their least.
        assert_eq!(plan_drag(520., above, &below, 0.), (520., vec![("l/b".to_string(), 56.)]));
        assert_eq!(plan_drag(900., above, &below, 0.), (544., vec![("l/b".to_string(), 56.)]));
        // Room under the last section first: nothing gives.
        assert_eq!(plan_drag(400., above, &below, 150.), (400., vec![]));
        // Up: down to its own least, nothing taken.
        assert_eq!(plan_drag(10., above, &below, 0.), (56., vec![]));
    }
}
