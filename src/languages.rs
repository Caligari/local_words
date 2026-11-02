use std::{fmt::Display, sync::Arc};

use eframe::egui::{
    ComboBox, FontData, FontDefinitions, FontFamily, FontSelection, RichText, TextStyle, Ui,
};
use enum_iterator::{Sequence, all};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};

use crate::{
    dictionary::INTERNAL_EXT, loader::LoadFormat, localize::fl, writer::TranslationStructureVersion,
};

pub const LANGUAGE_DEF_FILENAME: &str = "language.def";

type LanguageData = [&'static str; 3];
const LANGUAGE_DATA_INTERNAL: usize = 0;
const LANGUAGE_DATA_EXTERNAL: usize = 1;
const LANGUAGE_DATA_FONT_FAMILY: usize = 2;

const ENGLISH: LanguageData = ["english", "eng", FONT_BASE]; // internal, external
const FRENCH: LanguageData = ["french", "fre", FONT_BASE]; // internal, external
const GERMAN: LanguageData = ["german", "ger", FONT_BASE];
const CHINESE: LanguageData = ["simplified-chinese", "zho-CN", FONT_CHINESE];
const PORTUGUESE: LanguageData = ["portuguese-br", "por-BR", FONT_BASE];
const CZECH: LanguageData = ["czech", "ces", FONT_BASE];
const DUTCH: LanguageData = ["dutch", "dut", FONT_BASE];
const ITALIAN: LanguageData = ["italian", "ita", FONT_BASE];
const POLISH: LanguageData = ["polish", "pol", FONT_BASE]; // need a font
const RUSSIAN: LanguageData = ["russian", "rus", FONT_BASE]; // need a font
const SPANISH: LanguageData = ["spanish", "spa", FONT_BASE];
// !! when updating this, add to Language::language_data() and Language enum

// "../assets/fonts/Noto_Serif/static/NotoSerif-Regular.ttf"
const FONT_BASE_FILE: &str = "../assets/fonts/Noto_Sans/static/NotoSans-Regular.ttf"; // not used in const
pub const FONT_BASE: &str = "text";
// "../assets/fonts/Noto_Serif_SC/static/NotoSerifSC-Regular.ttf"
const FONT_CHINESE_FILE: &str = "../assets/fonts/Noto_Sans_SC/static/NotoSansSC-Regular.ttf"; // not used in const
pub const FONT_CHINESE: &str = "chinese";
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

        // ?? font_id as well?
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
    Czech,
    Dutch,
    Italian,
    Polish,
    Russian,
    Spanish,
}

impl Language {
    fn language_data(&self) -> &LanguageData {
        // !!!! This must match the Language enum order above
        const KNOWN_LANGUAGES: &[LanguageData] = &[
            ENGLISH, FRENCH, GERMAN, CHINESE, PORTUGUESE, CZECH, DUTCH, ITALIAN, POLISH, RUSSIAN,
            SPANISH,
        ];
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
            Czech => fl!("czech"),
            Dutch => fl!("dutch"),
            Italian => fl!("italian"),
            Polish => fl!("polish"),
            Russian => fl!("russian"),
            Spanish => fl!("spanish"),
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

    pub fn internal_main_file_name(&self, format: TranslationStructureVersion) -> String {
        match format {
            TranslationStructureVersion::Version0 => {
                format!("{}.{}", self.internal(), INTERNAL_EXT)
            }
            TranslationStructureVersion::Version1 => {
                format!("{}/00-base.{}", self.internal(), INTERNAL_EXT)
            }
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

    pub fn font_selection(&self) -> FontSelection {
        TextStyle::Name(self.language_data()[LANGUAGE_DATA_FONT_FAMILY].into()).into()
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

// ========================
// External Languages
// ========================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExternalLanguage {
    Standard(String),
    Asian(String),
    Cyrillic(String),
}

impl ExternalLanguage {
    pub fn from_def(buffer: &[u8]) -> Self {
        const DEF_NAME_FIELD: &str = "displayNameEnglish";
        const DEF_FONT_FIELD: &str = "customFont";
        info!("loading external language from def data");
        let load_format = LoadFormat::Vrt;
        match load_format.load(buffer) {
            Ok(terms) => {
                let name = if let Some(name) = terms
                    .iter()
                    .find(|t| t.0 == DEF_NAME_FIELD)
                    .map(|t| t.1.clone())
                {
                    name
                } else {
                    "".to_string()
                };
                match terms
                    .iter()
                    .find(|t| t.0 == DEF_FONT_FIELD)
                    .map(|t| t.1.clone())
                {
                    Some(font) => match font.as_str() {
                        "NotoSans-CJK" => {
                            info!("  found external language {name} with Asian font");
                            ExternalLanguage::Asian(name)
                        }
                        "NotoSans-Cyrillic" => {
                            info!("  found external language {name} with Cyrillic font");
                            ExternalLanguage::Cyrillic(name)
                        }
                        _ => {
                            warn!("unexpected font name in language def: {font}");
                            ExternalLanguage::Standard(name)
                        }
                    },
                    None => {
                        info!("  found external language <{name}> with Standard font");
                        ExternalLanguage::Standard(name)
                    }
                }
            }

            Err(e) => {
                error!("Unable to load language def: {e}");
                ExternalLanguage::Standard("".to_string())
            }
        }
    }

    pub fn reset_name(&mut self, name: &str) {
        use ExternalLanguage::*;
        match self {
            Standard(n) => *n = name.to_string(),
            Asian(n) => *n = name.to_string(),
            Cyrillic(n) => *n = name.to_string(),
        }

        info!("changed external language name to {name}");
    }

    pub fn has_name(&self) -> bool {
        use ExternalLanguage::*;
        match self {
            Standard(n) => !n.is_empty(),
            Asian(n) => !n.is_empty(),
            Cyrillic(n) => !n.is_empty(),
        }
    }

    pub fn name(&self) -> &str {
        use ExternalLanguage::*;
        match self {
            Standard(n) => n.as_str(),
            Asian(n) => n.as_str(),
            Cyrillic(n) => n.as_str(),
        }
    }

    pub fn external_define(&self) -> String {
        use ExternalLanguage::*;

        let (name, font) = match self {
            Standard(n) => (n, ""),
            Asian(n) => (n, "NotoSans-CJK"),
            Cyrillic(n) => (n, "NotoSans-Cyrillic"),
        };

        format!(
            r#"moLanguageDef
        {{
	displayNameEnglish "{}"

	# If your language requires a special font, this is where you can specify it.
	#
	# If you need CJK glyphs, use "NotoSans-CJK".
	# If you need cyrillic, use "NotoSans-Cyrillic"
	#
	# Otherwise, just leave it blank.

	customFont "{}"
        }}"#,
            name, font
        )
    }
}

impl Default for ExternalLanguage {
    fn default() -> Self {
        // info!("using default external language: <no name> with Standard font");
        ExternalLanguage::Standard("".to_string())
    }
}

// =========================
// Tests
// =========================

#[cfg(test)]
mod tests {
    use crate::languages::ExternalLanguage::{self, Asian, Cyrillic, Standard};

    #[test]
    fn export_import_external_def() {
        export_import(Asian("happy".to_string()));
        export_import(Cyrillic("snappy".to_string()));
        export_import(Standard("frog".to_string()));
        export_import(Asian("".to_string()));
        export_import(Cyrillic("".to_string()));
        export_import(Standard("".to_string()));
        export_import(ExternalLanguage::default());
    }

    fn export_import(orig: ExternalLanguage) {
        let output = orig.external_define();
        let buffer = output.as_bytes();
        let inp = ExternalLanguage::from_def(buffer);
        assert_eq!(orig, inp);
        println!("completed test with {orig:?}");
    }

    #[test]
    fn change_name() {
        let orig = Asian("happy".to_string());
        let mut other = orig.clone();
        assert_eq!(orig, other);

        println!("was: {other:?}");
        other.reset_name("sad");
        println!("now: {other:?}");
        assert_ne!(orig, other);
    }
}
