//! A plugin command waiting for an external program (`ccad.process.run`).
//!
//! The app's session does not block ([`cc_engine::Session::set_blocking`]):
//! a plugin command that runs a program is left pending, and [`poll`] goes
//! on with it every frame until it ends. Meanwhile a modal dialog shows the
//! plugin, the program, the time elapsed and the last line it wrote, with
//! Cancel, which stops the program and undoes what the command did; the
//! engine runs only queries, and [`crate::jobs::busy`] keeps shortcuts and
//! dropped files out. When the command ends, its form shows the result (see
//! [`crate::plugin_form`]), or the status bar when no form waits for it.

use crate::dialog::Dialog;
use crate::state::AppState;
use crate::t;
use crate::theme::{self, PALETTE};
use egui::{Context, RichText};
use std::time::Duration;

/// Every frame: go on with the plugin command waiting, and finish it when it
/// ended.
pub fn poll(ctx: &Context, state: &mut AppState) {
    if !state.session.has_job() {
        return;
    }
    match state.session.poll_job() {
        Some(result) => finish(state, result),
        // The program's end, its output and the time move on by themselves.
        None => ctx.request_repaint_after(Duration::from_millis(100)),
    }
}

/// Stop the program and undo what the command did.
pub fn cancel(state: &mut AppState) {
    if let Some(result) = state.session.cancel_job() {
        finish(state, result);
    }
}

/// The command ended with `result`: to its form when one waits for it, else
/// to the status bar.
fn finish(state: &mut AppState, result: cc_engine::Result<serde_json::Value>) {
    let printed = state.session.take_extension_messages();
    let waiting = state.dialogs.plugin_form.as_ref().is_some_and(crate::plugin_form::PluginForm::is_waiting);
    if waiting {
        crate::plugin_form::job_ended(state, result, printed);
        return;
    }
    for line in &printed {
        log::info!("{line}");
    }
    state.message = match result {
        Ok(_) => t!("job-plugin-done"),
        Err(e) if e.code == cc_engine::ErrorCode::Cancelled => t!("job-plugin-cancelled"),
        Err(e) => e.message,
    };
}

/// The progress dialog of the plugin command waiting.
pub fn show(ctx: &Context, state: &mut AppState) {
    let Some(job) = state.session.job() else { return };
    let title = if job.info.plugin_name.is_empty() { job.command.clone() } else { job.info.plugin_name.clone() };
    let tool = if job.info.tool.is_empty() { job.command.clone() } else { job.info.tool.clone() };
    let mut cancel_button = 0;
    let response = Dialog::new("plugin-job", title).width(420.0).show(
        ctx,
        |ui| {
            ui.horizontal(|ui| {
                ui.add(egui::Spinner::new().size(32.0).color(PALETTE.accent));
                ui.add_space(10.0);
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 4.0;
                    ui.label(theme::title(t!("job-plugin-running", "tool" => tool)));
                    let elapsed = crate::jobs::elapsed_text(job.elapsed);
                    ui.label(RichText::new(t!("job-elapsed", "time" => elapsed)).size(12.0).color(PALETTE.text_faint));
                    if !job.info.last_line.is_empty() {
                        let line = RichText::new(&job.info.last_line).monospace().size(11.5).color(PALETTE.text_muted);
                        ui.add(egui::Label::new(line).truncate());
                    }
                });
            });
            ui.add_space(10.0);
            ui.label(RichText::new(t!("job-plugin-hint")).size(11.5).color(PALETTE.text_muted));
        },
        |footer| cancel_button = footer.danger(t!("action-cancel")),
    );
    // The elapsed time moves on even when nothing else does.
    ctx.request_repaint_after(Duration::from_millis(250));
    if response.dismissed || response.clicked == Some(cancel_button) {
        cancel(state);
    }
}
