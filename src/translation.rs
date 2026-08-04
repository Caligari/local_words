use std::{
    collections::{HashMap, VecDeque},
    hash::Hash,
    path::{Path, PathBuf},
};

use anyhow::{Result, anyhow};
// use changecase::ChangeCase;
use log::{debug, error, info, warn};

use crate::{
    dictionary::{ContentType, ContextList, Location, TagList},
    languages::Language,
    loader::{Loaded, LoaderOld},
    localize::{arg, fl},
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LineData {
    main_line: String,
    modification: Option<String>,
}

impl LineData {
    pub fn new(line: String) -> Self {
        LineData {
            main_line: line,
            modification: None,
        }
    }

    pub fn replace_modified(&mut self, opt_line: Option<String>) {
        self.modification = if let Some(line) = opt_line {
            Some(line)
        } else {
            None
        }
    }

    pub fn base_line(&self) -> &str {
        &self.main_line
    }

    pub fn current_line(&self) -> (&str, ContentType) {
        if let Some(mod_line) = self.modification.as_ref() {
            (mod_line, ContentType::InProgress)
        } else {
            (&self.main_line, ContentType::Master)
        }
    }

    /// This reports on the current line
    pub fn is_empty(&self) -> bool {
        if let Some(mod_line) = self.modification.as_ref() {
            mod_line.is_empty()
        } else {
            self.main_line.is_empty()
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Translation {
    language: Language,
    // source: Location,
    lines: Vec<LineData>,
    extra_lines: HashMap<String, (String, Option<String>)>, // lines which do not match master tags list
    missing: Vec<usize>,                                    // which lines are blank?
}

impl Hash for Translation {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.language.hash(state)
    }
}

impl Translation {
    pub fn from_loaded(loaded: &Loaded, language: Language, tags: &TagList) -> Self {
        info!("creating translation for {language} from loaded data");
        let mut missing = Vec::new();
        let extra_lines = loaded.all_unknowns();

        let lines = tags
            .iter()
            .enumerate()
            .fold(Vec::new(), |mut acc, (i, tag)| {
                let line = if let Some(line) = loaded.get_line(tag) {
                    line.to_string()
                } else {
                    missing.push(i);
                    String::new()
                };
                acc.push(LineData::new(line));
                acc
            });

        if tags.len() > lines.len() {
            error!(
                "language {} has {} extra strings",
                language,
                tags.len() - lines.len(),
            )
        }

        info!(
            "language {}: Found {} tags (missing {} tags), with {} words",
            language,
            tags.len() - missing.len(),
            missing.len(),
            loaded.words()
        );

        Translation {
            language,
            // source: location,
            lines,
            extra_lines,
            missing,
        }
    }

    // pub fn from_loader(
    //     language: Language,
    //     loader: &mut LoaderOld,
    //     tags: &TagList,
    //     _location: Location,
    // ) -> Self {
    //     let mut missing = Vec::new();
    //     let (lines, _context) =
    //         tags.iter()
    //             .enumerate()
    //             .fold((Vec::new(), Vec::new()), |mut acc, (i, tag)| {
    //                 let con = if let Some(context) = loader.get_context(tag) {
    //                     context.to_string()
    //                 } else {
    //                     "".to_string()
    //                 };
    //                 let line = if let Some((line, _con)) = loader.remove_line(tag) {
    //                     line.to_string()
    //                 } else {
    //                     warn!("language {language} does not include string for '{tag}'");
    //                     missing.push(i); // note missing tag
    //                     String::new()
    //                 };
    //                 acc.0.push(LineData::new(line));
    //                 acc.1.push(con);
    //                 acc
    //             });
    //     if (tags.len() - lines.len()) > 0 {
    //         error!(
    //             "language {} has {} extra strings:\n  {}",
    //             language,
    //             loader.len(),
    //             loader.tags().join("\n  "),
    //         )
    //     } // !! improve this
    //     info!(
    //         "language {}: Found {} tags (missing {} tags), with {} words",
    //         language,
    //         tags.len() - missing.len(),
    //         missing.len(),
    //         loader.words()
    //     );
    //     let extra_lines = loader.leftovers();

    //     Translation {
    //         language,
    //         // source: location,
    //         lines,
    //         extra_lines,
    //         missing,
    //     }
    // }

    // pub fn new(
    //     language: Language,
    //     source_info: Option<(Location, &Path)>,
    //     in_tags: Option<&TagList>,
    // ) -> Result<(Self, Option<(TagList, ContextList)>)> {
    //     if let Some((location, input_path)) = source_info {
    //         let have_tags = in_tags.is_some();
    //         // let mut loader = get_loader(language, location, input_path)?;
    //         let mut loaders = get_loaders(language, location, input_path)?;
    //         let (mut trans, mut tags, mut context) = if let Some(mut loader) = loaders.pop_front() {
    //             let tags = if let Some(tags) = in_tags {
    //                 &tags // is this a good idea?
    //             } else {
    //                 &loader.tags()
    //             };
    //             let mut missing = Vec::new();
    //             let (lines, context) =
    //                 tags.iter()
    //                     .enumerate()
    //                     .fold((Vec::new(), Vec::new()), |mut acc, (i, tag)| {
    //                         let con = if let Some(context) = loader.get_context(tag) {
    //                             context.to_string()
    //                         } else {
    //                             "".to_string()
    //                         };
    //                         let line = if let Some((line, _con)) = loader.remove_line(tag) {
    //                             line.to_string()
    //                         } else {
    //                             warn!(
    //                                 "{} language {} does not include string for '{tag}'",
    //                                 location, language
    //                             );
    //                             missing.push(i); // note missing tag
    //                             String::new()
    //                         };
    //                         acc.0.push(LineData::new(line));
    //                         acc.1.push(con);
    //                         acc
    //                     });
    //             if (tags.len() - lines.len()) > 0 {
    //                 error!(
    //                     "{} language {} has {} extra strings:\n  {}",
    //                     location,
    //                     language,
    //                     loader.len(),
    //                     loader.tags().join("\n  "),
    //                 )
    //             } // !! improve this
    //             info!(
    //                 "{} language {}: Found {} tags (missing {} tags), with {} words",
    //                 location,
    //                 language,
    //                 if !have_tags {
    //                     tags.len()
    //                 } else {
    //                     tags.len() - missing.len()
    //                 },
    //                 missing.len(),
    //                 loader.words()
    //             );
    //             let extra_lines = loader.leftovers();

    //             (
    //                 Translation {
    //                     language,
    //                     // source: location,
    //                     lines,
    //                     extra_lines,
    //                     missing,
    //                 },
    //                 tags.clone(),
    //                 context.clone(),
    //             )
    //         } else {
    //             return Err(anyhow!("unable to load base translation"));
    //         };

    //         // remaining loaders
    //         for loader in loaders {
    //             trans.add_loader(&mut tags, &mut context, loader)?
    //         }

    //         let extras = if !have_tags {
    //             Some((tags, context))
    //         } else {
    //             None
    //         };
    //         Ok((trans, extras))
    //     } else {
    //         Err(anyhow!("No path present for {}", language,))
    //     }
    // }

    // fn add_loader(
    //     &mut self,
    //     master_tags: &mut TagList,
    //     old_context: &mut ContextList,
    //     mut loader: LoaderOld,
    // ) -> Result<()> {
    //     // make tags list from existing plus new lines
    //     let old_tags_num = master_tags.len();
    //     let mut new_tags = loader.tags();
    //     new_tags.retain(|t| !master_tags.contains(t));
    //     debug!("found {} new tags, adding", new_tags.len()); // should we add to tags, or report extras?
    //     // debug!("{:?}", new_tags);
    //     master_tags.append(&mut new_tags);

    //     let (lines, context) =
    //         master_tags
    //             .iter()
    //             .enumerate()
    //             .fold((Vec::new(), Vec::new()), |mut acc, (_i, tag)| {
    //                 let con = if let Some(context) = loader.get_context(tag) {
    //                     context.to_string()
    //                 } else {
    //                     "".to_string()
    //                 };
    //                 let line = if let Some((line, _con)) = loader.remove_line(tag) {
    //                     line.to_string()
    //                 } else {
    //                     // warn!(
    //                     //     "{} language {} does not include string for '{tag}'",
    //                     //     location, language
    //                     // );
    //                     // missing.push(i); // note missing tag
    //                     String::new()
    //                 };
    //                 acc.0.push(LineData::new(line));
    //                 acc.1.push(con);
    //                 acc
    //             });

    //     for (i, l) in lines.iter().enumerate() {
    //         if i >= old_tags_num {
    //             self.lines.push(l.clone());
    //         } else {
    //             if !l.is_empty() && (self.lines[i] != *l) {
    //                 debug!("updating {}", master_tags[i]);
    //                 self.lines[i] = l.clone();
    //             }
    //         }
    //     }

    //     for (i, l) in context.iter().enumerate() {
    //         if i >= old_tags_num {
    //             old_context.push(l.clone());
    //         } else {
    //             if !l.is_empty() && (old_context[i] != *l) {
    //                 debug!("updating context for {}", master_tags[i]);
    //                 old_context[i] = l.clone();
    //             }
    //         }
    //     }

    //     Ok(())
    // }

    pub fn language(&self) -> &Language {
        &self.language
    }

    /// Returns the list of LineData lines. Use line_data.line() to get current string.
    pub fn lines(&self) -> &Vec<LineData> {
        &self.lines
    }

    // returns the current line
    pub fn line(&self, string_id: usize) -> (&str, ContentType) {
        if string_id < self.lines.len() {
            self.lines[string_id].current_line()
        } else {
            ("", ContentType::Master)
        }
    }

    pub fn missing_lines(&self) -> &Vec<usize> {
        &self.missing
    }

    pub fn overview_string(&self) -> String {
        let lang = self.language.to_string();
        let lines = (self.lines.len() - self.missing.len()).to_string();
        let extra = string_if_non_zero(self.extra_lines.len());
        let missing = string_if_non_zero(self.missing.len());
        format!(
            "{}{}{}",
            fl!(
                "show_translation_info_trans_lines",
                arg!("trans", lang, "lines", lines)
            ),
            if extra.len() > 0 {
                format!(", {}", fl!("show_extra_lines", arg!("lines", extra)))
            } else {
                extra
            },
            if missing.len() > 0 {
                format!(", {}", fl!("show_missing_lines", arg!("lines", missing)))
            } else {
                missing
            },
        )
    }
}

fn string_if_non_zero(value: usize) -> String {
    if value > 0 {
        value.to_string()
    } else {
        String::new()
    }
}

fn get_loader(language: Language, location: Location, input_path: &Path) -> Result<LoaderOld> {
    let filename_base = {
        let mut f = PathBuf::from(input_path);
        // let language = match location {
        //     Location::Internal | Location::External => language.to_string().to_lowercase(),
        //     Location::InProgress => language.to_string().to_capitalized(),
        // };
        f.push(language.to_string()); // !! should we try capital and small on language?
        f
    };

    for ext in location.extensions() {
        let filename = {
            let mut f = filename_base.clone();
            f.set_extension(ext);
            f
        };

        // debug!("trying {}", filename.to_string_lossy());

        if filename.is_file() {
            return LoaderOld::new(&filename);
        }
    }

    Err(anyhow!(
        "No file found for language {} in '{}'",
        language,
        input_path.to_string_lossy()
    ))
}

fn get_loaders(
    language: Language,
    location: Location,
    input_path: &Path,
) -> Result<VecDeque<LoaderOld>> {
    let filename_base = {
        let mut f = PathBuf::from(input_path);
        // let language = match location {
        //     Location::Internal | Location::External => language.to_string().to_lowercase(),
        //     Location::InProgress => language.to_string().to_capitalized(),
        // };
        f.push(language.to_string()); // !! should we try capital and small on language?
        f
    };

    let mut ret = VecDeque::new();
    for ext in location.extensions() {
        let filename = {
            let mut f = filename_base.clone();
            f.set_extension(ext);
            f
        };

        // debug!("trying {}", filename.to_string_lossy());

        if filename.is_file() {
            // return Loader::new(&filename);
            let loader = LoaderOld::new(&filename)?;
            ret.push_back(loader);
        }
    }

    if ret.is_empty() {
        Err(anyhow!(
            "No file found for language {} in '{}'",
            language,
            input_path.to_string_lossy()
        ))
    } else {
        Ok(ret)
    }
}
