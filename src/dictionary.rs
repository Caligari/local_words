use std::{
    collections::{BTreeMap, HashMap},
    fmt::Display,
    hash::Hash,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::Result;
use eframe::egui::{ComboBox, Key, Label, RichText, Sense, Separator, Ui};
use enum_iterator::{Sequence, all};
use log::{debug, error, info};
use versions::SemVer;

use crate::{
    app::{
        ACTIVE_COLOR, AppStatus, BETWEEN_FIELDS, EDGE_COLUMN_WIDTH, MISSING_COLOR, MOD_MAIN_COLOR,
        MOD_TRANS_COLOR, SMALL_SPACE, STRING_HEIGHT, STRING_RECT, TINY_SPACE,
    },
    languages::{Language, Languages},
    loader::Loader,
    localize::{arg, fl},
    translation::Translation,
    writer::write_lang_csv,
};

const ACHIEVE_NAME: &str = "_achievements";
const PRESENCE_NAME: &str = "_presence";

pub const INTERNAL_EXT: &str = "vrt";
pub const JSON_EXT: &str = "json";
pub const CSV_EXT: &str = "csv";
pub const XLSX_EXT: &str = "xlsx";

pub type TagList = Vec<String>;
pub type ContextList = Vec<String>;

// ====================
// Dictionary
//
/// This struct contains the primary language and all translations, as well as the infomation
/// needed to show that data.
#[derive(Debug)]
pub struct Dictionary {
    primary_language: Language,
    // paths: LocationPaths, // ?
    words: Words,
    // history?
    show: ShowState,
    // version_history: ContentVersions,
}

impl Dictionary {
    pub fn new(primary_language: Language, loader: &Loader) -> Result<Self> {
        info!("creating Dictionary");
        let loaded = loader.load_primary_language(primary_language)?;
        info!(
            "load of primary language complete: {} tags, {} lines, {} unknowns, {} words",
            loaded.len(),
            loaded.lines(),
            loaded.unknowns(),
            loaded.words()
        );

        let tags = loaded.tags();
        let primary = Translation::from_loaded(&loaded, primary_language, &tags);

        // let paths = LocationPaths(HashMap::new()); // do we need this?

        info!("Dictionary created");
        Ok(Dictionary {
            primary_language,
            // paths,
            words: Words::new(primary, tags),
            show: ShowState::default_not(primary_language),
        })
    }

    pub fn load_translation(
        &mut self,
        language: Language,
        loader: &Loader,
        language_type: LanguageType,
    ) -> Result<()> {
        let tags = &self.words.tags;
        let loaded = loader.load_translation(language, tags)?;
        let translation = Translation::from_loaded(&loaded, language, tags);

        info!(
            "load of {language} translation complete: {} tags, {} lines, {} unknowns, {} words",
            loaded.len(),
            loaded.lines(),
            loaded.unknowns(),
            loaded.words()
        );

        self.add_translation(translation, language_type);
        Ok(())
    }

    pub fn show(&mut self, ui: &mut Ui) -> Option<AppStatus> {
        use DictionaryState::*;

        let mut new_show_state = None;

        ui.vertical(|ui| {
            // Main selector
            ui.horizontal(|ui| {
                // handle mouse over, selected (color and underline)

                // Overview?
                let over_text = {
                    let mut text = RichText::new(fl!("show_overview")).heading();
                    if matches!(self.show.state, Overview) {
                        text = text.color(ACTIVE_COLOR).underline();
                    }
                    text
                };
                // Primary Language
                let primary_language = self.primary_language.to_string();
                let prime_text = {
                    let mut text =
                        RichText::new(fl!("show_primary_lang", arg!("lang", primary_language)))
                            .heading();
                    if matches!(self.show.state, Primary) {
                        text = text.color(ACTIVE_COLOR).underline();
                    }
                    text
                };
                // Translation
                let trans_text = {
                    let mut text = RichText::new(fl!("show_translation")).heading();
                    if matches!(self.show.state, Translation(..)) {
                        text = text.color(ACTIVE_COLOR).underline();
                    }
                    text
                };
                // Export?
                let export_text = {
                    let mut text = RichText::new(fl!("show_export")).heading();
                    if matches!(self.show.state, Export) {
                        text = text.color(ACTIVE_COLOR).underline();
                    }
                    text
                };

                ui.add_space(SMALL_SPACE);

                if ui // no click if already selected?
                    .add(Label::new(over_text).sense(Sense::click()))
                    .clicked()
                {
                    debug!("clicked on overview");
                    new_show_state = Some(DictionaryState::Overview);
                }
                ui.add_space(TINY_SPACE);
                ui.add(Separator::default().spacing(TINY_SPACE));
                ui.add_space(TINY_SPACE);

                if ui
                    .add(Label::new(prime_text).sense(Sense::click()))
                    .clicked()
                {
                    debug!("clicked on primary");
                    new_show_state = Some(DictionaryState::Primary);
                }
                ui.add_space(TINY_SPACE);
                ui.add(Separator::default().spacing(TINY_SPACE));
                ui.add_space(TINY_SPACE);

                if ui // only click if language selected
                    .add(Label::new(trans_text).sense(Sense::click()))
                    .clicked()
                {
                    debug!("clicked on translation");
                    new_show_state =
                        Some(DictionaryState::Translation(self.show.selected_translation));
                }
                if self.show.select_language(ui) {
                    // selected new language
                    new_show_state =
                        Some(DictionaryState::Translation(self.show.selected_translation));
                }
                ui.add_space(TINY_SPACE);
                ui.add(Separator::default().spacing(TINY_SPACE));
                ui.add_space(TINY_SPACE);

                if ui
                    .add(Label::new(export_text).sense(Sense::click()))
                    .clicked()
                {
                    debug!("clicked on export");
                    new_show_state = Some(DictionaryState::Export);
                }
            });

            ui.separator();
            ui.add_space(SMALL_SPACE);

            if let Some(new_state) = new_show_state {
                if new_state != self.show.state {
                    info!("{} -> {}", self.show.state, new_state);
                    self.show.state = new_state;
                }
            }

            ui.horizontal(|ui| {
                ui.add_space(EDGE_COLUMN_WIDTH);

                // The main content
                match &self.show.state {
                    Overview => self.show_overview(ui),
                    Primary => self.show_primary(ui),
                    Translation(lang) => self.show_translation(*lang, ui),
                    Export => self.show_export(ui),
                    // _ => warn!("not implemented directory state"),
                }
            });
        });

        None
    }

    fn show_overview(&self, ui: &mut Ui) {
        // Overview
        ui.vertical(|ui| {
            let primary_lang =
                format!("{} {}", fl!("show_master_language"), self.primary_language,);
            ui.label(primary_lang);
            let number_tags = format!("{} {}", fl!("show_number_tags"), self.words.number_tags());
            ui.label(number_tags);

            ui.add_space(BETWEEN_FIELDS);

            let version_text = format!("{}", self.words.versions,);
            ui.label(version_text);

            ui.add_space(BETWEEN_FIELDS);

            let mut trans = self.translations();
            trans.sort_by(|a, b| {
                let a_name = a.language().name();
                let b_name = b.language().name();
                a_name.cmp(&b_name)
            });
            for tr in trans {
                ui.label(tr.overview_string());
            }

            ui.add_space(BETWEEN_FIELDS * 2.0);

            ui.horizontal(|ui| {
                let in_trans_file = fl!("import_translation_file");
                let in_trans_zip = fl!("import_translation_zip");

                if ui.button(in_trans_file).clicked() {
                    debug!("clicked import translation file");
                }

                ui.add_space(BETWEEN_FIELDS);

                if ui.button(in_trans_zip).clicked() {
                    debug!("clicked import translation zip");
                }
            });

            ui.add_space(BETWEEN_FIELDS);

            ui.horizontal(|ui| {
                let up_trans_file = fl!("update_translation_file");
                let up_trans_zip = fl!("update_translation_zip");

                if ui.button(up_trans_file).clicked() {
                    debug!("clicked import translation update file");
                }

                ui.add_space(BETWEEN_FIELDS);

                if ui.button(up_trans_zip).clicked() {
                    debug!("clicked import translation update zip");
                }
            });

            // import core updates all the core_translations (and the master)
            // import external creates/updates work_translations
            //
            // is import for mod or core?
            // import xlsx from translator
            // import xlsx files in zip from translator
            // import vrt file from mod
            // import vrt files in zip from mod
            //
            // assimilate selected languages into new baseline version
        });
    }

    fn show_string(&mut self, language: Option<Language>, ui: &mut Ui) {
        let missing_lines = self.words.missing_lines(language);
        ui.vertical(|ui| {
            // tag selector
            self.tag_selector(missing_lines, ui);
            ui.allocate_ui(STRING_RECT, |ui| {
                ui.set_min_height(STRING_HEIGHT);
                ui.label(self.words.master_line(self.show.selected_tag));
            });
            ui.separator();
            ui.add_space(BETWEEN_FIELDS);
            ui.allocate_ui(STRING_RECT, |ui| {
                ui.set_min_height(STRING_HEIGHT);
                if let Some(language) = language {
                    ui.label(
                        self.words
                            .translation_line(language, self.show.selected_tag),
                    );
                }
            });
            ui.separator();
            ui.add_space(BETWEEN_FIELDS);

            ui.label(format!("Context"));
            // primary language
            // translation
            // context
            // notes?
        });
    }

    fn show_translation(&mut self, language: Language, ui: &mut Ui) {
        self.show_string(Some(language), ui);
    }

    fn show_primary(&mut self, ui: &mut Ui) {
        self.show_string(None, ui);
    }

    fn show_export(&mut self, ui: &mut Ui) {
        // Exports
        ui.vertical(|ui| {
            let template_heading = RichText::new(fl!("export_template_heading")).heading();
            ui.label(template_heading);

            let template_description = RichText::new(fl!("export_template_description"));
            ui.label(template_description);

            ui.add_space(BETWEEN_FIELDS);

            ui.indent("template_options", |ui| {
                ui.checkbox(
                    &mut self.show.export_only_new,
                    fl!("export_template_only_new"),
                );
                ui.checkbox(
                    &mut self.show.export_existing,
                    fl!("export_template_existing"),
                );
                ui.checkbox(
                    &mut self.show.export_combine,
                    fl!("export_template_combine"),
                );
            });

            ui.add_space(BETWEEN_FIELDS);

            ui.horizontal(|ui| {
                let export_template_file = fl!("export_template_files");
                let export_template_zip = fl!("export_template_zip");

                if ui.button(export_template_file).clicked() {
                    debug!("clicked export template file(s)");
                }

                ui.add_space(BETWEEN_FIELDS);

                if ui.button(export_template_zip).clicked() {
                    debug!("clicked export template zip");
                }
            });

            // export creates a work_translation (with nothing, if new)
            //
            // export translation template for set of languages - with or without current translations
            // also full list or only changed/added since last - to zip
            //
            // export mod with selected translations - to zip
            // selected translations is everything in work_translations
            ui.label("Export selection");
        });
    }

    fn tag_selector(&mut self, missing_lines: Vec<usize>, ui: &mut Ui) {
        let tags: Vec<RichText> = {
            self.tags()
                .iter()
                .enumerate()
                .map(|(line_id, tag)| {
                    if missing_lines.contains(&line_id) {
                        // debug!("found missing line in tag selector");
                        RichText::new(format!("* {tag}")).color(MISSING_COLOR)
                    } else {
                        RichText::new(tag)
                    }
                })
                .collect()
        };

        ui.horizontal(|ui| {
            let tag_label = RichText::new(format!("{}: ", fl!("show_tag"))).heading();
            ui.label(tag_label);

            ComboBox::from_id_salt("Tag").show_index(
                ui,
                &mut self.show.selected_tag,
                tags.len(),
                |t| tags[t].clone(),
            );

            if !missing_lines.is_empty() {
                ui.add_space(BETWEEN_FIELDS);
                let missing_num = missing_lines.len().to_string();
                let missing_label =
                    RichText::new(fl!("show_missing_num", arg!("num", missing_num)))
                        .italics()
                        .color(MISSING_COLOR);
                ui.label(missing_label);
            }
        });

        // handle keyboard input - is this right to be here?

        ui.input(|i| {
            if i.key_pressed(Key::ArrowUp) {
                self.show.selected_tag = self.show.selected_tag.saturating_sub(1);
            }
            if i.key_pressed(Key::ArrowDown) {
                self.show.selected_tag =
                    self.show.selected_tag.saturating_add(1).min(tags.len() - 1);
            }
        });
        ui.separator();
    }

    pub fn master_language_name(&self) -> String {
        self.primary_language.to_string()
    }

    fn add_translation(&mut self, translation: Translation, language_type: LanguageType) {
        self.words.add_translation(&translation, language_type);
    }

    pub fn translations(&self) -> Vec<&Translation> {
        let langs = self.words.translation_languages();
        langs
            .iter()
            .filter_map(|l| self.words.translation_language(*l).map(|(trans, _t)| trans))
            .collect()
    }

    // ============ Old

    pub fn tags(&self) -> &TagList {
        &self.words.tags
    }

    // pub fn main_translations(&self) -> Vec<&Translation> {
    //     if let Some(cat) = self.categories.get(&Category::Main) {
    //         if let Some(translations) = cat.translation_in_language_type(LanguageType::External) {
    //             translations.values().collect()
    //         } else {
    //             Vec::new()
    //         }
    //     } else {
    //         Vec::new()
    //     }
    // }

    // pub fn category_translations(&self, category: &Category) -> Vec<&Translation> {
    //     if let Some(cat) = self.categories.get(category) {
    //         if let Some(translations) = cat.translation_in_language_type(LanguageType::External) {
    //             translations.values().collect()
    //         } else {
    //             Vec::new()
    //         }
    //     } else {
    //         Vec::new()
    //     }
    // }

    pub fn export_master_for_translation(&self, export_path: &str) -> Result<()> {
        let language = &self.words.master;
        let temp_path = PathBuf::from_str(export_path)?;
        let master_language = self.get_master_language();
        let language_file = format!("{}", master_language.language());
        let mut master_filepath = temp_path.join(language_file);
        master_filepath.add_extension("csv");
        if let Err(e) = write_lang_csv(
            &master_filepath,
            &self.words.tags,
            &language
                .lines()
                .iter()
                .map(|ld| ld.current_line().0)
                .collect(),
            &self.words.master_context,
        ) {
            error!(
                "unable to export {} for master language {}: {e}",
                format!("{}", master_language.language()),
                master_language.language()
            );
        }
        Ok(())
    }

    fn get_master_language(&self) -> &Translation {
        &self.words.master
    }

    pub fn external_translations_empty(&self) -> bool {
        let langs = self.words.translation_languages();
        for lang in langs {
            if let Some((_trans, t)) = self.words.translation_language(lang) {
                if t >= LanguageType::External {
                    return false;
                }
            }
        }

        true
    }

    pub fn add_translations(&mut self, translations: Vec<Translation>) {
        for trans in translations {
            self.add_translation(trans, LanguageType::External); // is that what you want?
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Location {
    Internal,
    External,
    InProgress,
}

impl Location {
    pub fn extensions(&self) -> &[&str] {
        use Location::*;
        const INT_EXTENSIONS: &[&str] = &[JSON_EXT, INTERNAL_EXT];
        const EXT_EXTENSIONS: &[&str] = &[INTERNAL_EXT];
        const TRANS_EXTENSIONS: &[&str] = &[CSV_EXT, JSON_EXT];
        match self {
            Internal => INT_EXTENSIONS,
            External => EXT_EXTENSIONS,
            InProgress => TRANS_EXTENSIONS,
        }
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Location::*;
        write!(
            f,
            "{}",
            match self {
                Internal => "internal",
                External => "external",
                InProgress => "translation",
            }
        )
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
pub enum LanguageType {
    Internal,
    InProgress,
    External,
}

// Core
// New(+minor)
// MainMod
// NewMod(+minor)

impl Hash for LanguageType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let string = format!("{}", self);
        string.hash(state)
    }
}

impl Display for LanguageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use LanguageType::*;
        write!(
            f,
            "{}",
            match self {
                // Master { .. } => "master",
                Internal => "internal",
                InProgress => "translation",
                External => "external",
            }
        )
    }
}

/// The categories of dictionary lines: Main, Achivements, and Presence.
// #[derive(Debug, PartialEq, Eq, PartialOrd, Hash, Sequence, Clone, Copy)]
// pub enum Category {
//     Main,
//     Achievements,
//     Presence,
// }

// const NUM_CATEGORIES: usize = cardinality::<Category>();

// impl Category {
//     /// return the filename tail for this category of lines
//     pub fn get_tail(&self) -> &str {
//         use Category::*;
//         match self {
//             Main => "",
//             Achievements => ACHIEVE_NAME,
//             Presence => PRESENCE_NAME,
//         }
//     }
// }

// ---------------------
// Words
//
/// The tags, context, and translations for a dictionary of lines, including the primary language and
/// its translations.
#[derive(Debug)]
struct Words {
    master: Translation,
    tags: TagList,
    core_translations: HashMap<Language, Translation>,
    work_translations: HashMap<Language, Translation>,
    master_context: ContextList,
    versions: ContentVersions,
}

impl Words {
    pub fn new(master: Translation, tags: TagList) -> Self {
        Words {
            master,
            tags,
            core_translations: HashMap::new(),
            work_translations: HashMap::new(),
            master_context: ContextList::new(),
            versions: ContentVersions::default(),
        }
    }

    fn translation_languages(&self) -> Vec<Language> {
        self.core_translations.keys().copied().collect()
    }

    // add_core update or replace
    // add_work update or replace
    fn add_translation(&mut self, translation: &Translation, language_type: LanguageType) {
        let language = translation.language();

        self.core_translations
            .entry(*language)
            .and_modify(|lang| {
                if !lang.contains_key(&language_type) {
                    lang.insert(language_type.clone(), translation.clone()); // return is None if not present
                } else {
                    error!("unable to add language {language}, {language_type}: already present");
                }
            })
            .or_insert(translation.clone());
    }

    fn translation_language(&self, language: Language) -> Option<(&Translation, LanguageType)> {
        if language == *self.master.language() {
            Some((&self.master, LanguageType::Internal))
        } else if let Some(translations) = self.core_translations.get(&language) {
            if let Some((typ, trans)) = translations.iter().last() {
                // !! I think we want the last one
                Some((trans, *typ))
            } else {
                None
            }
        } else {
            None
        }
    }

    fn number_tags(&self) -> usize {
        self.tags.len()
    }

    fn master_line(&self, string_id: usize) -> RichText {
        // should we be ready to flag modified lines?
        let (line, c_type) = self.master.line(string_id);
        let mut ret = RichText::new(line);
        ret = self.master.language().text_font(ret); // text_font(ret, self.master.language());
        match c_type {
            ContentType::InProgress => ret.color(MOD_MAIN_COLOR),
            ContentType::Master => ret,
        }
    }

    fn missing_lines(&self, language: Option<Language>) -> Vec<usize> {
        if let Some(language) = language {
            if let Some((translation, _l_type)) = self.translation_language(language) {
                return translation.missing_lines().clone();
            }
        }
        Vec::new()
    }

    fn translation_line(&self, language: Language, string_id: usize) -> RichText {
        let (line, c_type) =
            if let Some((translation, _l_type)) = self.translation_language(language) {
                // we should flag modified lines
                translation.line(string_id)
            } else {
                (" ", ContentType::Master)
            };
        let mut ret = RichText::new(line);
        ret = language.text_font(ret); // text_font(ret, &language);

        match c_type {
            ContentType::InProgress => ret.color(MOD_TRANS_COLOR),
            ContentType::Master => ret,
        }
    }
}

// -------------------
// Location Paths
//
#[derive(Debug, Default)]
struct LocationPaths(HashMap<Location, PathBuf>);

impl LocationPaths {
    pub fn new(base_loc: Location, base_path: PathBuf) -> Self {
        LocationPaths(HashMap::from([(base_loc, base_path)]))
    }

    pub fn get_path(&self, location: Location) -> Option<(Location, &Path)> {
        self.0.get(&location).map(|p| (location, p.as_path()))
    }
}

// -------------------
// ShowState
//
#[derive(Debug, Default, Clone)]
struct ShowState {
    state: DictionaryState,
    selected_translation: Language, // !! needs to be Option, as there might be no translations
    selected_tag: usize,
    export_only_new: bool,
    export_existing: bool,
    export_combine: bool,
    translation_list: Vec<(usize, String)>, // ? one for core, one for work?
}

impl ShowState {
    pub fn default_not(not_selected: Language) -> Self {
        let translation_list = Languages::list_not(not_selected);

        let (i, _l) = translation_list
            .first()
            .expect("no translation found in ShowState default_not");
        let selected_translation =
            Languages::from_index(*i).expect("index error for Language in ShowState default_not");

        ShowState {
            state: DictionaryState::default(),
            selected_translation,
            selected_tag: 0,
            translation_list,
            ..Default::default()
        }
    }

    /// returns true if a new language is selected
    pub fn select_language(&mut self, ui: &mut Ui) -> bool {
        let index = self.selected_translation.language_index();
        let languages = &self.translation_list;
        let mut selected = languages
            .iter()
            .position(|(i, _l)| index == *i)
            .expect("cannot find language in ShowState select language");
        ComboBox::from_id_salt("Lang").show_index(ui, &mut selected, languages.len(), |l| {
            languages[l].1.as_str()
        });
        let new_index = languages[selected].0;
        if let Some(new_lang) = Languages::from_index(new_index) {
            if new_lang != self.selected_translation {
                info!("selected new language: {new_lang}");
                self.selected_translation = new_lang;
                return true;
            }
        }
        false
    }
}

// =====================
// DictionaryState

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum DictionaryState {
    #[default]
    Overview,
    Primary,
    Translation(Language),
    Export,
}

impl Display for DictionaryState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use DictionaryState::*;

        write!(
            f,
            "{}",
            match self {
                Overview => fl!("show_overview"),
                Primary => fl!("primary"),
                Translation(lang) => format!("{} {}", fl!("translation"), lang.name()),
                Export => fl!("show_export"),
            }
        )
    }
}

// -------------------
// DictionaryVersion
//
#[derive(Debug, Clone)]
struct DictionaryVersion {
    version: SemVer,
    note: String,
}

impl DictionaryVersion {
    /// A new major version sets minor and patch to 0, and includes a new note.
    pub fn new_major_version(&self, note: String) -> Self {
        DictionaryVersion {
            version: SemVer {
                major: self.version.major + 1,
                ..Default::default()
            },
            note,
        }
    }

    /// A new minor version retains the major, and sets the patch to 0, and includes a new note.
    pub fn new_minor_version(&self, note: String) -> Self {
        DictionaryVersion {
            version: SemVer {
                major: self.version.major,
                minor: self.version.minor + 1,
                ..Default::default()
            },
            note,
        }
    }

    /// A new patch version retains the major and minor, and includes a new note.
    pub fn new_patch_version(&self, note: String) -> Self {
        DictionaryVersion {
            version: SemVer {
                major: self.version.major,
                minor: self.version.minor,
                patch: self.version.patch + 1,
                ..Default::default()
            },
            note,
        }
    }
}

impl Default for DictionaryVersion {
    fn default() -> Self {
        DictionaryVersion {
            version: SemVer {
                major: 1,
                ..Default::default()
            },
            note: fl!("initial_version").to_string(),
        }
    }
}

impl Display for DictionaryVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.version, self.note)
    }
}

// -------------------
// Versions
//
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Sequence)]
pub enum ContentType {
    Master,
    InProgress,
    // Core,
    // CoreUpdate,
    // Mod,
    // ModUpdate,
}

impl Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ContentType::*;

        write!(
            f,
            "{}",
            match self {
                Master => fl!("version_core"),
                InProgress => fl!("version_wip"),
                // Core => "Core",
                // CoreUpdate => "Updated Core",
                // Mod => "Mod",
                // ModUpdate => "Updated Mod",
            }
        )
    }
}

// -------------------
// VersionHistory
//
#[derive(Debug)]
struct ContentVersions {
    current: BTreeMap<ContentType, DictionaryVersion>, // can we calculate these? I think not
}

impl ContentVersions {
    pub fn current_version(&self, version: ContentType) -> &DictionaryVersion {
        if let Some(ver) = self.current.get(&version) {
            return ver;
        } else {
            error!("no version found for {version}");
            panic!("no version found for {version}");
        }
    }

    /// Loaded in an update to some data
    pub fn incremental_version(&mut self, version: ContentType, note: String) {
        use ContentType::*;

        // !! Bur surely this should update the other versions as well?

        self.current.insert(
            version,
            match version {
                Master => self.current_version(version).new_minor_version(note),
                InProgress => self.current_version(version).new_patch_version(note),
                // Core => self.current_version(version).new_major_version(note),
                // Mod => self.current_version(version).new_minor_version(note),
                // CoreUpdate | ModUpdate => self.current_version(version).new_patch_version(note),
            },
        );
    }

    /// Everything goes up to a new version
    pub fn major_version(&mut self, note: String) {
        let new_version = self
            .current_version(ContentType::Master)
            .new_major_version(note);
        for cont in all::<ContentType>() {
            self.current.insert(cont, new_version.clone());
        }
    }
}

impl Default for ContentVersions {
    fn default() -> Self {
        let first_ver = DictionaryVersion::default();

        ContentVersions {
            current: all::<ContentType>()
                .map(|ver| (ver, first_ver.clone()))
                .collect(),
        }
    }
}

impl Display for ContentVersions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.current
                .iter()
                .map(|(v, c)| { format!("{v}: {c}") }) // !! not translated
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateReplace {
    Update,
    Replace,
}
