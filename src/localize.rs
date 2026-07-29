use std::sync::LazyLock;

use i18n_embed::DesktopLanguageRequester;
use i18n_embed::fluent::{FluentLanguageLoader, fluent_language_loader};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "assets/text"]
struct Localizations;

#[allow(dead_code)]
pub static LANGUAGE_LOADER: LazyLock<FluentLanguageLoader> = LazyLock::new(|| {
    let loader = fluent_language_loader!();
    let languages = DesktopLanguageRequester::requested_languages();
    i18n_embed::select(&loader, &Localizations, &languages).unwrap();
    loader.set_use_isolating(false);
    loader
});

#[allow(unused_macros)]
macro_rules! fl {
    ($message_id:literal) => {
        ::i18n_embed_fl::fl!($crate::localize::LANGUAGE_LOADER, $message_id)
    };
    ($message_id:literal, $($args:expr),*) => {
        ::i18n_embed_fl::fl!($crate::localize::LANGUAGE_LOADER, $message_id, $($args), *)
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
