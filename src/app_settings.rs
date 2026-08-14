use std::{
    fmt::Display,
    fs::read_to_string,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use anyhow::{Result, anyhow};
use eframe::egui::{ComboBox, RichText, Ui};
use fluent_templates::LanguageIdentifier;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};

use crate::{
    APP_FILE_NAME,
    app::{AppStatus, BETWEEN_FIELDS, EDGE_COLUMN_WIDTH, INDENT_COLUMN_WIDTH},
    languages::{Language, Languages, select_language},
    localize::{CURRENT_LANGUAGES, LANGUAGE_LOADER, LANGUAGES_LIST, fl, language_name},
};

const SETTINGS_EXT: &str = "toml"; // do better than this
pub const DEFAULT_ZOOM: f32 = 1.0;

const APP_SETTINGS_FILENAME: LazyLock<PathBuf> =
    LazyLock::new(|| Path::new(&*APP_FILE_NAME).with_extension(SETTINGS_EXT));

pub fn app_settings_file_path(config_path: &Path) -> PathBuf {
    let filename = &*APP_SETTINGS_FILENAME;
    config_path.join(filename)
}

/// Settings needed to start the app
#[derive(Debug, Clone)]
pub struct AppSettings {
    theme: Theme,
    zoom: f32,
    default_master_language: Language,
    ui_language: LanguageIdentifier, // not yet used
                                     // autoload: Option<String>,  // name of data file to load
}

#[allow(dead_code)]
impl AppSettings {
    pub fn new(master_language: Language) -> Self {
        let current_language = unsafe {
            let language_loader = &*LANGUAGE_LOADER;
            language_loader.current_language()
        };
        AppSettings {
            theme: Theme::default(),
            zoom: DEFAULT_ZOOM,
            default_master_language: master_language,
            ui_language: current_language.clone(),
        }
    }

    pub fn load(settings_path: &Path) -> Result<Self> {
        settings_load(settings_path)
    }

    pub fn theme(&self) -> eframe::egui::Theme {
        self.theme.into()
    }

    pub fn master_language(&self) -> Language {
        self.default_master_language
    }

    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    pub fn show_and_edit(&mut self, ui: &mut Ui) -> Option<AppStatus> {
        let mut ret = None;

        ui.horizontal(|ui| {
            ui.add_space(EDGE_COLUMN_WIDTH);

            ui.vertical(|ui| {
                let extras_heading = RichText::from(fl!("settings_heading")).heading();
                ui.label(extras_heading);

                ui.add_space(16.0);

                let extras_description = RichText::from(fl!("settings_description"));
                ui.label(extras_description);

                ui.add_space(20.0);

                ui.horizontal(|ui| {
                    ui.add_space(INDENT_COLUMN_WIDTH);

                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            // ui language
                            let languages = &*LANGUAGES_LIST;

                            let ui_lang_label =
                                RichText::from(fl!("settings_ui_language")).strong();
                            ui.label(ui_lang_label);

                            let mut selected = languages
                                .iter()
                                .position(|li| li == &self.ui_language)
                                .unwrap();
                            let before = selected;

                            ComboBox::from_id_salt("UiLang").show_index(
                                ui,
                                &mut selected,
                                languages.len(),
                                |l| language_name(languages[l].language.as_str()), // translate?
                            );

                            if selected != before {
                                // Handle selection change
                                if let Some(language) = languages.get(selected) {
                                    self.ui_language = language.clone();
                                    unsafe {
                                        #[allow(static_mut_refs)]
                                        CURRENT_LANGUAGES.set_language(language.clone());
                                    }

                                    info!("changed ui language to {}", language.language.as_str())
                                } else {
                                    warn!("unable to set language {selected}");
                                }
                            }
                        });

                        ui.add_space(BETWEEN_FIELDS);

                        ui.horizontal(|ui| {
                            // theme
                            let theme_label = RichText::from(fl!("settings_theme")).strong();
                            ui.label(theme_label);

                            ComboBox::from_id_salt("Theme")
                                .selected_text(format!("{:?}", self.theme))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.theme,
                                        Theme::Light,
                                        Theme::Light.to_string(),
                                    );
                                    ui.selectable_value(
                                        &mut self.theme,
                                        Theme::Dark,
                                        Theme::Dark.to_string(),
                                    );
                                });
                        });

                        ui.add_space(BETWEEN_FIELDS);

                        ui.horizontal(|ui| {
                            // default master language
                            let master_language_label =
                                RichText::from(fl!("settings_master_language")).strong();
                            ui.label(master_language_label);

                            let mut selected = self.default_master_language.language_index();
                            let before = selected;
                            select_language(&mut selected, ui);

                            if selected != before {
                                // Handle selection change
                                if let Some(language) = Languages::from_index(selected) {
                                    self.default_master_language = language;
                                } else {
                                    warn!("unable to set language {selected}");
                                }
                            }
                        });

                        ui.add_space(BETWEEN_FIELDS * 2.0);

                        let done_text = RichText::from(fl!("settings_done"));
                        if ui.button(done_text).clicked() {
                            // should save settings - require another state?
                            ret = Some(AppStatus::Ready);
                        }
                    })
                })
            })
        });

        ret
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct SettingsVersion {
    save_version: u16,
}

// !! This is where to set the new save settings to use
type SaveSettings = SaveSettings1;

/// Save settings in current save version
fn settings_save(settings: &AppSettings, file_name: &Path) -> Result<()> {
    let save_settings: SaveSettings = settings.into();

    Ok(())
}

/// Load settings from any version of save data
fn settings_load(file_name: &Path) -> Result<AppSettings> {
    let data = read_to_string(file_name)?;

    let Ok(SettingsVersion { save_version }) = toml::from_str(&data) else {
        error!("unable to parse version");
        return Err(anyhow!("unable to parse save settings version"));
    };

    match save_version {
        SAVE1_VERSION => {
            info!("loading settings version {}", SAVE1_VERSION);
            let Ok(settings) = toml::from_str::<SaveSettings1>(&data) else {
                error!("unable to parse save settings 1");
                return Err(anyhow!("unable to parse save as settings version 1"));
            };
            if !settings.validate() {
                error!("settings does not validate");
                return Err(anyhow!("settings does not validate"));
            }
            Ok(settings.into())
        }

        _ => {
            error!("unknown version {save_version} for settings");
            Err(anyhow!("unknown version {save_version} for settings"))
        }
    }
}

// ====================
// SaveSettings1
const SAVE1_VERSION: u16 = 1;

#[derive(Debug, Serialize, Deserialize)] // Serialize, Deserialize
struct SaveSettings1 {
    save_version: u16,
    theme: Theme,
    default_master_language: String,
    ui_language: String,
}

impl SaveSettings1 {
    fn validate(&self) -> bool {
        self.save_version == SAVE1_VERSION
    }
}

impl From<SaveSettings1> for AppSettings {
    fn from(value: SaveSettings1) -> Self {
        let default_master_language =
            if let Some(language) = Languages::find_language(&value.default_master_language) {
                language
            } else {
                error!("Unable to parse master language; using English instead!");
                Language::English
            };
        let ui_language = value.ui_language.parse().unwrap(); // !! better error handling than this -> fallback to default
        AppSettings {
            theme: value.theme,
            zoom: DEFAULT_ZOOM,
            default_master_language,
            ui_language,
        }
    }
}

impl From<&AppSettings> for SaveSettings1 {
    fn from(value: &AppSettings) -> Self {
        SaveSettings1 {
            save_version: SAVE1_VERSION,
            theme: value.theme,
            default_master_language: value.default_master_language.name().to_string(),
            ui_language: value.ui_language.language.to_string(),
        }
    }
}

// -----------------------------
// eGUI Theme Save

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)] // Deserialize, Serialize,
enum Theme {
    Dark,
    #[default]
    Light,
}

impl From<eframe::egui::Theme> for Theme {
    fn from(value: eframe::egui::Theme) -> Self {
        match value {
            eframe::egui::Theme::Dark => Theme::Dark,
            eframe::egui::Theme::Light => Theme::Light,
        }
    }
}

impl From<Theme> for eframe::egui::Theme {
    fn from(value: Theme) -> Self {
        match value {
            Theme::Dark => eframe::egui::Theme::Dark,
            Theme::Light => eframe::egui::Theme::Light,
        }
    }
}

impl Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Theme::*;
        write!(
            f,
            "{}",
            match self {
                Light => "Light",
                Dark => "Dark",
            }
        )
    }
}
