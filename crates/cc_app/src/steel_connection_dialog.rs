//! Configuration dialog for creating a parametric steel connection on picked
//! structural members (`SteelConnectionCreate`).

use crate::dialog::Dialog;
use crate::state::AppState;
use crate::t;
use crate::theme::{self, PALETTE};
use cc_core::steel_connection::{ConnectionKind, ConnectionParams, recognize_for_members};
use cc_core::{Category, ElementId};
use cc_engine::commands::steel_connection::SteelConnectionCreate;
use egui::{Context, RichText};

/// State of the open Steel Connection configuration dialog.
pub struct SteelConnectionDialog {
    pub members: Vec<ElementId>,
    pub kind: ConnectionKind,
    pub plate_thickness_mm: f64,
    pub bolt_diameter_mm: f64,
    pub bolt_grade: String,
    pub rows: u32,
    pub cols: u32,
    pub extension_mm: f64,
    pub stiffeners: bool,
    pub base_mode: String,
    pub anchors: u32,
    pub error: Option<String>,
}

fn kind_label(kind: ConnectionKind) -> &'static str {
    match kind {
        ConnectionKind::BasePlate => "Chapa de base (Base plate)",
        ConnectionKind::EndPlate => "Chapa de topo / Rígida (End plate)",
        ConnectionKind::SinglePlate => "Chapa simples / Cisalhamento (Fin plate)",
        ConnectionKind::DoubleAngle => "Dupla cantoneira (Double angle)",
        ConnectionKind::GussetPlate => "Chapa de nó / Contraventamento (Gusset)",
        ConnectionKind::Splice => "Emenda parafusada (Splice)",
    }
}

fn defaults_for(state: &AppState, members: &[ElementId], forced: Option<ConnectionKind>) -> SteelConnectionDialog {
    let project = state.project();
    let recognized = recognize_for_members(project, members, forced, &ConnectionParams::default());
    if let Some(rc) = recognized {
        SteelConnectionDialog {
            members: members.to_vec(),
            kind: rc.kind,
            plate_thickness_mm: (rc.spec.plate_thickness * 1000.0).max(4.0),
            bolt_diameter_mm: (rc.spec.bolt_diameter * 1000.0).max(10.0),
            bolt_grade: rc.spec.bolt_grade,
            rows: rc.spec.rows.max(1),
            cols: rc.spec.cols.max(1),
            extension_mm: if rc.kind == ConnectionKind::EndPlate { 40.0 } else { 0.0 },
            stiffeners: rc.spec.stiffeners,
            base_mode: "pinned".to_string(),
            anchors: 4,
            error: None,
        }
    } else {
        let single_column =
            members.len() == 1 && project.element(members[0]).is_some_and(|e| e.category() == Category::Column);
        let kind =
            forced.unwrap_or(if single_column { ConnectionKind::BasePlate } else { ConnectionKind::SinglePlate });
        SteelConnectionDialog {
            members: members.to_vec(),
            kind,
            plate_thickness_mm: 12.0,
            bolt_diameter_mm: 16.0,
            bolt_grade: "8.8".to_string(),
            rows: 3,
            cols: 2,
            extension_mm: 0.0,
            stiffeners: false,
            base_mode: "pinned".to_string(),
            anchors: 4,
            error: None,
        }
    }
}

/// Open the steel connection configuration dialog for `members`.
pub fn open(state: &mut AppState, members: Vec<ElementId>) {
    if members.is_empty() {
        return;
    }
    state.dialogs.steel_connection = Some(defaults_for(state, &members, None));
}

fn member_summary(state: &AppState, members: &[ElementId]) -> String {
    let p = state.project();
    let names: Vec<String> = members
        .iter()
        .filter_map(|id| {
            let e = p.element(*id)?;
            let type_name = p.element_type(e.type_id).map(|t| t.name.as_str()).unwrap_or("");
            let cat = match e.category() {
                Category::Column => "Pilar",
                Category::Beam => "Viga",
                _ => "Barra",
            };
            if type_name.is_empty() {
                Some(format!("{cat} #{}", id.0))
            } else {
                Some(format!("{cat} {type_name} (#{})", id.0))
            }
        })
        .collect();
    names.join("  +  ")
}

pub fn show(ctx: &Context, state: &mut AppState) {
    let Some(mut d) = state.dialogs.steel_connection.take() else { return };
    let members_text = member_summary(state, &d.members);
    let mut create_btn = 0;
    let response = Dialog::new("steel-connection", t!("action-steel-connection")).width(480.0).show(
        ctx,
        |ui| {
            ui.label(theme::muted("Elementos selecionados:"));
            ui.label(RichText::new(&members_text).strong());
            ui.add_space(8.0);

            egui::Grid::new("steel-connection-grid").num_columns(2).spacing([14.0, 8.0]).min_col_width(150.0).show(
                ui,
                |ui| {
                    ui.label(theme::muted("Tipo de ligação"));
                    let mut chosen_kind = d.kind;
                    egui::ComboBox::from_id_salt("steel-conn-kind")
                        .selected_text(kind_label(d.kind))
                        .width(270.0)
                        .show_ui(ui, |ui| {
                            for k in ConnectionKind::ALL {
                                ui.selectable_value(&mut chosen_kind, k, kind_label(k));
                            }
                        });
                    if chosen_kind != d.kind {
                        let refreshed = defaults_for(state, &d.members, Some(chosen_kind));
                        d = refreshed;
                    }
                    ui.end_row();

                    ui.label(theme::muted("Espessura da chapa"));
                    ui.add(
                        egui::DragValue::new(&mut d.plate_thickness_mm)
                            .speed(0.5)
                            .range(3.0..=80.0)
                            .max_decimals(1)
                            .suffix(" mm"),
                    );
                    ui.end_row();

                    ui.label(theme::muted("Diâmetro do parafuso"));
                    egui::ComboBox::from_id_salt("steel-conn-bolt-d")
                        .selected_text(format!("M{:.0} ({:.0} mm)", d.bolt_diameter_mm, d.bolt_diameter_mm))
                        .width(160.0)
                        .show_ui(ui, |ui| {
                            for dia in [12.0, 16.0, 20.0, 22.0, 24.0, 27.0, 30.0] {
                                ui.selectable_value(&mut d.bolt_diameter_mm, dia, format!("M{dia:.0} ({dia:.0} mm)"));
                            }
                        });
                    ui.end_row();

                    ui.label(theme::muted("Classe do parafuso"));
                    egui::ComboBox::from_id_salt("steel-conn-grade").selected_text(&d.bolt_grade).width(120.0).show_ui(
                        ui,
                        |ui| {
                            for g in ["8.8", "10.9", "A325", "A490"] {
                                ui.selectable_value(&mut d.bolt_grade, g.to_string(), g);
                            }
                        },
                    );
                    ui.end_row();

                    if d.kind == ConnectionKind::BasePlate {
                        ui.label(theme::muted("Engastamento da base"));
                        egui::ComboBox::from_id_salt("steel-conn-base-mode")
                            .selected_text(if d.base_mode == "fixed" {
                                "Engastada (Fixed)"
                            } else {
                                "Rotulada (Pinned)"
                            })
                            .width(180.0)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut d.base_mode, "pinned".to_string(), "Rotulada (Pinned)");
                                ui.selectable_value(&mut d.base_mode, "fixed".to_string(), "Engastada (Fixed)");
                            });
                        ui.end_row();

                        ui.label(theme::muted("Chumbadores"));
                        egui::ComboBox::from_id_salt("steel-conn-anchors")
                            .selected_text(format!("{} chumbadores", d.anchors))
                            .width(140.0)
                            .show_ui(ui, |ui| {
                                for n in [2u32, 4, 6, 8] {
                                    ui.selectable_value(&mut d.anchors, n, format!("{n} chumbadores"));
                                }
                            });
                        ui.end_row();
                    } else {
                        ui.label(theme::muted("Linhas de parafusos"));
                        ui.add(egui::DragValue::new(&mut d.rows).speed(0.1).range(1..=12));
                        ui.end_row();

                        ui.label(theme::muted("Colunas de parafusos"));
                        ui.add(egui::DragValue::new(&mut d.cols).speed(0.1).range(1..=6));
                        ui.end_row();
                    }

                    if d.kind == ConnectionKind::EndPlate {
                        ui.label(theme::muted("Extensão superior"));
                        ui.add(
                            egui::DragValue::new(&mut d.extension_mm)
                                .speed(1.0)
                                .range(0.0..=250.0)
                                .max_decimals(0)
                                .suffix(" mm"),
                        );
                        ui.end_row();
                    }

                    ui.label(theme::muted("Enrijecedores"));
                    ui.checkbox(&mut d.stiffeners, "Incluir enrijecedores");
                    ui.end_row();
                },
            );

            if let Some(err) = &d.error {
                ui.add_space(6.0);
                ui.label(RichText::new(err).color(PALETTE.danger));
            }
        },
        |f| {
            create_btn = f.primary("Criar ligação");
            f.secondary(t!("action-cancel"));
        },
    );

    let close = match response.clicked {
        Some(i) if i == create_btn => {
            let cmd = SteelConnectionCreate {
                elements: d.members.iter().map(|id| id.0).collect(),
                kind: Some(d.kind.code().to_string()),
                plate_thickness: Some(d.plate_thickness_mm / 1000.0),
                bolt_diameter: Some(d.bolt_diameter_mm / 1000.0),
                bolt_grade: Some(d.bolt_grade.clone()),
                rows: (d.kind != ConnectionKind::BasePlate).then_some(d.rows),
                cols: (d.kind != ConnectionKind::BasePlate).then_some(d.cols),
                extension: (d.kind == ConnectionKind::EndPlate).then_some(d.extension_mm / 1000.0),
                stiffeners: Some(d.stiffeners),
                base_mode: (d.kind == ConnectionKind::BasePlate).then_some(d.base_mode.clone()),
                anchors: (d.kind == ConnectionKind::BasePlate).then_some(d.anchors),
                ..Default::default()
            };
            match state.session.run(cmd) {
                Ok(out) => {
                    if let Some(id) = out["id"].as_u64() {
                        state.selection.clear();
                        state.selection.insert(ElementId(id));
                    }
                    state.message = format!("Ligação {} criada.", kind_label(d.kind));
                    true
                }
                Err(e) => {
                    d.error = Some(e.message);
                    false
                }
            }
        }
        Some(_) => true,
        None => response.dismissed,
    };
    if !close {
        state.dialogs.steel_connection = Some(d);
    }
}
