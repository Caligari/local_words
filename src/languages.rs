use std::{fmt::Display, sync::Arc};

use eframe::egui::{ComboBox, FontData, FontDefinitions, FontFamily, RichText, Ui};
use enum_iterator::{Sequence, all};
use log::warn;

use crate::{dictionary::INTERNAL_EXT, localize::fl, writer::FormatVersion};

type LanguageData = [&'static str; 3];
const LANGUAGE_DATA_INTERNAL: usize = 0;
const LANGUAGE_DATA_EXTERNAL: usize = 1;
const LANGUAGE_DATA_FONT_FAMILY: usize = 2;

const ENGLISH: LanguageData = ["english", "eng", FONT_BASE]; // internal, external
const FRENCH: LanguageData = ["french", "fre", FONT_BASE]; // internal, external
const GERMAN: LanguageData = ["german", "ger", FONT_BASE];
const CHINESE: LanguageData = ["chinese", "zho-CN", FONT_CHINESE];
const PORTUGUESE: LanguageData = ["portuguese-br", "por-BR", FONT_BASE];
// !! when updating this, add to Language::language_data() and Language enum

// "../assets/fonts/Noto_Serif/static/NotoSerif-Regular.ttf"
const FONT_BASE_FILE: &str = "../assets/fonts/Noto_Sans/static/NotoSans-Regular.ttf"; // not used in const
const FONT_BASE: &str = "text";
// "../assets/fonts/Noto_Serif_SC/static/NotoSerifSC-Regular.ttf"
const FONT_CHINESE_FILE: &str = "../assets/fonts/Noto_Sans_SC/static/NotoSansSC-Regular.ttf"; // not used in const
const FONT_CHINESE: &str = "chinese";
// !! updating the font files requires updating Language::add_language_fonts

pub struct Languages;
impl Languages {
    pub fn all() -> Vec<Language> {
        all::<Language>().collect::<Vec<_>>()
    }

    pub fn find_language(name: &str) -> Option<Language> {
        for l in all::<Language>().collect::<Vec<_>>() {
            if l.name() == name {
                return Some(l);
            }
        }
        warn!("Language {name} not found in internal language list!");
        None
    }

    pub fn find_internal(name: &str) -> Option<Language> {
        for l in all::<Language>().collect::<Vec<_>>() {
            if l.internal() == name {
                return Some(l);
            }
        }
        warn!("Language {name} not found in internal language list!");
        None
    }

    pub fn from_index(index: usize) -> Option<Language> {
        let langs = all::<Language>().collect::<Vec<_>>();
        if index < langs.len() {
            Some(langs[index])
        } else {
            warn!("attempt to select out of bounds language ({index})!");
            None
        }
    }

    pub fn name_list() -> Vec<String> {
        all::<Language>().map(|l| l.name()).collect()
    }

    /// returns localized string and language index, ignoring the passed Language
    pub fn list_not(not: Language) -> Vec<(usize, String)> {
        let mut langs = all::<Language>()
            .filter_map(|l| {
                if l != not {
                    Some((l.language_index(), l.name()))
                } else {
                    None
                }
            })
            .collect::<Vec<(usize, String)>>();
        langs.sort_by(|a, b| a.1.cmp(&b.1)); // order based on translated names
        langs
    }

    pub fn add_language_fonts(fonts: &mut FontDefinitions) {
        // font data
        fonts.font_data.insert(
            FONT_BASE.to_string(),
            Arc::new(FontData::from_static(include_bytes!(
                "../assets/fonts/Noto_Sans/static/NotoSans-Regular.ttf"
            ))),
        );
        fonts.font_data.insert(
            FONT_CHINESE.to_string(),
            Arc::new(FontData::from_static(include_bytes!(
                "../assets/fonts/Noto_Sans_SC/static/NotoSansSC-Regular.ttf"
            ))),
        );

        // families
        fonts
            .families
            .entry(FontFamily::Name(FONT_BASE.into()))
            .or_default()
            .insert(0, FONT_BASE.to_owned());
        fonts
            .families
            .entry(FontFamily::Name(FONT_CHINESE.into()))
            .or_default()
            .insert(0, FONT_CHINESE.to_owned());
    }
}

#[derive(Debug, Sequence, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum Language {
    #[default]
    English,
    French,
    German,
    Chinese,
    PortugueseBr,
}

impl Language {
    fn language_data(&self) -> &LanguageData {
        // !!!! This must match the Language enum order above
        const KNOWN_LANGUAGES: &[LanguageData] = &[ENGLISH, FRENCH, GERMAN, CHINESE, PORTUGUESE];
        &KNOWN_LANGUAGES[self.language_index()]
    }

    pub fn name(&self) -> String {
        use Language::*;
        match self {
            English => fl!("english"),
            French => fl!("french"),
            German => fl!("german"),
            Chinese => fl!("chinese"),
            PortugueseBr => fl!("portuguese-br"),
        }
    }

    pub fn internal(&self) -> &'static str {
        self.language_data()[LANGUAGE_DATA_INTERNAL]
    }

    pub fn external(&self) -> &'static str {
        self.language_data()[LANGUAGE_DATA_EXTERNAL]
    }

    pub fn external_file_name(&self, master_name: &str) -> String {
        format!("{master_name}_{}", self.external(),)
    }

    pub fn internal_main_file_name(&self, format: FormatVersion) -> String {
        match format {
            FormatVersion::Version0 => format!("{}.{}", self.internal(), INTERNAL_EXT),
            FormatVersion::Version1 => format!("{}/00-base.{}", self.internal(), INTERNAL_EXT),
        }
    }

    pub fn internal_category_file_name(&self) -> String {
        format!("{}.{}", self.internal(), INTERNAL_EXT)
    }

    pub fn language_index(&self) -> usize {
        *self as usize
    }

    pub fn text_font(&self, text: RichText) -> RichText {
        text.family(FontFamily::Name(
            self.language_data()[LANGUAGE_DATA_FONT_FAMILY].into(),
        ))
    }
}

impl Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

// ========================
pub fn select_language(selected: &mut usize, ui: &mut Ui) {
    let languages = Languages::name_list();
    ComboBox::from_id_salt("Lang")
        .show_index(ui, selected, languages.len(), |l| languages[l].clone());
}
