use std::collections::HashMap;

use eframe::egui::RichText;
use log::warn;

use crate::{
    dictionary::{ContentType, ContentVersions, ContextList, LanguageType, TagList},
    display::{MOD_MAIN_COLOR, MOD_TRANS_COLOR},
    languages::{CustomLanguage, UiLanguage},
    translation::Translation,
};

// ---------------------
// Words
//
/// The tags, context, and translations for a dictionary of lines, including the primary language and
/// its translations.
#[derive(Debug, Clone)]
pub struct Words {
    master: Translation,
    tags: TagList, // !! changes here imply changes to translations and context
    core_translations: HashMap<UiLanguage, Translation>,
    work_translations: HashMap<UiLanguage, Translation>,
    master_context: ContextList,
    versions: ContentVersions,
}

impl Words {
    pub fn new(master: Translation, tags: TagList) -> Self {
        let size = tags.len();
        Words {
            master,
            tags,
            core_translations: HashMap::new(),
            work_translations: HashMap::new(),
            master_context: vec![String::new(); size],
            versions: ContentVersions::default(),
        }
    }

    pub fn master_external_language(&self) -> &CustomLanguage {
        self.master.external_language()
    }

    pub fn translation_languages(&self) -> Vec<UiLanguage> {
        self.core_translations.keys().copied().collect()
    }

    pub fn translation_external_languages(&self) -> Vec<CustomLanguage> {
        self.core_translations
            .iter()
            .map(|(_k, trans)| trans.external_language().clone())
            .collect()
    }

    pub fn add_core_translation(&mut self, translation: &Translation) {
        let language = translation.language();

        if self
            .core_translations
            .insert(*language, translation.clone())
            .is_some()
        {
            warn!("replacing existing {language} translation, when adding translation");
        }
    }

    fn add_work_translation(&mut self, translation: &Translation) {
        let language = translation.language();

        if self
            .core_translations
            .insert(*language, translation.clone())
            .is_some()
        {
            warn!("replacing existing {language} translation, when adding translation");
        }
    }

    pub fn tags(&self) -> &TagList {
        &self.tags
    }

    pub fn versions(&self) -> &ContentVersions {
        &self.versions
    }

    pub fn master_translation(&self) -> &Translation {
        &self.master
    }

    pub fn master_context(&self) -> &ContextList {
        &self.master_context
    }

    pub fn translation_language(
        &self,
        language: UiLanguage,
    ) -> Option<(&Translation, LanguageType)> {
        if language == *self.master.language() {
            Some((&self.master, LanguageType::Internal))
        } else if let Some(translation) = self.work_translations.get(&language) {
            Some((translation, LanguageType::InProgress))
        } else if let Some(translation) = self.core_translations.get(&language) {
            // !! I think we want the last one
            Some((translation, LanguageType::Internal))
        } else {
            None
        }
    }

    fn translation_present(&self, language: UiLanguage) -> bool {
        *self.master.language() == language
            || self.work_translations.contains_key(&language)
            || self.core_translations.contains_key(&language)
    }

    /// Warning: this assumes the language is present and asserts if that is not the case
    fn translation_language_editable(
        &mut self,
        language: UiLanguage,
    ) -> (&mut Translation, LanguageType) {
        if language == *self.master.language() {
            (&mut self.master, LanguageType::Internal)
        } else if self.work_translations.contains_key(&language) {
            (
                &mut *self.work_translations.get_mut(&language).unwrap(),
                LanguageType::InProgress,
            )
        } else {
            // this must be where the language is, or we assert
            assert!(self.core_translations.contains_key(&language));
            (
                &mut *self.core_translations.get_mut(&language).unwrap(),
                LanguageType::Internal,
            )
        }
    }

    pub fn number_tags(&self) -> usize {
        self.tags.len()
    }

    pub fn master_line(&self, string_id: usize) -> RichText {
        // should we be ready to flag modified lines?
        let (line, c_type) = self.master.line(string_id);
        let mut ret = RichText::new(line);
        ret = self.master.language().text_font(ret); // text_font(ret, self.master.language());
        match c_type {
            ContentType::InProgress => ret.color(MOD_MAIN_COLOR),
            ContentType::Master => ret,
        }
    }

    // ?? do we need this
    fn context_line(&self, string_id: usize) -> RichText {
        if let Some(context) = self.master_context.get(string_id) {
            self.master.language().text_font(RichText::new(context)) // is this the right language?
        } else {
            RichText::new("")
        }
    }

    pub fn context_editable(&mut self, string_id: usize) -> Option<(&mut String, &UiLanguage)> {
        if string_id < self.master_context.len() {
            Some((&mut self.master_context[string_id], self.master.language()))
        } else {
            None
        }
    }

    pub fn missing_lines(&self, language: Option<UiLanguage>) -> Vec<usize> {
        if let Some(language) = language
            && let Some((translation, _l_type)) = self.translation_language(language)
        {
            return translation.missing_lines().clone();
        }
        Vec::new()
    }

    pub fn translation_line(&self, language: UiLanguage, string_id: usize) -> RichText {
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

    fn translation_notes(&self, language: UiLanguage, string_id: usize) -> RichText {
        let notes = if let Some((translation, _l_type)) = self.translation_language(language) {
            // we should flag modified lines
            translation.notes(string_id)
        } else {
            " "
        };
        self.master.language().text_font(RichText::new(notes)) // this is from the master language
    }

    /// Warning, this assumes the language and string_id are valid, and will assert, otherwise
    pub fn translation_notes_editable(
        &mut self,
        language: UiLanguage,
        string_id: usize,
    ) -> &mut String {
        assert!(self.translation_present(language));
        let (translation, _l_type) = self.translation_language_editable(language); // will assert if no language found
        translation.notes_editable(string_id)
    }
}
