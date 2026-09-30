//! tvty's own widgets, over the kit's.

pub mod buttons;
pub mod chipbar;
pub mod combo;
pub mod fields;
pub mod ticket_text;
pub mod ticketref;

use gpui_kit::*;

/// An element known by its id: GPUI's own id and, while the debug control
/// is on, where it is painted (`crate::inspect`) — so a test sees it and
/// clicks it by name. Use it wherever `.id(…)` would be: nothing more is
/// done when the control is off.
pub trait Named: Sized {
    fn named(self, id: impl Into<ElementId>) -> Stateful<Self>;
}

impl Named for Div {
    fn named(self, id: impl Into<ElementId>) -> Stateful<Self> {
        let id: ElementId = id.into();
        self.children(crate::inspect::mark_if(id.to_string())).id(id)
    }
}
