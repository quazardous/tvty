//! Settings declared once: each one's key in its file, where an options
//! page shows it, what it says, and what kind of value it takes. From the
//! declarations come the pages, the reading and writing of a value by its
//! key, and the check of a file edited by hand.
//!
//! A value is reached through the set's own serde form (a TOML table), so a
//! setting needs no code of its own: `appearance.terminal_font_size` is the
//! key `terminal_font_size` of the table `appearance`. A value set back to
//! its default leaves the file, so that the default can change later.

use serde::Serialize;
use serde::de::DeserializeOwned;
use toml::{Table, Value as Toml};

/// One setting.
#[derive(Clone, Copy, Debug)]
pub struct Setting {
    /// Its path in the file: `section.key`.
    pub key: &'static str,
    /// The options page, and the group on it, that show it.
    pub page: &'static str,
    pub group: &'static str,
    pub label: &'static str,
    pub about: &'static str,
    pub kind: Kind,
}

/// What a setting takes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    /// A number between bounds, by steps.
    Number { min: f64, max: f64, step: f64, default: f64, unit: &'static str, integer: bool },
    /// On or off, each state with its word (`shown` / `hidden`).
    Toggle { default: bool, on: &'static str, off: &'static str },
    /// One of a list the program gives (themes); none: the default.
    Choice,
    /// No value: a button, and the program does the rest.
    Action { button: &'static str },
}

/// A setting's value, as a page shows it.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Number(f64),
    Toggle(bool),
    /// None: the default.
    Choice(Option<String>),
}

/// The declarations of a set, in the order pages show them.
#[derive(Clone, Copy, Debug)]
pub struct Schema(pub &'static [Setting]);

fn table<T: Serialize>(set: &T) -> Table {
    match Toml::try_from(set) {
        Ok(Toml::Table(table)) => table,
        _ => Table::new(),
    }
}

fn lookup<'a>(table: &'a Table, key: &str) -> Option<&'a Toml> {
    let mut parts = key.split('.');
    let mut here = table.get(parts.next()?)?;
    for part in parts {
        here = here.as_table()?.get(part)?;
    }
    Some(here)
}

/// Sets (or, with none, removes) the value at `key`, making the sections
/// on the way.
fn place(table: &mut Table, key: &str, value: Option<Toml>) {
    let (sections, last) = match key.rsplit_once('.') {
        Some((sections, last)) => (Some(sections), last),
        None => (None, key),
    };
    let mut here = table;
    for section in sections.into_iter().flat_map(|s| s.split('.')) {
        let entry = here.entry(section.to_string()).or_insert_with(|| Toml::Table(Table::new()));
        if !entry.is_table() {
            *entry = Toml::Table(Table::new());
        }
        here = entry.as_table_mut().expect("a table");
    }
    match value {
        Some(value) => {
            here.insert(last.to_string(), value);
        }
        None => {
            here.remove(last);
        }
    }
}

fn number(value: &Toml) -> Option<f64> {
    value.as_float().or_else(|| value.as_integer().map(|i| i as f64))
}

/// A number as its unit says: `14 px`, `1.5×`.
pub fn shown(value: f64, unit: &str) -> String {
    let text = if value.fract() == 0. { format!("{value:.0}") } else { format!("{value:.1}") };
    match unit {
        "" => text,
        "×" => format!("{text}×"),
        unit => format!("{text} {unit}"),
    }
}

impl Schema {
    pub fn get(&self, key: &str) -> Option<&'static Setting> {
        self.0.iter().find(|s| s.key == key)
    }

    /// The pages, in order.
    pub fn pages(&self) -> Vec<&'static str> {
        let mut pages: Vec<&'static str> = Vec::new();
        for setting in self.0 {
            if !pages.contains(&setting.page) {
                pages.push(setting.page);
            }
        }
        pages
    }

    /// A page's groups, in order, each with its settings.
    pub fn groups(&self, page: &str) -> Vec<(&'static str, Vec<&'static Setting>)> {
        let mut groups: Vec<(&'static str, Vec<&'static Setting>)> = Vec::new();
        for setting in self.0.iter().filter(|s| s.page == page) {
            match groups.iter_mut().find(|(g, _)| *g == setting.group) {
                Some((_, settings)) => settings.push(setting),
                None => groups.push((setting.group, vec![setting])),
            }
        }
        groups
    }

    /// The value of `key` in `set`, or its default.
    pub fn value<T: Serialize>(&self, set: &T, key: &str) -> Option<Value> {
        let setting = self.get(key)?;
        let table = table(set);
        let found = lookup(&table, key);
        Some(match setting.kind {
            Kind::Number { default, .. } => Value::Number(found.and_then(number).unwrap_or(default)),
            Kind::Toggle { default, .. } => Value::Toggle(found.and_then(Toml::as_bool).unwrap_or(default)),
            Kind::Choice => Value::Choice(found.and_then(Toml::as_str).map(String::from)),
            Kind::Action { .. } => return None,
        })
    }

    /// The default of `key`, as a [`Value`]; none for an action.
    pub fn default_value(&self, key: &str) -> Option<Value> {
        Some(match self.get(key)?.kind {
            Kind::Number { default, .. } => Value::Number(default),
            Kind::Toggle { default, .. } => Value::Toggle(default),
            Kind::Choice => Value::Choice(None),
            Kind::Action { .. } => return None,
        })
    }

    /// Whether `key` in `set` is not at its default.
    pub fn is_modified<T: Serialize>(&self, set: &T, key: &str) -> bool {
        self.value(set, key) != self.default_value(key)
    }

    /// `set` with `key` at `value` — out of the file when it is the default.
    /// A number is kept within its bounds, on its steps.
    pub fn with<T: Serialize + DeserializeOwned>(&self, set: &T, key: &str, value: Value) -> Result<T, String> {
        let setting = self.get(key).ok_or_else(|| format!("no setting {key}"))?;
        let written = match (setting.kind, value) {
            (Kind::Number { min, max, step, default, integer, .. }, Value::Number(n)) => {
                // On a step, and free of float noise (1.2, not 1.2000000000000002).
                let n = (((n - min) / step).round() * step + min).clamp(min, max);
                let n = (n * 1e6).round() / 1e6;
                if n == default {
                    None
                } else if integer {
                    Some(Toml::Integer(n.round() as i64))
                } else {
                    Some(Toml::Float(n))
                }
            }
            (Kind::Toggle { default, .. }, Value::Toggle(on)) => (on != default).then_some(Toml::Boolean(on)),
            (Kind::Choice, Value::Choice(choice)) => choice.map(Toml::String),
            (kind, value) => return Err(format!("{key}: {value:?} is not a {kind:?}")),
        };
        let mut table = table(set);
        place(&mut table, key, written);
        Toml::Table(table).try_into().map_err(|e: toml::de::Error| format!("{key}: {}", e.message()))
    }

    /// `set` with the number at `key` moved by `steps` steps.
    pub fn stepped<T: Serialize + DeserializeOwned>(&self, set: &T, key: &str, steps: i32) -> Result<T, String> {
        let Some(Kind::Number { step, .. }) = self.get(key).map(|s| s.kind) else {
            return Err(format!("{key}: not a number"));
        };
        let Some(Value::Number(now)) = self.value(set, key) else { return Err(format!("no setting {key}")) };
        self.with(set, key, Value::Number(now + step * steps as f64))
    }

    /// `set` with `key` back to its default.
    pub fn reset<T: Serialize + DeserializeOwned>(&self, set: &T, key: &str) -> Result<T, String> {
        let mut table = table(set);
        place(&mut table, key, None);
        Toml::Table(table).try_into().map_err(|e: toml::de::Error| format!("{key}: {}", e.message()))
    }

    /// What in `set` is out of its declared bounds, named.
    pub fn check<T: Serialize>(&self, set: &T) -> Result<(), String> {
        let table = table(set);
        for setting in self.0 {
            if let Kind::Number { min, max, unit, .. } = setting.kind
                && let Some(n) = lookup(&table, setting.key).and_then(number)
                && !(min..=max).contains(&n)
            {
                return Err(format!("{} = {}: from {} to {}", setting.key, shown(n, ""), shown(min, unit), shown(max, unit)));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Kind, Schema, Setting, Value};
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
    #[serde(default)]
    struct Prefs {
        look: Look,
        alerts: Alerts,
    }

    #[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
    #[serde(default)]
    struct Look {
        #[serde(skip_serializing_if = "Option::is_none")]
        font: Option<f32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        theme: Option<String>,
    }

    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    #[serde(default)]
    struct Alerts {
        max: usize,
        own: bool,
    }

    impl Default for Alerts {
        fn default() -> Self {
            Alerts { max: 5, own: true }
        }
    }

    const SCHEMA: Schema = Schema(&[
        Setting { key: "look.font", page: "Look", group: "Sizes", label: "Font", about: "", kind: Kind::Number { min: 8., max: 32., step: 1., default: 14., unit: "px", integer: false } },
        Setting { key: "look.theme", page: "Look", group: "Colours", label: "Theme", about: "", kind: Kind::Choice },
        Setting { key: "alerts.max", page: "Alerts", group: "Alerts", label: "At most", about: "", kind: Kind::Number { min: 1., max: 10., step: 1., default: 5., unit: "", integer: true } },
        Setting { key: "alerts.own", page: "Alerts", group: "Alerts", label: "Own", about: "", kind: Kind::Toggle { default: true, on: "shown", off: "hidden" } },
    ]);

    #[test]
    fn pages_and_groups_in_their_order() {
        assert_eq!(SCHEMA.pages(), vec!["Look", "Alerts"]);
        let groups = SCHEMA.groups("Look");
        assert_eq!(groups.iter().map(|(g, s)| (*g, s.len())).collect::<Vec<_>>(), vec![("Sizes", 1), ("Colours", 1)]);
    }

    #[test]
    fn a_value_by_its_key_or_its_default() {
        let prefs = Prefs::default();
        assert_eq!(SCHEMA.value(&prefs, "look.font"), Some(Value::Number(14.)));
        assert_eq!(SCHEMA.value(&prefs, "alerts.own"), Some(Value::Toggle(true)));
        assert_eq!(SCHEMA.value(&prefs, "look.theme"), Some(Value::Choice(None)));
    }

    #[test]
    fn a_value_set_by_its_key() {
        let prefs = SCHEMA.with(&Prefs::default(), "look.font", Value::Number(16.)).unwrap();
        assert_eq!(prefs.look.font, Some(16.));
        let prefs = SCHEMA.with(&prefs, "alerts.max", Value::Number(7.)).unwrap();
        assert_eq!(prefs.alerts.max, 7);
        let prefs = SCHEMA.with(&prefs, "look.theme", Value::Choice(Some("Dusk".into()))).unwrap();
        assert_eq!(prefs.look.theme.as_deref(), Some("Dusk"));
    }

    #[test]
    fn the_default_leaves_the_file() {
        let prefs = SCHEMA.with(&Prefs::default(), "look.font", Value::Number(16.)).unwrap();
        let prefs = SCHEMA.with(&prefs, "look.font", Value::Number(14.)).unwrap();
        assert_eq!(prefs.look.font, None);
        let prefs = SCHEMA.stepped(&SCHEMA.with(&prefs, "look.font", Value::Number(20.)).unwrap(), "look.font", 0).unwrap();
        assert_eq!(SCHEMA.reset(&prefs, "look.font").unwrap().look.font, None);
    }

    #[test]
    fn a_number_stays_within_its_bounds_and_steps() {
        let prefs = SCHEMA.with(&Prefs::default(), "look.font", Value::Number(99.)).unwrap();
        assert_eq!(prefs.look.font, Some(32.));
        let prefs = SCHEMA.stepped(&prefs, "look.font", 1).unwrap();
        assert_eq!(prefs.look.font, Some(32.));
        let prefs = SCHEMA.stepped(&prefs, "look.font", -3).unwrap();
        assert_eq!(prefs.look.font, Some(29.));
        let prefs = SCHEMA.with(&prefs, "alerts.max", Value::Number(3.4)).unwrap();
        assert_eq!(prefs.alerts.max, 3);
        // A fractional step writes the number people expect.
        const SPEED: Schema = Schema(&[Setting { key: "look.font", page: "", group: "", label: "", about: "", kind: Kind::Number { min: 0.2, max: 5., step: 0.2, default: 1., unit: "×", integer: false } }]);
        let prefs = SPEED.stepped(&Prefs::default(), "look.font", 6).unwrap();
        assert_eq!(prefs.look.font, Some(2.2));
    }

    #[test]
    fn modified_is_away_from_the_default() {
        let prefs = Prefs::default();
        assert!(!SCHEMA.is_modified(&prefs, "look.font"));
        let prefs = SCHEMA.with(&prefs, "look.font", Value::Number(20.)).unwrap();
        assert!(SCHEMA.is_modified(&prefs, "look.font"));
        let prefs = SCHEMA.with(&prefs, "alerts.own", Value::Toggle(false)).unwrap();
        assert!(SCHEMA.is_modified(&prefs, "alerts.own") && !SCHEMA.is_modified(&prefs, "look.theme"));
    }

    #[test]
    fn a_file_out_of_bounds_is_named() {
        let mut prefs = Prefs::default();
        prefs.look.font = Some(50.);
        assert_eq!(SCHEMA.check(&prefs).unwrap_err(), "look.font = 50: from 8 px to 32 px");
        prefs.look.font = Some(20.);
        assert!(SCHEMA.check(&prefs).is_ok());
    }
}
