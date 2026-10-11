//! A project renamed from its options page ("Name"): its name in aiball,
//! its `.aiball.yaml` and its folder, at once, as `aiball project
//! rename-all` does (`project.rename_all`). tvty asks first what would be
//! done and what stands in the way — a loop running there, which it offers
//! to stop — then renames once nothing does. Every tvty follows the new
//! name as aiball says it.

use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::{Disableable as _, Sizable as _};
use gpui_kit::*;

use super::Shell;
use crate::activity::{self, Activity};
use crate::aiball::RenameAll;
use crate::theme::p;
use crate::ui::buttons::{self, Look as _};

/// The rename of the project the options are scoped to.
pub(super) struct Rename {
    project: String,
    name: Entity<InputState>,
    /// The folder stays where it is: only the name and the file change.
    keep_folder: bool,
    /// The folder to rename, chosen when aiball knows several (or none).
    folder: Option<String>,
    /// The folders aiball knows, once it said it cannot choose.
    roots: Vec<String>,
    /// What aiball answered, for the asking it answers.
    answer: Option<(Asked, Result<RenameAll, String>)>,
    busy: Option<Busy>,
}

/// What a dry run was asked for: an answer holds only for it.
#[derive(Clone, PartialEq)]
struct Asked {
    new_name: String,
    keep_folder: bool,
    folder: Option<String>,
}

#[derive(Clone, Copy, PartialEq)]
enum Busy {
    Checking,
    Renaming,
}

/// A name aiball takes: not empty, no white space, another than the one it has.
fn fits(new_name: &str, project: &str) -> bool {
    !new_name.is_empty() && !new_name.chars().any(char::is_whitespace) && new_name != project
}

impl Shell {
    /// The rename's state for `project`, fresh when the scope changes.
    pub(super) fn rename_for(&mut self, project: &str, window: &mut Window, cx: &mut Context<Self>) {
        if self.rename.as_ref().is_some_and(|r| r.project == project) {
            return;
        }
        let name = cx.new(|cx| InputState::new(window, cx).placeholder(crate::t!("rename-placeholder")));
        cx.subscribe_in(&name, window, |shell: &mut Self, _, event: &InputEvent, _, cx| {
            if matches!(event, InputEvent::PressEnter { .. }) {
                shell.rename_check(cx);
            }
            cx.notify();
        })
        .detach();
        self.rename = Some(Rename { project: project.to_string(), name, keep_folder: false, folder: None, roots: Vec::new(), answer: None, busy: None });
    }

    fn rename_asked(&self, cx: &App) -> Option<Asked> {
        let r = self.rename.as_ref()?;
        let new_name = r.name.read(cx).value().trim().to_string();
        fits(&new_name, &r.project).then(|| Asked { new_name, keep_folder: r.keep_folder, folder: r.folder.clone() })
    }

    /// The dry run: what would be done, what stands in the way.
    fn rename_check(&mut self, cx: &mut Context<Self>) {
        let Some(asked) = self.rename_asked(cx) else { return };
        let Some(r) = self.rename.as_mut().filter(|r| r.busy.is_none()) else { return };
        r.busy = Some(Busy::Checking);
        let (aiball, project) = (self.aiball.clone(), r.project.clone());
        cx.spawn(async move |this, cx| {
            let done = {
                let (asked, project) = (asked.clone(), project.clone());
                cx.background_executor()
                    .spawn(async move { aiball.rename_project(&project, &asked.new_name, asked.folder.as_deref(), asked.keep_folder, false) })
                    .await
            };
            let _ = this.update(cx, |shell, cx| {
                let Some(r) = shell.rename.as_mut().filter(|r| r.project == project) else { return };
                r.busy = None;
                let answer = match done {
                    Ok(answer) => Ok(answer),
                    Err(error) => match crate::aiball::roots_refused(&error) {
                        // Several folders, or none: one to choose, then asked again.
                        Some(roots) => {
                            r.roots = roots;
                            Err(crate::t!(if r.roots.is_empty() { "rename-no-root" } else { "rename-which-root" }))
                        }
                        None => Err(format!("{error:#}")),
                    },
                };
                r.answer = Some((asked, answer));
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }

    /// Renames for good, the dry run having found nothing in the way.
    fn rename_apply(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(asked) = self.rename_asked(cx) else { return };
        let Some(r) = self.rename.as_mut().filter(|r| r.busy.is_none()) else { return };
        r.busy = Some(Busy::Renaming);
        let (aiball, project) = (self.aiball.clone(), r.project.clone());
        cx.spawn_in(window, async move |this, cx| {
            let done = {
                let (asked, project) = (asked.clone(), project.clone());
                cx.background_executor()
                    .spawn(async move { aiball.rename_project(&project, &asked.new_name, asked.folder.as_deref(), asked.keep_folder, true) })
                    .await
            };
            let _ = this.update_in(cx, |shell, window, cx| {
                match done {
                    Ok(answer) if answer.applied => {
                        activity::publish(cx, Activity::done(None, crate::t!("rename-done", old = project.clone(), new = asked.new_name.clone())));
                        // The options follow the project under its new name.
                        shell.rename = None;
                        shell.project_opts = None;
                        shell.scope_options(Some(asked.new_name.clone()), window, cx);
                    }
                    // Something came in the way meanwhile: said, as a dry run says it.
                    Ok(answer) => {
                        if let Some(r) = shell.rename.as_mut() {
                            r.busy = None;
                            r.answer = Some((asked, Ok(answer)));
                        }
                    }
                    Err(error) => {
                        activity::publish(cx, Activity::failed(None, &crate::t!("rename-failed", project = project.clone()), format!("{error:#}")));
                        if let Some(r) = shell.rename.as_mut() {
                            r.busy = None;
                            r.answer = Some((asked, Err(format!("{error:#}"))));
                        }
                    }
                }
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }

    /// A loop in the way, stopped; then asked again.
    fn rename_stop_loop(&mut self, name: String, cx: &mut Context<Self>) {
        let Some(known) = self.board.known.iter().find(|l| l.name == name).cloned() else {
            activity::publish(cx, Activity::failed(None, &crate::t!("rename-stop-failed", name = name.clone()), crate::t!("rename-loop-unknown")));
            return;
        };
        let aiball = self.aiball.clone();
        cx.spawn(async move |this, cx| {
            let done = cx.background_executor().spawn(async move { crate::loops::stop(&aiball, &known) }).await;
            let _ = this.update(cx, |shell, cx| {
                if let Err(error) = done {
                    activity::publish(cx, Activity::failed(None, &crate::t!("rename-stop-failed", name = name.clone()), format!("{error:#}")));
                }
                shell.rename_check(cx);
            });
        })
        .detach();
    }

    /// The Project page's "Name": the new name, whether the folder moves,
    /// what aiball would do and what stands in the way, then Rename.
    pub(super) fn rename_section(&self, cx: &mut Context<Self>) -> Div {
        let mut out = div().flex().flex_col().gap_2().max_w(px(720.)).child(super::option_group("Name"));
        let Some(r) = self.rename.as_ref() else { return out };
        let asked = self.rename_asked(cx);
        let this = cx.entity().downgrade();
        out = out
            .child(div().text_sm().text_color(p().muted).child(crate::t!("rename-about", project = r.project.clone())))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(div().w(px(240.)).flex_none().child(Input::new(&r.name).small()))
                    .child(buttons::switch_with("rename-keep-folder", !r.keep_folder, div().text_sm().child(crate::t!("rename-move-folder")), move |on, _, cx| {
                        let _ = this.update(cx, |shell, cx| {
                            if let Some(r) = shell.rename.as_mut() {
                                r.keep_folder = !*on;
                            }
                            cx.notify();
                        });
                    }))
                    .child(
                        buttons::secondary("rename-check", crate::t!("rename-check"))
                            .small()
                            .disabled(asked.is_none() || r.busy.is_some())
                            .on_click(cx.listener(|shell, _, _, cx| shell.rename_check(cx))),
                    ),
            );
        // aiball knows several folders for it: the one to rename.
        if !r.roots.is_empty() {
            let mut chooser = div().flex().flex_wrap().gap_1();
            for (at, root) in r.roots.iter().enumerate() {
                let root = root.clone();
                chooser = chooser.child(
                    buttons::chip(SharedString::from(format!("rename-root-{at}")), super::loopstabs::home_short(&root))
                        .py_0p5()
                        .text_sm()
                        .chosen(r.folder.as_deref() == Some(root.as_str()))
                        .on_click(cx.listener(move |shell, _, _, cx| {
                            if let Some(r) = shell.rename.as_mut() {
                                r.folder = Some(root.clone());
                            }
                            shell.rename_check(cx);
                        })),
                );
            }
            out = out.child(div().text_sm().text_color(p().muted).child(crate::t!("rename-choose-root"))).child(chooser);
        }
        match r.busy {
            Some(Busy::Checking) => return out.child(div().text_sm().text_color(p().muted).child(crate::t!("rename-checking"))),
            Some(Busy::Renaming) => return out.child(div().text_sm().text_color(p().muted).child(crate::t!("rename-renaming"))),
            None => {}
        }
        // An answer holds for what it was asked: typed again, asked again.
        let Some((for_what, answer)) = r.answer.as_ref().filter(|(for_what, _)| Some(for_what) == asked.as_ref()) else { return out };
        let answer = match answer {
            Err(error) => return out.child(div().text_sm().text_color(p().danger).child(error.clone())),
            Ok(answer) => answer,
        };
        out = out.child(div().text_sm().child(match &answer.plan.folder {
            Some(folder) => crate::t!("rename-folder-moves", from = super::loopstabs::home_short(&folder.from), to = super::loopstabs::home_short(&folder.to)),
            None => crate::t!("rename-folder-stays"),
        }));
        for (at, blocker) in answer.blockers.iter().enumerate() {
            let stop = blocker.running.clone().map(|name| {
                buttons::chip(SharedString::from(format!("rename-stop-{at}")), crate::t!("rename-stop"))
                    .py_0p5()
                    .text_sm()
                    .on_click(cx.listener(move |shell, _, _, cx| shell.rename_stop_loop(name.clone(), cx)))
            });
            out = out.child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .text_sm()
                    .child(div().flex_1().min_w_0().text_color(p().danger).child(format!("✗ {}", blocker.text)))
                    .children(stop),
            );
        }
        let clear = answer.blockers.is_empty();
        out.child(
            div().flex().items_center().gap_2().child(
                buttons::primary("rename-apply", crate::t!("rename-apply", new = for_what.new_name.clone()))
                    .small()
                    .disabled(!clear)
                    .on_click(cx.listener(|shell, _, window, cx| shell.rename_apply(window, cx))),
            ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::fits;

    #[test]
    fn a_new_name_is_one_word_and_another() {
        assert!(fits("helios", "helio"));
        assert!(!fits("", "helio"));
        assert!(!fits("helio", "helio"));
        assert!(!fits("my project", "helio"));
    }
}
