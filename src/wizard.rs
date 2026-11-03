//! A setting's wizard, as aiball describes and runs it (`project.setting_wizard_*`,
//! followed on `wizard.<run>.state`): the questions asked before the start,
//! then the steps aiball runs. tvty knows nothing of the setting: it says
//! the questions and the steps in its own words from their codes and facts,
//! and falls back on aiball's English sentence for a code it does not know.

use serde::Deserialize;
use serde_json::{Value, json};

use crate::aiball::Aiball;

/// What the wizard would ask and do, before anything runs.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct Preview {
    pub key: String,
    #[serde(default)]
    pub value: Value,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub loops: Vec<PreviewLoop>,
    #[serde(default)]
    pub questions: Vec<Question>,
    #[serde(default)]
    pub steps: Vec<Step>,
    #[serde(default)]
    pub fallback: Option<String>,
    #[serde(default)]
    pub fallback_code: Option<String>,
    #[serde(default)]
    pub fallback_facts: Value,
}

/// A running loop of the folder, which the run stops and starts again.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct PreviewLoop {
    pub name: String,
    #[serde(default)]
    pub agent: Option<String>,
    #[serde(default)]
    pub busy: bool,
}

/// A question asked before the start: `confirm` (answered `true`) or
/// `choose` (answered with one of its options' key).
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct Question {
    pub id: String,
    pub kind: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub options: Vec<Answer>,
    #[serde(default)]
    pub default: Option<String>,
    #[serde(default)]
    pub facts: Value,
}

/// One of a `choose` question's answers.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct Answer {
    pub key: String,
    #[serde(default)]
    pub label: String,
}

/// A step, as planned or as it runs.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct Step {
    pub id: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub point_of_no_return: bool,
    /// `pending`, `running`, `done`, `failed`, `skipped`; none in a preview.
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub said: Option<String>,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub details: Value,
}

/// A run, whole, as `wizard.<run>.state` carries it.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct Run {
    pub run: String,
    #[serde(default)]
    pub cwd: String,
    #[serde(default)]
    pub key: String,
    #[serde(default)]
    pub value: Value,
    /// `running`, `done`, `failed`, `cancelled`, `interrupted`.
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub steps: Vec<Step>,
    #[serde(default)]
    pub fallback: Option<Fallback>,
}

/// What the fallback did, once it applied.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct Fallback {
    #[serde(default)]
    pub ran: bool,
    #[serde(default)]
    pub said: Option<String>,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub details: Value,
}

impl Run {
    pub fn running(&self) -> bool {
        self.status == "running"
    }

    /// Whether it may still be cancelled: running, its point of no return
    /// not begun.
    pub fn cancellable(&self) -> bool {
        self.running() && !self.steps.iter().any(|s| s.point_of_no_return && s.status.as_deref().is_some_and(|st| st != "pending"))
    }
}

/// What refused a preview or a start, said: a dir a loop would refuse in
/// tvty's words, anything else as aiball said it.
pub fn refusal_said(error: &anyhow::Error) -> String {
    match error.downcast_ref::<crate::wire::Refused>() {
        Some(refused) if refused.code == "CONFIG_DIR_NOT_READY" => {
            let dir = refused.details.get("config_dir").map(path).filter(|d| !d.is_empty());
            crate::t!("wizard-not-ready", dir = dir.unwrap_or_default())
        }
        _ => format!("{error:#}"),
    }
}

pub fn preview(aiball: &Aiball, cwd: &str, key: &str, value: &Value) -> anyhow::Result<Preview> {
    aiball.call("project.setting_wizard_preview", json!({ "cwd": cwd, "key": key, "value": value }))
}

pub fn start(aiball: &Aiball, cwd: &str, key: &str, value: &Value, answers: &serde_json::Map<String, Value>) -> anyhow::Result<Run> {
    aiball.call("project.setting_wizard_start", json!({ "cwd": cwd, "key": key, "value": value, "answers": answers }))
}

pub fn cancel(aiball: &Aiball, run: &str) -> anyhow::Result<Run> {
    aiball.call("project.setting_wizard_cancel", json!({ "run": run }))
}

/// The answers to send: each question's chosen key, `true` for a confirm.
/// None while one is unanswered.
pub fn answers(questions: &[Question], chosen: &std::collections::HashMap<String, String>, confirmed: bool) -> Option<serde_json::Map<String, Value>> {
    let mut out = serde_json::Map::new();
    for q in questions {
        let answer = match q.kind.as_str() {
            "confirm" => confirmed.then_some(Value::Bool(true))?,
            _ => Value::String(chosen.get(&q.id).cloned().or_else(|| q.default.clone())?),
        };
        out.insert(q.id.clone(), answer);
    }
    Some(out)
}

// ── Said in tvty's words ────────────────────────────────────────────────

/// A path as tvty shows it: `~` for the home.
fn path(value: &Value) -> String {
    value.as_str().map(crate::shell::home_short).unwrap_or_default()
}

/// A list of a fact, said: names, pids, or each item's `name` / `agent` / `pid`.
fn list(value: &Value) -> String {
    let item = |v: &Value| -> Option<String> {
        match v {
            Value::String(s) => Some(s.clone()),
            Value::Number(n) => Some(n.to_string()),
            Value::Object(o) => ["agent", "name", "loop", "pid"].iter().find_map(|k| o.get(*k)).and_then(|v| match v {
                Value::String(s) => Some(s.clone()),
                Value::Number(n) => Some(n.to_string()),
                _ => None,
            }),
            _ => None,
        }
    };
    match value {
        Value::Array(items) => items.iter().filter_map(item).collect::<Vec<_>>().join(", "),
        other => item(other).unwrap_or_default(),
    }
}

fn count(value: &Value) -> usize {
    value.as_array().map_or(0, Vec::len)
}

/// The wizard's title.
pub fn title_said(key: &str, english: &str) -> String {
    match key {
        "claude.config_dir" => crate::t!("wizard-title-config-dir"),
        _ => english.to_string(),
    }
}

/// A question, said; the loops it names by their agent, as `loops` has them.
pub fn question_said(q: &Question, loops: &[PreviewLoop]) -> String {
    let f = &q.facts;
    let names = || {
        let named: Vec<String> = f["loops"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(|name| loops.iter().find(|l| l.name == name).and_then(|l| l.agent.clone()).unwrap_or_else(|| name.to_string()))
            .collect();
        named.join(", ")
    };
    match q.id.as_str() {
        "confirm" if count(&f["loops"]) == 0 => crate::t!("wizard-q-confirm-none", target = path(&f["target"])),
        "confirm" => crate::t!("wizard-q-confirm", loops = count(&f["loops"]), names = names(), target = path(&f["target"])),
        "fork" => crate::t!("wizard-q-fork", target = path(&f["target"]), current = path(&f["current"])),
        "move_sessions" => crate::t!("wizard-q-move", from = path(&f["from"]), to = path(&f["to"])),
        _ => q.label.clone(),
    }
}

/// One of a question's answers, said.
pub fn answer_said(q: &Question, a: &Answer) -> String {
    let f = &q.facts;
    match (q.id.as_str(), a.key.as_str()) {
        ("fork", "fork") => crate::t!("wizard-a-fork"),
        ("fork", "keep") => crate::t!("wizard-a-keep", target = path(&f["target"])),
        ("move_sessions", "yes") => crate::t!("wizard-a-move-yes"),
        ("move_sessions", "no") => crate::t!("wizard-a-move-no"),
        _ => a.label.clone(),
    }
}

/// A step's name.
pub fn step_said(s: &Step) -> String {
    match s.id.as_str() {
        "idle" => crate::t!("wizard-step-idle"),
        "stop" => crate::t!("wizard-step-stop"),
        "free" => crate::t!("wizard-step-free"),
        "ready" => crate::t!("wizard-step-ready"),
        "write" => crate::t!("wizard-step-write"),
        "start" => crate::t!("wizard-step-start"),
        _ => s.label.clone(),
    }
}

/// What a step says as it runs: from its code and facts, else aiball's sentence.
pub fn step_news(s: &Step) -> Option<String> {
    let d = &s.details;
    let said = match s.code.as_deref()? {
        "waiting_busy" => crate::t!("wizard-said-waiting-busy", names = list(&d["busy"])),
        "loops_idle" => crate::t!("wizard-said-loops-idle"),
        "still_busy" => crate::t!("wizard-said-still-busy", names = list(&d["busy"])),
        "stopping" => crate::t!("wizard-said-stopping", names = list(&d["loop"])),
        "waiting_stop" => crate::t!("wizard-said-waiting-stop", names = list(&d["running"])),
        "stopped" => crate::t!("wizard-said-stopped", names = list(&d["stopped"])),
        "still_running" => crate::t!("wizard-said-still-running", names = list(&d["running"])),
        "claude_in_folder" => crate::t!("wizard-said-claude-in-folder", pids = list(&d["pids"])),
        "folder_free" => crate::t!("wizard-said-folder-free"),
        "not_checked" => crate::t!("wizard-said-not-checked"),
        "claude_still_in_folder" => crate::t!("wizard-said-claude-still", pids = list(&d["pids"])),
        "target_ready" => crate::t!("wizard-said-target-ready", dir = path(&d["dir"])),
        "target_not_ready" => crate::t!("wizard-said-target-not-ready"),
        "written" => crate::t!("wizard-said-written", dir = path(&d["dir"])),
        "starting" => crate::t!("wizard-said-starting", names = list(&d["loop"])),
        "started" => crate::t!("wizard-said-started", names = list(&d["started"])),
        "not_started" => crate::t!("wizard-said-not-started", names = list(&d["failed"])),
        "cancelled" => crate::t!("wizard-said-cancelled"),
        _ => return s.said.clone(),
    };
    Some(said)
}

/// What happens when the run stops before its point of no return (the
/// preview's), or what the fallback did (the run's).
pub fn fallback_said(code: Option<&str>, facts: &Value, english: Option<&str>) -> Option<String> {
    Some(match code {
        Some("restart_on_current") => crate::t!("wizard-fallback-plan", current = path(&facts["current"])),
        Some("restarted_on_current") => crate::t!("wizard-fallback-restarted", dir = path(&facts["dir"]), names = list(&facts["restarted"])),
        Some("not_restarted") => crate::t!("wizard-fallback-not-restarted", names = list(&facts["failed"])),
        _ => english?.to_string(),
    })
}

/// A run's end, said.
pub fn status_said(status: &str) -> String {
    crate::t!(match status {
        "done" => "wizard-status-done",
        "failed" => "wizard-status-failed",
        "cancelled" => "wizard-status-cancelled",
        "interrupted" => "wizard-status-interrupted",
        _ => "wizard-status-running",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const PREVIEW: &str = r#"{
        "key": "claude.config_dir", "value": "/h/.claude-b", "title": "Switch the Claude config dir",
        "loops": [ { "name": "cl-wiz-1", "agent": "wiz-claude", "busy": true } ],
        "questions": [
            { "id": "confirm", "kind": "confirm", "facts": { "current": "/h/.claude", "target": "/h/.claude-b", "loops": ["cl-wiz-1"] }, "label": "Stop and restart 1 loop" },
            { "id": "fork", "kind": "choose", "default": "fork", "facts": { "current": "/h/.claude", "target": "/h/.claude-b", "target_first_run": false },
              "label": "start it?", "options": [ { "key": "fork", "label": "Link" }, { "key": "empty", "label": "Leave it empty" } ] },
            { "id": "zap", "kind": "choose", "label": "A question tvty does not know", "options": [ { "key": "a", "label": "Option A" } ] }
        ],
        "steps": [ { "id": "idle", "label": "Wait" }, { "id": "write", "label": "Write", "point_of_no_return": true } ],
        "fallback": "Before the write, the loops start again on /h/.claude",
        "fallback_code": "restart_on_current", "fallback_facts": { "current": "/h/.claude", "loops": ["cl-wiz-1"] }
    }"#;

    #[test]
    fn a_preview_reads_and_its_answers_wait_for_the_confirm() {
        let preview: Preview = serde_json::from_str(PREVIEW).unwrap();
        assert_eq!(preview.questions.len(), 3);
        assert!(preview.steps[1].point_of_no_return);
        let mut chosen = std::collections::HashMap::new();
        // Unconfirmed: nothing to send.
        assert!(answers(&preview.questions, &chosen, false).is_none());
        // `zap` has no default: it waits for its answer.
        assert!(answers(&preview.questions, &chosen, true).is_none());
        chosen.insert("zap".to_string(), "a".to_string());
        let sent = answers(&preview.questions, &chosen, true).unwrap();
        assert_eq!(sent["confirm"], json!(true));
        assert_eq!(sent["fork"], json!("fork"));
        assert_eq!(sent["zap"], json!("a"));
    }

    #[test]
    fn a_question_or_code_tvty_does_not_know_keeps_aiballs_sentence() {
        let preview: Preview = serde_json::from_str(PREVIEW).unwrap();
        assert_eq!(question_said(&preview.questions[2], &preview.loops), "A question tvty does not know");
        assert!(question_said(&preview.questions[0], &preview.loops).contains("wiz-claude"));
        assert_eq!(answer_said(&preview.questions[2], &preview.questions[2].options[0]), "Option A");
        let step = Step { id: "zap".into(), label: "Zap it".into(), code: Some("zapping".into()), said: Some("Zapping now".into()), ..Default::default() };
        assert_eq!(step_said(&step), "Zap it");
        assert_eq!(step_news(&step).as_deref(), Some("Zapping now"));
        assert_eq!(fallback_said(Some("zap"), &Value::Null, Some("English")).as_deref(), Some("English"));
    }

    #[test]
    fn a_known_code_is_said_from_its_facts() {
        let step = Step { id: "free".into(), code: Some("claude_in_folder".into()), details: json!({ "pids": [12, 34] }), ..Default::default() };
        let said = step_news(&step).unwrap();
        assert!(said.contains("12, 34"), "{said}");
        let step = Step { id: "idle".into(), code: Some("waiting_busy".into()), details: json!({ "busy": ["cl-a", { "name": "cl-b" }] }), ..Default::default() };
        assert!(step_news(&step).unwrap().contains("cl-a, cl-b"));
    }

    #[test]
    fn a_run_is_cancellable_until_its_point_of_no_return_begins() {
        let mut run = Run {
            run: "w-1".into(),
            status: "running".into(),
            steps: vec![
                Step { id: "idle".into(), status: Some("running".into()), ..Default::default() },
                Step { id: "write".into(), point_of_no_return: true, status: Some("pending".into()), ..Default::default() },
            ],
            ..Default::default()
        };
        assert!(run.cancellable());
        run.steps[1].status = Some("running".into());
        assert!(!run.cancellable());
        run.steps[1].status = Some("pending".into());
        run.status = "failed".into();
        assert!(!run.cancellable());
    }
}
