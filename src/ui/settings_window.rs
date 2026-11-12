//! Ventana de Ajustes (todas las secciones salvo Biblioteca, que llega en el hito 3)
//! y ventana del registro.

use super::{App, Dialog, action_name, shortcut_label, shortcut_of, theme_pref};
use crate::editor::Tool;
use crate::i18n::I18n;
use crate::settings::{self, ACTIONS, Action, Settings, StartView, Theme};
use eframe::egui::{self, Key};

/// Lista desplegable para elegir entre valores con nombre.
fn combo<T: PartialEq + Copy>(ui: &mut egui::Ui, id: &str, value: &mut T, options: &[(T, String)]) {
    let current = options
        .iter()
        .find(|(v, _)| v == value)
        .map(|(_, n)| n.clone())
        .unwrap_or_default();
    egui::ComboBox::from_id_salt(id)
        .selected_text(current)
        .show_ui(ui, |ui| {
            for (v, n) in options {
                ui.selectable_value(value, *v, n);
            }
        });
}

fn row(ui: &mut egui::Ui, label: String, add: impl FnOnce(&mut egui::Ui)) {
    ui.label(label);
    add(ui);
    ui.end_row();
}

impl App {
    pub(super) fn settings_window(&mut self, ctx: &egui::Context) {
        let mut open = self.show_settings;
        let before = self.settings.clone();
        let mut reset = false;
        let mut show_log = false;
        let i = &self.i18n;
        egui::Window::new(i.tr("settings-title"))
            .open(&mut open)
            .default_width(420.0)
            .vscroll(true)
            .show(ctx, |ui| {
                let s = &mut self.settings;
                egui::CollapsingHeader::new(i.tr("settings-general"))
                    .default_open(true)
                    .show(ui, |ui| general(ui, i, s));
                egui::CollapsingHeader::new(i.tr("settings-editor"))
                    .default_open(true)
                    .show(ui, |ui| editor(ui, i, s));
                egui::CollapsingHeader::new(i.tr("settings-shortcuts")).show(ui, |ui| {
                    shortcuts_editor(ui, i, s, &mut self.binding, &mut self.bind_msg);
                });
                egui::CollapsingHeader::new(i.tr("settings-3d")).show(ui, |ui| view_3d(ui, i, s));
                egui::CollapsingHeader::new(i.tr("settings-export"))
                    .show(ui, |ui| export(ui, i, s));
                egui::CollapsingHeader::new(i.tr("settings-advanced")).show(ui, |ui| {
                    reset = ui.button(i.tr("settings-reset")).clicked();
                    show_log = ui.button(i.tr("settings-view-log")).clicked();
                    ui.label(i.tr("settings-data-path"));
                    for p in [settings::log::data_dir(), self.settings_path.clone()]
                        .into_iter()
                        .flatten()
                    {
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(p.display().to_string()).monospace(),
                            )
                            .selectable(true),
                        );
                    }
                });
            });
        if !open {
            self.binding = None;
        }
        self.show_settings = open;
        if reset {
            self.dialog = Some(Dialog::ResetSettings);
        }
        if show_log {
            self.log_text = Some(read_log(&self.i18n));
        }

        if self.settings != before {
            ctx.set_theme(theme_pref(self.settings.theme));
            self.editor.set_limit(self.settings.undo_levels);
            self.settings_changed = true;
        }
        // Se escribe al soltar el ratón, no en cada cuadro mientras se arrastra un control.
        if self.settings_changed && !ctx.input(|i| i.pointer.any_down()) {
            self.settings_changed = false;
            if let Some(path) = &self.settings_path
                && let Err(e) = self.settings.save_to(path)
            {
                log::error!("no se pudieron guardar los ajustes: {e}");
                self.status = self
                    .i18n
                    .tr_args("error-io", &[("detail", e.to_string().into())]);
            }
        }
    }

    pub(super) fn log_window(&mut self, ctx: &egui::Context) {
        let Some(text) = &mut self.log_text else {
            return;
        };
        let mut open = true;
        let mut refresh = false;
        egui::Window::new(self.i18n.tr("log-title"))
            .open(&mut open)
            .default_size([640.0, 360.0])
            .show(ctx, |ui| {
                refresh = ui.button(self.i18n.tr("log-refresh")).clicked();
                egui::ScrollArea::both()
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        ui.add(
                            egui::Label::new(egui::RichText::new(text.as_str()).monospace())
                                .selectable(true),
                        );
                    });
            });
        if refresh {
            self.log_text = Some(read_log(&self.i18n));
        }
        if !open {
            self.log_text = None;
        }
    }
}

fn read_log(i: &I18n) -> String {
    settings::log::path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| i.tr("log-empty"))
}

fn general(ui: &mut egui::Ui, i: &I18n, s: &mut Settings) {
    egui::Grid::new("general").num_columns(2).show(ui, |ui| {
        row(ui, i.tr("settings-language"), |ui| {
            let langs: Vec<(&str, String)> = settings::LANGUAGES
                .iter()
                .map(|&l| (l, i.tr("lang-es")))
                .collect();
            let mut lang = s.language.as_str();
            combo(ui, "lang", &mut lang, &langs);
            s.language = lang.to_string();
        });
        row(ui, i.tr("settings-theme"), |ui| {
            combo(
                ui,
                "theme",
                &mut s.theme,
                &[
                    (Theme::System, i.tr("theme-system")),
                    (Theme::Dark, i.tr("theme-dark")),
                    (Theme::Light, i.tr("theme-light")),
                ],
            )
        });
        row(ui, i.tr("settings-start-view"), |ui| {
            combo(
                ui,
                "start-view",
                &mut s.start_view,
                &[
                    (StartView::Last, i.tr("start-view-last")),
                    (StartView::Flat, i.tr("view-2d")),
                    (StartView::ThreeD, i.tr("view-3d")),
                ],
            )
        });
        row(ui, i.tr("settings-open-last"), |ui| {
            ui.checkbox(&mut s.open_last, "");
        });
    });
}

fn editor(ui: &mut egui::Ui, i: &I18n, s: &mut Settings) {
    egui::Grid::new("editor").num_columns(2).show(ui, |ui| {
        row(ui, i.tr("settings-pixel-size"), |ui| {
            ui.add(egui::Slider::new(&mut s.pixel_size, settings::PIXEL_SIZE));
        });
        row(ui, i.tr("settings-grid"), |ui| {
            ui.checkbox(&mut s.show_grid, "");
        });
        row(ui, i.tr("settings-labels"), |ui| {
            ui.checkbox(&mut s.show_labels, "");
        });
        row(ui, i.tr("settings-undo"), |ui| {
            ui.add(egui::DragValue::new(&mut s.undo_levels).range(settings::UNDO_LEVELS));
        });
        row(ui, i.tr("settings-autosave"), |ui| {
            ui.horizontal(|ui| {
                ui.checkbox(&mut s.autosave, "")
                    .on_hover_text(i.tr("settings-autosave-hint"));
                ui.add_enabled(
                    s.autosave,
                    egui::DragValue::new(&mut s.autosave_minutes)
                        .range(settings::AUTOSAVE_MINUTES)
                        .suffix(i.tr("unit-minutes")),
                );
            });
        });
        row(ui, i.tr("settings-start-tool"), |ui| {
            combo(
                ui,
                "start-tool",
                &mut s.start_tool,
                &[
                    (Tool::Pencil, i.tr("tool-pencil")),
                    (Tool::Eraser, i.tr("tool-eraser")),
                    (Tool::Fill, i.tr("tool-fill")),
                    (Tool::Picker, i.tr("tool-picker")),
                    (Tool::Gradient, i.tr("tool-gradient")),
                    (Tool::Stamp, i.tr("tool-stamp")),
                    (Tool::Select, i.tr("tool-select")),
                ],
            )
        });
    });
}

fn view_3d(ui: &mut egui::Ui, i: &I18n, s: &mut Settings) {
    egui::Grid::new("view3d").num_columns(2).show(ui, |ui| {
        row(ui, i.tr("settings-background"), |ui| {
            ui.horizontal(|ui| {
                let mut custom = s.background_3d.is_some();
                ui.checkbox(&mut custom, i.tr("settings-background-custom"));
                match (custom, &mut s.background_3d) {
                    (true, Some(c)) => {
                        ui.color_edit_button_srgb(c);
                    }
                    (true, None) => s.background_3d = Some([40, 44, 52]),
                    (false, _) => s.background_3d = None,
                }
            });
        });
        row(ui, i.tr("settings-lighting"), |ui| {
            ui.add(egui::Slider::new(&mut s.lighting, settings::LIGHTING));
        });
        row(ui, i.tr("settings-rotate-speed"), |ui| {
            ui.add(egui::Slider::new(
                &mut s.rotate_speed,
                settings::ROTATE_SPEED,
            ));
        });
        row(ui, i.tr("settings-shadow"), |ui| {
            ui.checkbox(&mut s.shadow, "");
        });
    });
    ui.weak(i.tr("settings-capture-note"));
}

fn export(ui: &mut egui::Ui, i: &I18n, s: &mut Settings) {
    egui::Grid::new("export").num_columns(2).show(ui, |ui| {
        row(ui, i.tr("settings-export-dir"), |ui| {
            ui.horizontal(|ui| {
                let shown = s.export_dir.as_ref().map_or_else(
                    || i.tr("settings-export-dir-none"),
                    |p| p.display().to_string(),
                );
                ui.label(shown);
                if ui.button(i.tr("settings-choose")).clicked()
                    && let Some(dir) = rfd::FileDialog::new().pick_folder()
                {
                    s.export_dir = Some(dir);
                }
                if s.export_dir.is_some() && ui.button(i.tr("shortcut-clear")).clicked() {
                    s.export_dir = None;
                }
            });
        });
        row(ui, i.tr("settings-export-name"), |ui| {
            ui.text_edit_singleline(&mut s.export_name)
                .on_hover_text(i.tr("settings-export-name-hint"));
        });
        row(ui, i.tr("settings-confirm-overwrite"), |ui| {
            ui.checkbox(&mut s.confirm_overwrite, "");
        });
    });
    ui.weak(i.tr("settings-export-note"));
}

/// Lista de atajos: clic en uno y pulsar la combinación nueva. Esc cancela.
fn shortcuts_editor(
    ui: &mut egui::Ui,
    i: &I18n,
    s: &mut Settings,
    binding: &mut Option<Action>,
    msg: &mut String,
) {
    if let Some(action) = *binding {
        let pressed = ui.input(|inp| {
            inp.events.iter().find_map(|e| match e {
                egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } => Some((*key, *modifiers)),
                _ => None,
            })
        });
        match pressed {
            Some((Key::Escape, _)) => *binding = None,
            Some((key, m)) => {
                *binding = None;
                match s.bind(action, Some(shortcut_of(key, m))) {
                    Ok(()) => msg.clear(),
                    Err(other) => {
                        *msg = i.tr_args(
                            "shortcut-clash",
                            &[("action", action_name(i, other).into())],
                        )
                    }
                }
            }
            None => {}
        }
    }
    if !msg.is_empty() {
        ui.colored_label(ui.visuals().warn_fg_color, msg.as_str());
    }
    egui::ScrollArea::vertical()
        .max_height(300.0)
        .show(ui, |ui| {
            egui::Grid::new("shortcuts").num_columns(3).show(ui, |ui| {
                for action in ACTIONS {
                    ui.label(action_name(i, action));
                    let text = if *binding == Some(action) {
                        i.tr("shortcut-press")
                    } else {
                        s.shortcut(action)
                            .map_or_else(|| i.tr("shortcut-none"), |sc| shortcut_label(i, &sc))
                    };
                    if ui.button(text).clicked() {
                        *binding = Some(action);
                        msg.clear();
                    }
                    if ui.small_button(i.tr("shortcut-clear")).clicked() {
                        let _ = s.bind(action, None);
                    }
                    ui.end_row();
                }
            });
        });
    if ui.button(i.tr("shortcut-reset")).clicked() {
        s.shortcuts.clear();
        msg.clear();
    }
}
