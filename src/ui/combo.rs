//! A choice in a long list: a dropdown searched as it is typed (the kit's
//! select); the likeliest first — a project's own agents, then the others,
//! greyed.
//! One value at a time; a field that takes several (tags) shows what it
//! holds beside, and uses the combo to add.

use gpui_kit::component::select::{SearchableVec, Select, SelectItem as SearchableListItem, SelectState};

use crate::theme::p;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

/// One choice: what it stands for, and what it reads.
#[derive(Clone, Debug, PartialEq)]
pub struct Choice {
    pub value: String,
    pub label: SharedString,
    /// Less likely: greyed, after the others.
    pub muted: bool,
}

impl Choice {
    pub fn new(value: impl Into<String>, label: impl Into<SharedString>) -> Self {
        Self { value: value.into(), label: label.into(), muted: false }
    }

    /// Greyed: less likely than the others.
    pub fn muted(mut self) -> Self {
        self.muted = true;
        self
    }

    /// A choice that reads as what it stands for.
    pub fn plain(value: impl Into<String>) -> Self {
        let value = value.into();
        Self { label: value.clone().into(), value, muted: false }
    }
}

impl SearchableListItem for Choice {
    type Value = String;

    fn title(&self) -> SharedString {
        self.label.clone()
    }

    fn value(&self) -> &String {
        &self.value
    }

    fn render(&self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div().when(self.muted, |d| d.text_color(p().muted)).child(self.label.clone())
    }
}

pub type Choices = SearchableVec<Choice>;
pub type ComboState = SelectState<Choices>;
pub use gpui_kit::component::select::SelectEvent as ComboEvent;

/// A combo over `groups`, `selected` chosen.
pub fn new(choices: Vec<Choice>, selected: Option<&str>, window: &mut Window, cx: &mut App) -> Entity<ComboState> {
    let items = SearchableVec::new(choices);
    cx.new(|cx| {
        let mut state = ComboState::new(items, None, window, cx).searchable(true);
        if let Some(value) = selected {
            state.set_selected_value(&value.to_string(), window, cx);
        }
        state
    })
}

/// Its choices changed (a catalog read), `selected` chosen again.
pub fn set(state: &Entity<ComboState>, choices: Vec<Choice>, selected: Option<&str>, window: &mut Window, cx: &mut App) {
    let items = SearchableVec::new(choices);
    state.update(cx, |state, cx| {
        state.set_items(items, window, cx);
        match selected {
            Some(value) => state.set_selected_value(&value.to_string(), window, cx),
            None => state.set_selected_index(None, window, cx),
        }
    });
}

/// The combo as drawn: `placeholder` while nothing is chosen, `search` in
/// its search field.
pub fn view(state: &Entity<ComboState>, id: impl Into<ElementId>, placeholder: &str, search: &str) -> Select<Choices> {
    Select::new(state)
        .id(id)
        .placeholder(placeholder.to_string())
        .search_placeholder(search.to_string())
        .menu_max_h(px(320.))
}
