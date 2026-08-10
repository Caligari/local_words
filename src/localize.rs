use std::collections::HashMap;
use std::sync::LazyLock;

use eframe::egui::{FontFamily, RichText};
use fluent_templates::LanguageIdentifier;
use i18n_embed::fluent::{FluentLanguageLoader, fluent_language_loader};
use i18n_embed::{DesktopLanguageRequester, LanguageLoader};
use rust_embed::RustEmbed;

use crate::languages::FONT_BASE;

#[derive(RustEmbed)]
#[folder = "assets/text"]
struct Localizations;

#[derive(Debug, Clone, Copy)]
pub struct LangInfo {
    name: &'static str,
    font: &'static str,
}
impl From<(&'static str, &'static str)> for LangInfo {
    fn from(value: (&'static str, &'static str)) -> Self {
        LangInfo {
            name: value.0,
            font: value.1,
        }
    }
}
pub static LANGUAGE_NAMES: LazyLock<HashMap<&str, LangInfo>> =
    LazyLock::new(|| HashMap::from([("en", ("English", FONT_BASE).into())]));

pub fn language_name(id: &str) -> RichText {
    let names = &*LANGUAGE_NAMES;
    let name = if let Some(name) = names.get(id) {
        *name
    } else {
        LangInfo {
            name: "unknown",
            font: FONT_BASE,
        }
    };
    RichText::new(name.name).family(FontFamily::Name(name.font.into()))
}

pub static mut CURRENT_LANGUAGES: LazyLock<LanguagesList> = LazyLock::new(LanguagesList::new);

#[allow(dead_code)]
pub static mut LANGUAGE_LOADER: LazyLock<MyLanguageLoader> =
    LazyLock::new(MyLanguageLoader::new);

pub static LANGUAGES_LIST: LazyLock<Vec<LanguageIdentifier>> = LazyLock::new(|| {
    let loader = fluent_language_loader!();
    loader.load_available_languages(&Localizations).unwrap();
    loader.available_languages(&Localizations).unwrap()
});

pub static FALLBACK_LANGUAGE: LazyLock<LanguageIdentifier> = LazyLock::new(|| {
    let loader = fluent_language_loader!();
    loader.fallback_language().clone()
});

#[allow(unused_macros)]
macro_rules! fl {
    ($message_id:literal) => {
        ::i18n_embed_fl::fl!(unsafe{
            #[allow(static_mut_refs)]
            $crate::localize::LANGUAGE_LOADER.loader()}, $message_id)
    };
    ($message_id:literal, $($args:expr),*) => {
        ::i18n_embed_fl::fl!(unsafe{
            #[allow(static_mut_refs)]
            $crate::localize::LANGUAGE_LOADER.loader()}, $message_id, $($args), *)
    };
}

#[allow(unused_macros)]
macro_rules! arg {
    ($arg_name:literal, $arg:ident) => {{
        let mut hash = ::std::collections::HashMap::new();
        hash.insert($arg_name, $arg.as_str());
        hash
    }};
    ($arg_name:literal, $arg:literal) => {{
        let mut hash = ::std::collections::HashMap::new();
        hash.insert($arg_name, $arg);
        hash
    }};
    ($arg1_name:literal, $arg1:ident, $arg2_name:literal, $arg2:ident) => {{
        let mut hash = ::std::collections::HashMap::new();
        hash.insert($arg1_name, $arg1.as_str());
        hash.insert($arg2_name, $arg2.as_str());
        hash
    }};
    ($arg1_name:literal, $arg1:literal, $arg2_name:literal, $arg2:literal) => {{
        let mut hash = ::std::collections::HashMap::new();
        hash.insert($arg1_name, $arg1);
        hash.insert($arg2_name, $arg2);
        hash
    }};
}

#[allow(unused_imports)]
pub(crate) use {arg, fl};

// ======================
// LanguagesList
//
pub struct LanguagesList {
    languages: Vec<LanguageIdentifier>,
}

impl LanguagesList {
    pub fn new() -> Self {
        LanguagesList {
            languages: DesktopLanguageRequester::requested_languages(),
        }
    }

    pub fn languages(&self) -> Vec<LanguageIdentifier> {
        self.languages.clone()
    }

    pub fn set_language(&mut self, lang: LanguageIdentifier) {
        let new_first = if let Some(cur_pos) = self.languages.iter().position(|li| *li == lang) {
            self.languages.remove(cur_pos)
        } else {
            lang
        };
        self.languages.insert(0, new_first);
        unsafe {
            #[allow(static_mut_refs)]
            LANGUAGE_LOADER.refresh();
        }
    }
}

// ==================
// MyLanguageLoader
//
pub struct MyLanguageLoader {
    loader: FluentLanguageLoader,
}

impl MyLanguageLoader {
    fn new() -> Self {
        MyLanguageLoader {
            loader: MyLanguageLoader::new_loader(),
        }
    }

    pub fn loader(&self) -> &FluentLanguageLoader {
        &self.loader
    }

    fn new_loader() -> FluentLanguageLoader {
        let loader = fluent_language_loader!();
        let languages = unsafe {
            #[allow(static_mut_refs)]
            CURRENT_LANGUAGES.languages()
        };
        i18n_embed::select(&loader, &Localizations, &languages).unwrap();
        loader.set_use_isolating(false);
        loader
    }

    fn refresh(&mut self) {
        self.loader = MyLanguageLoader::new_loader();
    }

    pub fn current_language(&self) -> LanguageIdentifier {
        self.loader.current_language()
    }
}
