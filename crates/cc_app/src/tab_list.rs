//! The open views list at the right end of each view tab bar (#371). When
//! many views are open their tabs overflow the bar; the button lists the
//! bar's views, grouped by view type as the project browser groups them, to
//! bring one to the front, and closes the others or all of them.

use crate::browser::category_icon;
use crate::i18n::tr;
use crate::icons::{self, Icon};
use crate::state::AppState;
use crate::t;
use crate::theme::{self, PALETTE};
use cc_core::{ViewCategory, ViewId};
use egui::{Rect, Response, Sense, Ui, UiBuilder, pos2, vec2};
use egui_dock::{DockState, NodePath, SurfaceIndex};
use std::collections::HashMap;

/// Width kept free at the right end of each tab bar for the button.
pub const BUTTON_WIDTH: i8 = 28;
const ROW_HEIGHT: f32 = 22.0;
/// Room left of a row's label: the check mark and the view type's icon.
const ROW_LEAD: f32 = 44.0;

/// What the tab bars need between frames.
#[derive(Debug, Default)]
pub struct TabBar {
    /// Where each tab was drawn in the last frame.
    pub tab_rects: HashMap<ViewId, Rect>,
    /// A tab brought to the front, to scroll into view once drawn.
    pub reveal: Option<ViewId>,
    /// Tabs to close on the next frame.
    pub close: Vec<ViewId>,
}

/// A choice made in a bar's list.
enum Choice {
    Activate(ViewId),
    CloseOthers,
    CloseAll,
}

/// The button of every tab bar of `dock`, drawn after the dock, which kept
/// `style`'s right margin of each bar free for it; and the tab brought to
/// the front scrolled into view.
pub fn show(ui: &mut Ui, dock: &mut DockState<ViewId>, state: &mut AppState, style: &egui_dock::Style) {
    let reveal = state.tab_bar.reveal.take();
    let bar_height = style.tab_bar.height;
    let margin = style.tab_bar.inner_margin;
    let mut chosen = None;
    for (path, leaf) in dock.iter_leaves_mut() {
        let rect = leaf.rect;
        if leaf.tabs.is_empty() || rect.width() <= 2.0 * f32::from(BUTTON_WIDTH) || rect.height() < bar_height {
            continue;
        }
        let bar = Rect::from_min_size(rect.min, vec2(rect.width(), bar_height));
        if let Some(view) = reveal
            && leaf.tabs.contains(&view)
            && let Some(tab) = state.tab_bar.tab_rects.get(&view)
        {
            // The dock draws the tabs `scroll` points to the left (it is never positive).
            let left = bar.left() + margin.leftf();
            let right = bar.right() - margin.rightf();
            if tab.right() > right {
                leaf.scroll -= tab.right() - right;
            } else if tab.left() < left {
                leaf.scroll += left - tab.left();
            }
            ui.ctx().request_repaint();
        }
        let active = leaf.tabs.get(leaf.active.0).copied();
        let button = Rect::from_min_max(pos2(bar.right() - f32::from(BUTTON_WIDTH), bar.top()), bar.right_bottom());
        let line = style.tab_bar.hline_color;
        if let Some(choice) = bar_button(ui, path, button, line, &leaf.tabs, active, state) {
            chosen = Some((choice, leaf.tabs.clone(), active));
        }
    }
    let Some((choice, tabs, active)) = chosen else { return };
    match choice {
        Choice::Activate(view) => state.open_view(view),
        Choice::CloseOthers => state.tab_bar.close.extend(tabs.into_iter().filter(|t| Some(*t) != active)),
        // Closing every open view closes the project: the active tab stays
        // until then, as when the last tab is closed (`Tabs::on_close`).
        Choice::CloseAll if tabs.len() >= state.open_tabs => {
            state.tab_bar.close.extend(tabs.into_iter().filter(|t| Some(*t) != active));
            state.close_with_last_view = true;
        }
        Choice::CloseAll => state.tab_bar.close.extend(tabs),
    }
    ui.ctx().request_repaint();
}

/// The layer the tabs of `surface` are drawn in: a floating surface is an
/// egui window that egui_dock names after its index.
fn layer(ui: &Ui, surface: SurfaceIndex) -> egui::LayerId {
    if surface.is_main() {
        return ui.layer_id();
    }
    egui::LayerId::new(egui::Order::Middle, egui::Id::new(format!("window {surface:?}")))
}

/// The button at `rect` and its menu of `tabs`.
fn bar_button(
    ui: &mut Ui,
    path: NodePath,
    rect: Rect,
    line: egui::Color32,
    tabs: &[ViewId],
    active: Option<ViewId>,
    state: &AppState,
) -> Option<Choice> {
    let mut ui = ui.new_child(UiBuilder::new().layer_id(layer(ui, path.surface)).max_rect(rect));
    ui.set_clip_rect(rect);
    let id = egui::Id::new(("open-views", path));
    let response = ui.interact(rect, id, Sense::click());
    let label = t!("tabs-open-views");
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &label));
    let painter = ui.painter();
    let px = ui.ctx().pixels_per_point().recip();
    painter.hline(rect.x_range(), rect.bottom() - px, egui::Stroke::new(px, line));
    let open = egui::Popup::is_id_open(ui.ctx(), egui::Popup::default_response_id(&response));
    if response.hovered() || open {
        painter.rect_filled(rect.shrink2(vec2(2.0, 4.0)), theme::RADIUS as f32, PALETTE.hover);
    }
    let color = if response.hovered() || open { PALETTE.text } else { PALETTE.text_muted };
    icons::paint(painter, Rect::from_center_size(rect.center(), vec2(14.0, 14.0)), Icon::ChevronDown, color);
    let response = response.on_hover_text(label);
    let mut chosen = None;
    egui::Popup::menu(&response).align(egui::RectAlign::BOTTOM_END).show(|ui| {
        chosen = menu(ui, tabs, active, state);
    });
    chosen
}

/// The bar's views by view type, in the browser's order and with its group
/// names, then Close Other Views and Close All Views.
fn menu(ui: &mut Ui, tabs: &[ViewId], active: Option<ViewId>, state: &AppState) -> Option<Choice> {
    let project = state.project();
    let rows: Vec<(ViewCategory, ViewId, String)> = tabs
        .iter()
        .filter_map(|id| project.view(*id).map(|v| (v.category(), *id, cc_core::browser::label(v))))
        .collect();
    let font = egui::TextStyle::Button.resolve(ui.style());
    let text_width = |text: String| ui.painter().layout_no_wrap(text, font.clone(), PALETTE.text).size().x;
    let close_others = t!("action-close-other-views");
    let close_all = t!("action-close-all-views");
    let padding = 2.0 * ui.spacing().button_padding.x;
    let width = rows
        .iter()
        .map(|(_, _, label)| text_width(label.clone()) + ROW_LEAD + 12.0)
        .chain([close_others.clone(), close_all.clone()].map(|text| text_width(text) + padding))
        .fold(160.0_f32, f32::max);
    ui.set_min_width(width);
    let mut chosen = None;
    for category in ViewCategory::ALL {
        let mut group = rows.iter().filter(|(c, _, _)| *c == category).peekable();
        if group.peek().is_none() {
            continue;
        }
        ui.label(egui::RichText::new(tr(category.key())).small().color(PALETTE.text_faint));
        for (_, id, label) in group {
            if row(ui, width, category_icon(category), label, Some(*id) == active).clicked() {
                chosen = Some(Choice::Activate(*id));
                ui.close();
            }
        }
    }
    ui.separator();
    if ui.add_enabled(tabs.len() > 1, egui::Button::new(close_others)).clicked() {
        chosen = Some(Choice::CloseOthers);
        ui.close();
    }
    if ui.button(close_all).clicked() {
        chosen = Some(Choice::CloseAll);
        ui.close();
    }
    chosen
}

/// A view's row: a check mark when it is the bar's active tab, its type's icon and its name.
fn row(ui: &mut Ui, width: f32, icon: Icon, label: &str, active: bool) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(width, ROW_HEIGHT), Sense::click());
    response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Button, true, active, label));
    if !ui.is_rect_visible(rect) {
        return response;
    }
    let painter = ui.painter();
    if response.hovered() {
        painter.rect_filled(rect, theme::RADIUS as f32, PALETTE.hover);
    }
    let cy = rect.center().y;
    let color = if active { PALETTE.accent_text } else { PALETTE.text };
    if active {
        let x = rect.left() + 6.0;
        let check = vec![pos2(x, cy), pos2(x + 3.5, cy + 3.5), pos2(x + 10.0, cy - 4.0)];
        painter.add(egui::Shape::line(check, egui::Stroke::new(1.5, color)));
    }
    let icon_rect = Rect::from_center_size(pos2(rect.left() + 29.0, cy), vec2(14.0, 14.0));
    icons::paint(painter, icon_rect, icon, if active { PALETTE.accent_text } else { PALETTE.text_muted });
    let font = egui::TextStyle::Button.resolve(ui.style());
    painter.text(pos2(rect.left() + ROW_LEAD, cy), egui::Align2::LEFT_CENTER, label, font, color);
    response
}
