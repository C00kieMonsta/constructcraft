//! Plugins in the app: loading the plugins of the plugins folder into the
//! session (their commands join the engine's catalog, the palette and the
//! ribbon's *Plugin Commands* menu, and their buttons make the ribbon's
//! Plugins tab, see [`unavailable`] for their `when`) and the Plugins window,
//! which turns them on and off, grants the `files` permission, edits their
//! settings, removes them, and installs and updates them from the community
//! index.
//!
//! Plugins run sandboxed in `cc_plugins`; installing is `cc_plugins::store` (checked
//! archives), and the only network use is here: the index and the archives
//! it lists, downloaded over HTTPS in background threads with bounded sizes,
//! only when the user opens the *Available* tab or installs.

use crate::dialog::Dialog;
use crate::plugin_form::Fmt;
use crate::state::AppState;
use crate::t;
use crate::table;
use crate::theme::{self, PALETTE};
use cc_core::ViewCategory;
use cc_engine::Extension;
use cc_plugins::contributions::current_values;
use cc_plugins::store::{self, Index, IndexPlugin, IndexVersion};
use cc_plugins::{Limits, Permissions, PluginHost, Setting, SettingKind, When};
use egui::{Context, RichText, Stroke, Ui};
use serde_json::{Map, Value, json};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::Duration;

/// The whole request gives up after this.
const TIMEOUT: Duration = Duration::from_secs(60);

/// Which list the window shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    Installed,
    Available,
}

/// The index as the window knows it.
#[derive(Debug, Default)]
enum IndexState {
    #[default]
    NotRead,
    Reading(Receiver<Result<Index, String>>),
    Read(Index),
    Failed(String),
}

/// A plugin of the plugins folder, as listed.
#[derive(Debug, Clone)]
pub struct Listed {
    pub dir: PathBuf,
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub authors: Vec<String>,
    pub license: String,
    pub homepage: String,
    pub asks_files: bool,
    /// Its commands, when loaded: `(full name, description)`.
    pub commands: Vec<(String, String)>,
    /// It declares settings (shown in the window, also when turned off).
    pub has_settings: bool,
    /// Why it could not be read or loaded.
    pub error: Option<String>,
}

/// A loaded plugin's ribbon button, with what its tooltip tells.
#[derive(Debug, Clone)]
pub struct RibbonButton {
    pub plugin_id: String,
    /// The plugin's name, in the current language.
    pub plugin: String,
    /// What the command does, in the current language.
    pub description: String,
    pub button: cc_plugins::Button,
}

/// The settings of the plugin picked in the window, being edited.
#[derive(Debug)]
struct SettingsEdit {
    id: String,
    settings: Vec<Setting>,
    /// One per setting, in the same order (the command form's fields).
    fields: Vec<crate::plugin_form::Field>,
}

/// An install in a background thread.
#[derive(Debug)]
struct Job {
    name: String,
    id: String,
    files: bool,
    result: Receiver<Result<PathBuf, String>>,
}

/// Plugins state of the app (kept when the document changes).
#[derive(Debug, Default)]
pub struct Plugins {
    /// The Plugins window is open.
    pub open: bool,
    pub tab: Tab,
    /// The plugins of the folder, read at the last load.
    pub listed: Vec<Listed>,
    /// The plugin picked in the window (by id).
    pub selected: Option<String>,
    pub search: String,
    index: IndexState,
    /// An install running: the plugin's name and id, whether it gets the
    /// `files` permission, and its result.
    job: Option<Job>,
    /// Installs that may work with files (the user ticked the box).
    allow_files: std::collections::BTreeSet<String>,
    /// The last install or removal, for the window.
    pub notice: Option<Result<String, String>>,
    /// The folder the plugins were loaded from (loaded again when the
    /// preference changes).
    loaded_from: Option<PathBuf>,
    /// The loaded plugins' ribbon buttons, in load order then manifest order.
    pub buttons: Vec<RibbonButton>,
    /// The titles of the loaded plugins' commands, by full name.
    titles: HashMap<String, String>,
    /// The settings being edited in the window.
    settings: Option<SettingsEdit>,
}

impl Plugins {
    /// The title of the plugin command `full_name`, in the language the
    /// plugins were loaded in (they load again when it changes).
    pub fn command_title(&self, full_name: &str) -> Option<&str> {
        self.titles.get(full_name).map(String::as_str)
    }

    /// The plugin commands, by plugin: `(plugin name, [(command, description)])`.
    pub fn commands(&self) -> Vec<(&str, &[(String, String)])> {
        self.listed
            .iter()
            .filter(|p| !p.commands.is_empty())
            .map(|p| (p.name.as_str(), p.commands.as_slice()))
            .collect()
    }
}

/// Load the plugins of the plugins folder into the session, as the
/// preferences say (turned off ones are left out and run no code; `files`
/// only where the user allowed it), and list them for the window.
pub fn load(state: &mut AppState) {
    let mut host = PluginHost::new();
    let mut listed = Vec::new();
    if let Some(folder) = state.prefs.plugins_dir() {
        let off = state.prefs.plugins_off.clone();
        let files = state.prefs.plugins_files.clone();
        let errors = host.load_folder(&folder, Limits::default(), |m| {
            (!off.contains(&m.id)).then(|| Permissions { files: files.contains(&m.id) })
        });
        for found in store::installed(&folder) {
            let error = errors.iter().find(|e| e.dir == found.dir).map(|e| e.message.clone());
            let item = match found.manifest {
                Ok(m) => {
                    let loaded = host.plugins().iter().find(|p| p.manifest.id == m.id && p.dir == found.dir);
                    let commands = loaded
                        .map(|p| p.commands().map(|c| (c.name.clone(), c.description.clone())).collect())
                        .unwrap_or_default();
                    // A plugin turned off is not loaded: its own text is
                    // looked up in its locales here.
                    let (name, description) = match loaded {
                        Some(p) => (p.display_name(), p.display_description()),
                        None if m.api >= 2 => match cc_plugins::Locales::load(&found.dir) {
                            Ok(l) => (l.resolve(&m.name), l.resolve(&m.description)),
                            Err(_) => (m.name.clone(), m.description.clone()),
                        },
                        None => (m.name.clone(), m.description.clone()),
                    };
                    Listed {
                        asks_files: m.requested().files,
                        has_settings: !m.settings.is_empty(),
                        id: m.id,
                        name,
                        version: m.version,
                        description,
                        authors: m.authors,
                        license: m.license,
                        homepage: m.homepage,
                        commands,
                        error,
                        dir: found.dir,
                    }
                }
                Err(message) => Listed {
                    id: found.dir.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(),
                    name: found.dir.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(),
                    version: String::new(),
                    description: String::new(),
                    authors: Vec::new(),
                    license: String::new(),
                    homepage: String::new(),
                    asks_files: false,
                    has_settings: false,
                    commands: Vec::new(),
                    error: Some(message),
                    dir: found.dir,
                },
            };
            listed.push(item);
        }
    }
    for line in host.take_messages() {
        log::info!("{line}");
    }
    // What the ribbon and the forms show, read before the host goes into
    // the session (which keeps it as an engine extension).
    let mut buttons = Vec::new();
    let mut titles = HashMap::new();
    for plugin in host.plugins() {
        let descriptions: HashMap<String, String> = plugin.commands().map(|c| (c.name, c.description)).collect();
        for name in descriptions.keys() {
            if let Some(title) = host.command_title(name) {
                titles.insert(name.clone(), title);
            }
        }
        let name = plugin.display_name();
        for button in plugin.buttons() {
            buttons.push(RibbonButton {
                plugin_id: plugin.manifest.id.clone(),
                plugin: name.clone(),
                description: descriptions.get(&button.command).cloned().unwrap_or_default(),
                button,
            });
        }
    }
    state.session.set_extension(Some(Box::new(host)));
    state.plugins.listed = listed;
    state.plugins.buttons = buttons;
    state.plugins.titles = titles;
    state.plugins.settings = None;
    state.plugins.loaded_from = state.prefs.plugins_dir();
}

/// Why a plugin button whose `when` is `when` is unavailable now (`None`:
/// it is available); the first term that does not hold tells.
pub fn unavailable(state: &AppState, when: &[When]) -> Option<String> {
    let project = state.project();
    let categories = || state.selection.iter().filter_map(|id| project.element(*id)).map(|e| e.category());
    for term in when {
        let holds = match term {
            When::Selection => !state.selection.is_empty(),
            When::SelectionOne => state.selection.len() == 1,
            When::SelectionOf(name) => {
                !state.selection.is_empty()
                    && state.selection.len() == categories().count()
                    && categories().all(|c| cc_engine::category_name(c) == *name)
            }
            When::View(kind) => {
                state.active_view.and_then(|v| project.view(v)).is_some_and(|v| view_kind(v.category()) == kind)
            }
        };
        if holds {
            continue;
        }
        return Some(match term {
            When::Selection => t!("plug-when-selection"),
            When::SelectionOne => t!("plug-when-one"),
            When::SelectionOf(name) => {
                let category = crate::plugin_form::parse_category(name)
                    .map(crate::actions::t_category)
                    .unwrap_or_else(|| name.clone());
                t!("plug-when-category", "category" => category)
            }
            When::View(kind) => {
                let label = ViewCategory::ALL
                    .into_iter()
                    .find(|c| view_kind(*c) == kind)
                    .map(|c| crate::i18n::tr(c.key()))
                    .unwrap_or_else(|| kind.clone());
                t!("plug-when-view", "kind" => label)
            }
        });
    }
    None
}

/// The name a `view:<kind>` term gives a view category.
fn view_kind(category: ViewCategory) -> &'static str {
    match category {
        ViewCategory::FloorPlan => "floor_plan",
        ViewCategory::CeilingPlan => "ceiling_plan",
        ViewCategory::ThreeD => "three_d",
        ViewCategory::Elevation => "elevation",
        ViewCategory::Section => "section",
        ViewCategory::Legend => "legend",
        ViewCategory::Drafting => "drafting",
        ViewCategory::Schedule => "schedule",
        ViewCategory::Diagram => "diagram",
        ViewCategory::Sheet => "sheet",
    }
}

/// Every frame: load the plugins again when the plugins folder changed, and
/// show what plugins printed while their commands ran (logged, the last one
/// in the status bar).
pub fn drain_messages(state: &mut AppState) {
    if state.plugins.loaded_from != state.prefs.plugins_dir() {
        load(state);
    }
    for line in state.session.take_extension_messages() {
        log::info!("{line}");
        state.message = line;
    }
}

/// Open the Plugins window.
pub fn open(state: &mut AppState) {
    state.plugins.open = true;
    state.plugins.notice = None;
}

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        // No request, and no redirect, leaves https.
        .https_only(true)
        .timeout_global(Some(TIMEOUT))
        .user_agent(format!("ConstructCraft/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .into()
}

/// GET an https `url`, at most `limit` bytes.
pub(crate) fn get(url: &str, limit: usize) -> Result<Vec<u8>, String> {
    if !url.starts_with("https://") {
        return Err(format!("not an https link: {url}"));
    }
    // Tests never reach the network.
    if cfg!(test) || crate::update::disabled_by_env() {
        return Err("offline".into());
    }
    let mut response = agent().get(url).call().map_err(|e| e.to_string())?;
    response.body_mut().with_config().limit(limit as u64).read_to_vec().map_err(|e| e.to_string())
}

/// Run `work` in a background thread; the receiver gets its result and
/// `ctx` repaints then.
pub(crate) fn spawn<T: Send + 'static>(
    ctx: &Context,
    name: &str,
    work: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Receiver<Result<T, String>> {
    let (tx, rx) = std::sync::mpsc::channel();
    let ctx = ctx.clone();
    let started = std::thread::Builder::new().name(name.into()).spawn(move || {
        let _ = tx.send(work());
        ctx.request_repaint();
    });
    if let Err(e) = started {
        log::warn!("could not start {name}: {e}");
    }
    rx
}

fn read_index(ctx: &Context) -> Receiver<Result<Index, String>> {
    spawn(ctx, "plugin-index", || Index::parse(&get(store::INDEX_URL, store::MAX_INDEX)?))
}

fn start_install(
    ctx: &Context,
    plugin: IndexPlugin,
    version: IndexVersion,
    folder: PathBuf,
) -> Receiver<Result<PathBuf, String>> {
    spawn(ctx, "plugin-install", move || {
        let archive = get(&version.url, store::MAX_ARCHIVE)?;
        store::install(&archive, &plugin, &version, &folder)
    })
}

/// This build's version, for the index's `min_app`.
fn app_version() -> (u64, u64, u64) {
    store::parse_version(env!("CARGO_PKG_VERSION")).unwrap_or((0, 0, 0))
}

/// Check the background work: the index read, an install finished.
fn poll(state: &mut AppState) {
    let p = &mut state.plugins;
    if let IndexState::Reading(rx) = &p.index {
        p.index = match rx.try_recv() {
            Ok(Ok(index)) => IndexState::Read(index),
            Ok(Err(message)) => IndexState::Failed(message),
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Disconnected) => IndexState::Failed(String::new()),
        };
    }
    let Some(job) = &p.job else { return };
    let result = match job.result.try_recv() {
        Ok(r) => r,
        Err(TryRecvError::Empty) => return,
        Err(TryRecvError::Disconnected) => Err(String::new()),
    };
    let Some(Job { name, id, files, .. }) = p.job.take() else { return };
    match result {
        Ok(_) => {
            if files {
                state.prefs.plugins_files.insert(id);
            }
            load(state);
            let version = state.plugins.listed.iter().find(|l| l.name == name).map(|l| l.version.clone());
            let version = version.unwrap_or_default();
            state.plugins.notice = Some(Ok(t!("plug-installed-ok", "name" => name, "version" => version)));
        }
        Err(message) => {
            state.plugins.notice = Some(Err(t!("plug-install-failed", "name" => name, "message" => message)));
        }
    }
}

pub fn show(ctx: &Context, state: &mut AppState) {
    if !state.plugins.open {
        return;
    }
    poll(state);
    let mut reload = 0;
    let response = Dialog::new("plugins", t!("dlg-plugins")).width(860.0).show(
        ctx,
        |ui| body(ui, ctx, state),
        |f| {
            reload = f.extra(t!("plug-reload"));
            f.secondary(t!("action-close"));
        },
    );
    if response.dismissed {
        state.plugins.open = false;
        return;
    }
    match response.clicked {
        Some(i) if i == reload => {
            load(state);
            state.plugins.notice = None;
        }
        Some(_) => state.plugins.open = false,
        None => {}
    }
}

fn body(ui: &mut Ui, ctx: &Context, state: &mut AppState) {
    ui.horizontal(|ui| {
        let count = state.plugins.listed.len();
        let installed = t!("plug-tab-installed", "count" => count);
        if ui.selectable_label(state.plugins.tab == Tab::Installed, installed).clicked() {
            state.plugins.tab = Tab::Installed;
            state.plugins.selected = None;
        }
        if ui.selectable_label(state.plugins.tab == Tab::Available, t!("plug-tab-available")).clicked() {
            state.plugins.tab = Tab::Available;
            state.plugins.selected = None;
            if matches!(state.plugins.index, IndexState::NotRead) {
                state.plugins.index = IndexState::Reading(read_index(ctx));
            }
        }
        ui.add_space(12.0);
        ui.add(egui::TextEdit::singleline(&mut state.plugins.search).hint_text(t!("plug-search")).desired_width(220.0));
    });
    match state.prefs.plugins_dir() {
        Some(folder) => ui.label(theme::muted(t!("plug-folder", "path" => folder.display().to_string())).size(11.5)),
        None => ui.label(RichText::new(t!("plug-no-folder")).color(PALETTE.warning)),
    };
    match &state.plugins.notice {
        Some(Ok(text)) => {
            ui.label(RichText::new(text).color(PALETTE.success));
        }
        Some(Err(text)) => {
            ui.label(RichText::new(text).color(PALETTE.danger));
        }
        None => {}
    }
    ui.add_space(6.0);
    match state.plugins.tab {
        Tab::Installed => installed_tab(ui, state),
        Tab::Available => available_tab(ui, ctx, state),
    }
    ui.add_space(6.0);
    ui.label(theme::muted(t!("plug-review")).size(11.5));
}

fn matches_search(search: &str, texts: &[&str]) -> bool {
    let q = search.trim().to_lowercase();
    q.is_empty() || texts.iter().any(|t| t.to_lowercase().contains(&q))
}

/// The left column: the names, one selectable row each.
fn name_list(ui: &mut Ui, rows: &[(String, String, bool)], selected: &mut Option<String>, empty: &str) {
    egui::Frame::new().stroke(Stroke::new(1.0, PALETTE.border)).show(ui, |ui| {
        egui::ScrollArea::vertical().id_salt("plugin-list").max_height(340.0).auto_shrink([false, false]).show(
            ui,
            |ui| {
                if rows.is_empty() {
                    ui.label(theme::muted(empty));
                }
                for (id, label, faded) in rows {
                    let is = selected.as_deref() == Some(id);
                    let text =
                        if *faded { RichText::new(label).color(PALETTE.text_muted) } else { RichText::new(label) };
                    if ui.selectable_label(is, text).clicked() {
                        *selected = Some(id.clone());
                    }
                }
            },
        );
    });
}

fn installed_tab(ui: &mut Ui, state: &mut AppState) {
    let rows: Vec<(String, String, bool)> = state
        .plugins
        .listed
        .iter()
        .filter(|l| matches_search(&state.plugins.search, &[&l.name, &l.id, &l.description]))
        .map(|l| {
            let off = state.prefs.plugins_off.contains(&l.id);
            (l.id.clone(), format!("{}  ·  {}", l.name, l.version), off || l.error.is_some())
        })
        .collect();
    let mut selected = state.plugins.selected.clone();
    let mut changed = false;
    let mut remove = None;
    let mut settings_action = None;
    crate::dialog::two_columns(ui, [300.0, 540.0], |ui, column| {
        if column == 0 {
            name_list(ui, &rows, &mut selected, &t!("plug-none-installed"));
            return;
        }
        let Some(item) = selected.as_ref().and_then(|id| state.plugins.listed.iter().find(|l| &l.id == id)).cloned()
        else {
            ui.label(theme::muted(t!("plug-pick")));
            return;
        };
        header(ui, &item.name, &item.version, &item.authors, &item.license, &item.homepage, &item.description);
        if let Some(error) = &item.error {
            ui.label(RichText::new(t!("plug-error", "message" => error)).color(PALETTE.danger));
        }
        ui.add_space(4.0);
        let mut on = !state.prefs.plugins_off.contains(&item.id);
        if ui.checkbox(&mut on, t!("plug-enabled")).changed() {
            if on {
                state.prefs.plugins_off.remove(&item.id);
            } else {
                state.prefs.plugins_off.insert(item.id.clone());
            }
            changed = true;
        }
        if item.asks_files {
            let mut files = state.prefs.plugins_files.contains(&item.id);
            if ui.checkbox(&mut files, t!("plug-files")).changed() {
                if files {
                    state.prefs.plugins_files.insert(item.id.clone());
                } else {
                    state.prefs.plugins_files.remove(&item.id);
                }
                changed = true;
            }
            ui.label(theme::muted(t!("plug-files-hint")).size(11.5));
        }
        ui.add_space(6.0);
        ui.label(RichText::new(t!("plug-commands")).strong());
        if item.commands.is_empty() {
            ui.label(theme::muted(t!("plug-no-commands")));
        }
        egui::ScrollArea::vertical().id_salt("plugin-commands").max_height(150.0).show(ui, |ui| {
            for (name, description) in &item.commands {
                ui.label(RichText::new(name).monospace().size(12.0));
                if !description.is_empty() {
                    ui.label(theme::muted(description).size(11.5));
                }
            }
        });
        settings_action = settings_section(ui, state, &item);
        ui.add_space(6.0);
        if crate::dialog::styled_button(ui, &t!("plug-remove"), crate::dialog::Style::Danger).clicked() {
            remove = Some(item.clone());
        }
    });
    state.plugins.selected = selected;
    if changed {
        load(state);
    }
    match settings_action {
        Some(SettingsAction::Save) => save_settings(state),
        Some(SettingsAction::Reset) => write_settings(state, Map::new()),
        None => {}
    }
    if let Some(item) = remove {
        let folder = item.dir.parent().map(PathBuf::from).unwrap_or_default();
        state.plugins.notice = Some(match store::uninstall(&folder, &item.id) {
            Ok(()) => {
                state.prefs.plugins_off.remove(&item.id);
                state.prefs.plugins_files.remove(&item.id);
                state.plugins.selected = None;
                Ok(t!("plug-removed", "name" => item.name.clone()))
            }
            Err(message) => Err(t!("plug-remove-failed", "name" => item.name.clone(), "message" => message)),
        });
        load(state);
    }
}

/// What the buttons under a plugin's settings ask for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SettingsAction {
    Save,
    Reset,
}

/// The JSON Schema property the command form makes a setting's field from.
fn setting_property(s: &Setting) -> Value {
    let kind = match s.kind {
        SettingKind::Boolean => "boolean",
        SettingKind::Number => "number",
        SettingKind::Integer => "integer",
        SettingKind::String => "string",
    };
    let mut prop = json!({"type": kind, "title": s.title, "description": s.description, "default": s.default});
    if let Some(ui) = &s.ui {
        prop["ui"] = json!(ui);
    }
    if let Some(min) = s.minimum {
        prop["minimum"] = json!(min);
    }
    if let Some(max) = s.maximum {
        prop["maximum"] = json!(max);
    }
    if !s.enum_values.is_empty() {
        prop["enum"] = json!(s.enum_values);
        prop["enum_labels"] = json!(s.enum_labels);
    }
    prop
}

/// The settings of the plugin `item` with their stored values, read from
/// its manifest (its code does not run, so one turned off has them too).
fn settings_edit(state: &AppState, item: &Listed, fmt: &Fmt) -> Option<SettingsEdit> {
    let folder = state.prefs.plugins_dir()?;
    let (_, settings) = cc_plugins::plugin::settings_schema(&item.dir).ok()?;
    let values = current_values(&settings, &store::read_settings(&folder, &item.id));
    let properties: Map<String, Value> = settings.iter().map(|s| (s.key.clone(), setting_property(s))).collect();
    let keys: Vec<&str> = settings.iter().map(|s| s.key.as_str()).collect();
    let mut fields = crate::plugin_form::fields(&json!({"properties": properties, "required": keys}), fmt);
    for field in &mut fields {
        if let Some(value) = values.get(&field.name) {
            field.set_value(value.clone(), fmt);
        }
    }
    Some(SettingsEdit { id: item.id.clone(), settings, fields })
}

/// The Settings section of the selected plugin, with Save and Reset to
/// defaults under it; returns the button clicked.
fn settings_section(ui: &mut Ui, state: &mut AppState, item: &Listed) -> Option<SettingsAction> {
    if !item.has_settings {
        return None;
    }
    let fmt = Fmt::current();
    if state.plugins.settings.as_ref().is_none_or(|e| e.id != item.id) {
        state.plugins.settings = settings_edit(state, item, &fmt);
    }
    let off = state.prefs.plugins_off.contains(&item.id);
    let edit = state.plugins.settings.as_mut()?;
    ui.add_space(6.0);
    table::section(ui, &format!("plug-settings-{}", item.id), t!("plug-settings"), |ui| {
        for field in &mut edit.fields {
            let label = field.label.clone();
            let tip = (!field.description.is_empty()).then(|| field.description.clone());
            table::row_tip(ui, &label, tip.as_deref(), |ui| crate::plugin_form::plain_cell(ui, field, &fmt));
        }
    });
    if off {
        ui.label(theme::muted(t!("plug-settings-off")).size(11.5));
    }
    let mut action = None;
    ui.horizontal(|ui| {
        if ui.button(t!("plug-settings-save")).clicked() {
            action = Some(SettingsAction::Save);
        }
        if ui.button(t!("plug-settings-reset")).clicked() {
            action = Some(SettingsAction::Reset);
        }
    });
    action
}

/// Store the settings being edited: the valid values that differ from the
/// defaults (a value typed wrong keeps the one stored before).
fn save_settings(state: &mut AppState) {
    let Some(edit) = &state.plugins.settings else { return };
    let Some(folder) = state.prefs.plugins_dir() else { return };
    let stored = store::read_settings(&folder, &edit.id);
    let mut values = Map::new();
    for (setting, field) in edit.settings.iter().zip(&edit.fields) {
        let typed = match field.problem() {
            None => field.effective(),
            Some(_) => stored.get(&setting.key),
        };
        if let Some(value) = typed.and_then(|v| setting.validate(v).ok())
            && value != setting.default
        {
            values.insert(setting.key.clone(), value);
        }
    }
    write_settings(state, values);
}

/// Write the settings of the plugin being edited and load the plugins again,
/// so the plugin runs with them.
fn write_settings(state: &mut AppState, values: Map<String, Value>) {
    let Some(edit) = &state.plugins.settings else { return };
    let Some(folder) = state.prefs.plugins_dir() else { return };
    let id = edit.id.clone();
    let name = state.plugins.listed.iter().find(|l| l.id == id).map(|l| l.name.clone()).unwrap_or(id.clone());
    state.plugins.notice = Some(match store::write_settings(&folder, &id, &values) {
        Ok(()) => Ok(t!("plug-settings-saved", "name" => name)),
        Err(message) => Err(t!("plug-settings-failed", "name" => name, "message" => message)),
    });
    load(state);
}

/// Name, version, authors, license, page and description.
fn header(
    ui: &mut Ui,
    name: &str,
    version: &str,
    authors: &[String],
    license: &str,
    homepage: &str,
    description: &str,
) {
    ui.label(theme::title(name));
    let mut facts = Vec::new();
    if !version.is_empty() {
        facts.push(t!("plug-version", "version" => version));
    }
    if !authors.is_empty() {
        facts.push(t!("plug-by", "authors" => authors.join(", ")));
    }
    if !license.is_empty() {
        facts.push(t!("plug-license", "license" => license));
    }
    ui.label(theme::muted(facts.join("  ·  ")).size(11.5));
    if homepage.starts_with("https://") {
        ui.hyperlink_to(t!("plug-homepage"), homepage);
    }
    if !description.is_empty() {
        ui.add_space(4.0);
        ui.label(description);
    }
}

fn available_tab(ui: &mut Ui, ctx: &Context, state: &mut AppState) {
    let index = match &state.plugins.index {
        IndexState::NotRead | IndexState::Reading(_) => {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(t!("plug-loading"));
            });
            return;
        }
        IndexState::Failed(message) => {
            ui.label(RichText::new(t!("plug-index-failed", "message" => message)).color(PALETTE.danger));
            if ui.button(t!("plug-refresh")).clicked() {
                state.plugins.index = IndexState::Reading(read_index(ctx));
            }
            return;
        }
        IndexState::Read(index) => index.clone(),
    };
    if ui.button(t!("plug-refresh")).clicked() {
        state.plugins.index = IndexState::Reading(read_index(ctx));
    }
    let rows: Vec<(String, String, bool)> = index
        .plugins
        .iter()
        .filter(|p| matches_search(&state.plugins.search, &[&p.name, &p.id, &p.description, &p.tags.join(" ")]))
        .map(|p| {
            let best = p.best(app_version());
            let shown = best.or_else(|| p.newest());
            let label = format!("{}  ·  {}", p.name, shown.map_or("", |v| v.version.as_str()));
            (p.id.clone(), label, best.is_none())
        })
        .collect();
    let mut selected = state.plugins.selected.clone();
    let mut install = None;
    crate::dialog::two_columns(ui, [300.0, 540.0], |ui, column| {
        if column == 0 {
            name_list(ui, &rows, &mut selected, &t!("plug-index-empty"));
            return;
        }
        let Some(plugin) = selected.as_ref().and_then(|id| index.plugins.iter().find(|p| &p.id == id)) else {
            ui.label(theme::muted(t!("plug-pick")));
            return;
        };
        let best = plugin.best(app_version());
        // An incompatible plugin shows its newest version and what it needs.
        let shown = best.or_else(|| plugin.newest());
        let version = shown.map_or_else(String::new, |v| v.version.clone());
        let homepage = plugin.homepage.clone().or_else(|| plugin.repository.clone()).unwrap_or_default();
        header(ui, &plugin.name, &version, &plugin.authors, &plugin.license, &homepage, &plugin.description);
        if let Some(notes) = shown.and_then(|v| v.notes.as_ref()) {
            ui.label(theme::muted(notes).size(11.5));
        }
        ui.add_space(6.0);
        let Some(best) = best else {
            let current = env!("CARGO_PKG_VERSION");
            let why = match shown.and_then(|v| v.min_app.clone()) {
                Some(min) if shown.is_some_and(|v| v.api <= cc_plugins::API_VERSION) => {
                    t!("plug-needs-app", "version" => min, "current" => current)
                }
                _ => t!("plug-incompatible"),
            };
            ui.label(RichText::new(why).color(PALETTE.warning));
            return;
        };
        let asks_files = best.permissions.iter().any(|p| p == "files");
        if asks_files {
            let mut allow = state.plugins.allow_files.contains(&plugin.id);
            if ui.checkbox(&mut allow, t!("plug-files")).changed() {
                if allow {
                    state.plugins.allow_files.insert(plugin.id.clone());
                } else {
                    state.plugins.allow_files.remove(&plugin.id);
                }
            }
            ui.label(theme::muted(t!("plug-files-hint")).size(11.5));
        }
        let installed = state.plugins.listed.iter().find(|l| l.id == plugin.id);
        let newer = installed.is_none_or(|l| {
            store::parse_version(&l.version)
                .is_none_or(|have| store::parse_version(&best.version).is_some_and(|v| v > have))
        });
        if let Some(job) = &state.plugins.job {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(t!("plug-installing", "name" => job.name.clone()));
            });
        } else if !newer {
            ui.label(theme::muted(
                t!("plug-installed-version", "version" => installed.map(|l| l.version.clone()).unwrap_or_default()),
            ));
        } else {
            let label = if installed.is_some() {
                t!("plug-update", "version" => best.version.clone())
            } else {
                t!("plug-install")
            };
            if crate::dialog::styled_button(ui, &label, crate::dialog::Style::Primary).clicked() {
                install = Some((plugin.clone(), best.clone(), asks_files));
            }
        }
    });
    state.plugins.selected = selected;
    if let Some((plugin, version, asks_files)) = install {
        let Some(folder) = state.prefs.plugins_dir() else { return };
        let files = asks_files && state.plugins.allow_files.contains(&plugin.id);
        state.plugins.notice = None;
        let (name, id) = (plugin.name.clone(), plugin.id.clone());
        let result = start_install(ctx, plugin, version, folder);
        state.plugins.job = Some(Job { name, id, files, result });
    }
}

/// Show `index` in the Available tab as if it had been read.
#[cfg(test)]
pub fn set_index(state: &mut AppState, index: Index) {
    state.plugins.index = IndexState::Read(index);
    state.plugins.tab = Tab::Available;
}

/// Wait for the install running (tests: it fails at once, offline).
#[cfg(test)]
pub fn finish_job(state: &mut AppState) {
    if let Some(job) = &state.plugins.job
        && let Ok(result) = job.result.recv_timeout(Duration::from_secs(10))
    {
        let Some(Job { name, .. }) = state.plugins.job.take() else { return };
        state.plugins.notice = Some(match result {
            Ok(_) => Ok(name),
            Err(message) => Err(t!("plug-install-failed", "name" => name, "message" => message)),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cc_core::{Category, ElementId};
    use std::collections::BTreeSet;

    fn state() -> AppState {
        let project = crate::home::sample_project("casa_simples").unwrap();
        AppState::new(cc_core::Document::new(project))
    }

    fn first(state: &AppState, category: Category) -> ElementId {
        state.project().elements.values().find(|e| e.category() == category).unwrap().id
    }

    #[test]
    fn when_terms_all_hold_or_the_first_that_does_not_tells_why() {
        let mut s = state();
        let (wall, door) = (first(&s, Category::Wall), first(&s, Category::Door));
        let walls = vec![When::SelectionOf("wall".into())];
        let one = [When::SelectionOne];
        assert_eq!(unavailable(&s, &[]), None, "no terms: always");
        assert_eq!(unavailable(&s, &[When::Selection]), Some(t!("plug-when-selection")));
        assert_eq!(unavailable(&s, &one), Some(t!("plug-when-one")));
        let why_walls = t!("plug-when-category", "category" => crate::actions::t_category(Category::Wall));
        assert_eq!(unavailable(&s, &walls), Some(why_walls.clone()), "nothing selected");
        s.selection = BTreeSet::from([wall]);
        assert_eq!(unavailable(&s, &[When::Selection, When::SelectionOne, walls[0].clone()]), None);
        s.selection.insert(door);
        assert_eq!(unavailable(&s, &[When::Selection]), None);
        assert_eq!(unavailable(&s, &one), Some(t!("plug-when-one")));
        assert_eq!(unavailable(&s, &walls), Some(why_walls), "a door too");
        assert!(unavailable(&s, &[When::SelectionOf("door".into())]).is_some());
        // A selected id that is not in the project does not count as a wall.
        s.selection = BTreeSet::from([wall, ElementId(u64::MAX)]);
        assert!(unavailable(&s, &walls).is_some());
        // The active view's kind.
        let three_d = [When::View("three_d".into())];
        let plan = s.project().views.values().find(|v| v.category() == ViewCategory::FloorPlan).unwrap().id;
        let view3d = s.project().views.values().find(|v| v.category() == ViewCategory::ThreeD).unwrap().id;
        s.active_view = Some(plan);
        let why = t!("plug-when-view", "kind" => crate::i18n::tr(ViewCategory::ThreeD.key()));
        assert_eq!(unavailable(&s, &three_d), Some(why));
        assert_eq!(unavailable(&s, &[When::View("floor_plan".into())]), None);
        s.active_view = Some(view3d);
        assert_eq!(unavailable(&s, &three_d), None);
        s.active_view = None;
        assert!(unavailable(&s, &three_d).is_some(), "no view open");
        // Every view kind a manifest may name has its view category.
        for kind in cc_plugins::contributions::VIEW_KINDS {
            assert!(ViewCategory::ALL.into_iter().any(|c| view_kind(c) == kind), "{kind}");
        }
    }
}
