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
    fn named(self, id: impl Into<ElementId>) -> Stateful<Self> {
        self.named_saying(id, None)
    }

    /// [`Named::named`], with what the element says (a row's title, a
    /// chip's label): a test reads it by the element's name.
    fn saying(self, id: impl Into<ElementId>, text: impl Into<String>) -> Stateful<Self> {
        self.named_saying(id, Some(text.into()))
    }

    fn named_saying(self, id: impl Into<ElementId>, text: Option<String>) -> Stateful<Self>;
}

impl Named for Div {
    fn named_saying(self, id: impl Into<ElementId>, text: Option<String>) -> Stateful<Self> {
        let id: ElementId = id.into();
        let mark = crate::inspect::enabled().then(|| crate::inspect::mark_saying(id.to_string(), text));
        self.children(mark).id(id)
    }
}

/// What a label says, when it is plain text: a button built from it is
/// read by a test. An element (an icon, a row of things) says nothing.
pub trait Said: IntoElement {
    fn said(&self) -> Option<String> {
        None
    }
}

impl Said for &'static str {
    fn said(&self) -> Option<String> {
        Some(self.to_string())
    }
}

impl Said for String {
    fn said(&self) -> Option<String> {
        Some(self.clone())
    }
}

impl Said for SharedString {
    fn said(&self) -> Option<String> {
        Some(self.to_string())
    }
}

impl Said for Div {}
impl Said for Stateful<Div> {}
impl Said for Svg {}
impl Said for AnyElement {}
impl Said for Img {}
