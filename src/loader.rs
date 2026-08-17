use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    fs::{File, read_dir},
    io::{BufRead, BufReader, Cursor, Read, Seek},
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::{Ok, Result, anyhow};
use calamine::{DataType, Reader, ReaderRef, Xlsx, open_workbook_from_rs};
use csv::ReaderBuilder;
use json::parse;
use log::{debug, error, info, warn};
use nom::{Parser, bytes::complete::take_until, combinator::rest};
use words_count::WordsCount;
use zip::ZipArchive;

use crate::{
    dictionary::TagList,
    languages::{Language, Languages},
    loader::LoaderContainer::{DirLoader, ZipLoader},
    writer::FormatVersion,
};

const LANGUAGE_DIRECTORY: &str = "i18n/";

#[derive(Debug, Clone)]
pub struct Loader {
    container: LoaderContainer,
}

impl Loader {
    pub fn load_directory(&self) -> PathBuf {
        self.container.load_directory().to_path_buf()
    }

    pub fn zip_loader(zip_file: PathBuf) -> Loader {
        Loader {
            container: ZipLoader(zip_file),
        }
    }

    pub fn dir_loader(directory: PathBuf) -> Loader {
        Loader {
            container: DirLoader(directory),
        }
    }

    pub fn load_primary_language(&self, language: Language) -> Result<Loaded> {
        let buffer = self.container.get_buffer(language)?;
        debug!("have buffer of size {}", buffer.len());
        Loaded::load_primary_language(LoadFormat::Vrt, &buffer)
    }

    pub fn load_translation(&self, language: Language, tags: &TagList) -> Result<Loaded> {
        let buffer = self.container.get_buffer(language)?;
        debug!("have buffer of size {}", buffer.len());
        Loaded::load_with_tags(tags, LoadFormat::Vrt, &buffer)
    }

    pub fn list_languages(&self) -> (HashSet<Language>, Vec<String>) {
        match self.container.list_languages() {
            std::result::Result::Ok(language_lists) => language_lists,

            Err(e) => {
                error!("loader unable to list languages: {e}");
                (HashSet::new(), Vec::new())
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct Loaded {
    tags: HashMap<String, (String, Option<String>)>, // identifier, (text, context)
    lines: usize,
    count: WordsCount,
    unknowns: HashMap<String, (String, Option<String>)>, // identifier, (text, context)
}

impl Loaded {
    fn load_primary_language(format: LoadFormat, buffer: &[u8]) -> Result<Loaded> {
        let lines = format.load(buffer)?;
        let mut loaded = Loaded::default();
        for (tag, line) in lines {
            loaded.add_line(tag.trim(), &line, None, true);
        }
        Ok(loaded)
    }

    fn load_with_tags(tags: &TagList, format: LoadFormat, buffer: &[u8]) -> Result<Loaded> {
        let lines = format.load(buffer)?;
        let mut loaded = Loaded::default();
        for (tag, line) in lines {
            if tags.contains(&tag) {
                loaded.add_line(tag.trim(), &line, None, true);
            } else {
                loaded.add_unknown(tag.trim(), &line, None);
            }
        }
        Ok(loaded)
    }

    pub fn add_line(&mut self, id: &str, text: &str, context: Option<&str>, count_words: bool) {
        self.lines += 1;
        if let Some((old_val, _old_con)) = self.tags.insert(
            id.to_string(),
            (text.to_string(), context.map(|c| c.to_string())),
        ) {
            if old_val == text {
                warn!("found duplicate line for tag [{id}]");
                // no change to word count
            } else {
                error!("found additional definition for tag [{id}]: '{old_val}' vs '{text}'");
                // subtract previous version count, then add this version
                // self.count -= words_count::count(old_val);  // cannot subtract old count
                if count_words {
                    self.count += words_count::count(text);
                }
            }
        } else if count_words {
            // this is a new entry
            self.count += words_count::count(text);
        }
    }

    pub fn add_unknown(&mut self, id: &str, text: &str, context: Option<&str>) {
        warn!("found unknown tag [{id}]");
        if let Some((old_val, _old_con)) = self.unknowns.insert(
            id.to_string(),
            (text.to_string(), context.map(|c| c.to_string())),
        ) {
            if old_val == text {
                warn!("found duplicate unknown line for tag [{id}]");
            } else {
                error!(
                    "found additional unknown definition for tag [{id}]: '{old_val}' vs '{text}'"
                );
            }
        }
    }

    pub fn len(&self) -> usize {
        self.tags.len()
    }

    pub fn lines(&self) -> usize {
        self.lines
    }

    pub fn words(&self) -> usize {
        self.count.words
    }

    pub fn unknowns(&self) -> usize {
        self.unknowns.len()
    }

    pub fn get_line(&self, tag: &str) -> Option<&str> {
        self.tags.get(tag).map(|(t, _c)| t.as_str())
    }

    pub fn tags(&self) -> Vec<String> {
        let mut tags: Vec<String> = self.tags.keys().cloned().collect();
        tags.sort();
        tags
    }

    pub fn all_unknowns(&self) -> HashMap<String, (String, Option<String>)> {
        self.unknowns.clone()
    }
}

#[derive(Debug, Clone)]
pub enum LoaderContainer {
    ZipLoader(PathBuf),
    DirLoader(PathBuf),
}

impl LoaderContainer {
    pub fn load_directory(&self) -> &Path {
        match self {
            ZipLoader(directory) => directory.parent().expect("master data has no directory"),
            DirLoader(directory) => directory.parent().expect("master data has no directory"),
        }
    }

    /// Note: this reads all data into a buffer
    pub fn get_buffer(&self, language: Language) -> Result<Vec<u8>> {
        match self {
            ZipLoader(zip_file) => {
                let mut zfile = open_zip(zip_file)?;
                let pathbase = PathBuf::from_str(LANGUAGE_DIRECTORY)?;
                let interior_path =
                    pathbase.join(language.internal_main_file_name(FormatVersion::Version1));
                debug!(
                    "trying to find file {} in master zip",
                    interior_path.to_string_lossy()
                );
                let mut arc = zfile.by_name(interior_path.to_string_lossy().as_ref())?;
                let mut buf = Vec::new();
                arc.read_to_end(&mut buf)?;
                Ok(buf)
            }

            DirLoader(directory) => {
                let interior_path =
                    directory.join(language.internal_main_file_name(FormatVersion::Version1));
                let mut ret = File::open(interior_path)?;
                let mut buf = Vec::new();
                ret.read_to_end(&mut buf)?;
                Ok(buf)
            }
        }
    }

    pub fn list_languages(&self) -> Result<(HashSet<Language>, Vec<String>)> {
        let mut set = HashSet::new();
        let mut unused = Vec::new();

        match self {
            ZipLoader(zip_file) => {
                // get list of all file names
                let zfile = open_zip(zip_file)?;
                let list = zfile.file_names();
                set = list
                    .filter_map(|filename| {
                        let std::result::Result::Ok(filepath) = PathBuf::from_str(filename);
                        if let std::result::Result::Ok(suffix) =
                            filepath.strip_prefix(LANGUAGE_DIRECTORY)
                        {
                            let mut path = suffix.iter();
                            if let Some(name) = path.next() {
                                if let Some(name) = name.to_str() {
                                    if let Some(lang) = Languages::find_internal(name) {
                                        Some(lang)
                                    } else {
                                        unused.push(name.to_string()); // add to unused list
                                        None
                                    }
                                } else {
                                    warn!("! unable to make string in list languages");
                                    None
                                }
                            } else {
                                warn!("! did not find first component in list languages");
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect();
            }

            DirLoader(directory) => {
                if directory.is_dir() {
                    for filename in read_dir(directory)? {
                        let filename = filename?;
                        let path = filename.path();
                        if path.is_dir() {
                            if let Some(name) = path.file_name() {
                                if let Some(name) = name.to_str() {
                                    if let Some(lang) = Languages::find_internal(name) {
                                        set.insert(lang);
                                    } else {
                                        unused.push(name.to_string()); // add to unused list
                                    }
                                }
                            } else {
                                error!("unable to find file name in {}", path.to_string_lossy());
                            }
                        }
                    }
                }
            }
        }

        Ok((set, unused))
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
enum LoadFormat {
    #[default]
    Vrt,
    Csv,
    Json,
}

impl LoadFormat {
    pub fn load(&self, buffer: &[u8]) -> Result<Vec<(String, String)>> {
        use LoadFormat::*;
        match self {
            Vrt => {
                // let buf = Cursor::new(buffer);
                let lines = self.read_vrt(buffer)?;
                Ok(lines)
            }

            _ => {
                error!("load format not yet implemented: {:?}", self);
                Err(anyhow!("load format not yet implemented: {:?}", self))
            }
        }
    }

    fn read_vrt(&self, buffer: &[u8]) -> Result<Vec<(String, String)>> {
        let mut ret = Vec::new();
        for line in buffer.lines() {
            let line = line?;
            let Some(equal) = line.find(" ") else {
                continue; // ignoring empty? line
            };
            let tag = line[0..equal].trim();
            let val = repair_vrt_line_end(line[equal..].trim(), &line);

            let std::result::Result::Ok(val) = json::parse(val.extract_data()) else {
                error!("can't find string in: {line}");
                continue;
            }; // no string
            ret.push((tag.to_string(), val.to_string()));
            // self.add_line(tag, &val.to_string(), true);
        }
        Ok(ret)
    }
}

// ===============================
// ===============================

#[derive(Default, Debug)]
pub struct LoaderOld {
    file: PathBuf,
    tags: HashMap<String, (String, Option<String>)>, // identifier, (text, context)
    lines: usize,
    count: WordsCount,
}

impl LoaderOld {
    pub fn new(filename: &Path) -> Result<Self> {
        let mut loader = LoaderOld {
            file: PathBuf::from(filename),
            ..Default::default()
        };

        loader.load()?;
        Ok(loader)
    }

    pub fn add_line(&mut self, id: &str, text: &str, context: Option<&str>, count_words: bool) {
        self.lines += 1;
        if let Some((old_val, _old_con)) = self.tags.insert(
            id.to_string(),
            (text.to_string(), context.map(|c| c.to_string())),
        ) {
            if old_val == text {
                warn!("found duplicate line for tag [{id}]");
                // no change to word count
            } else {
                error!("found additional definition for tag [{id}]: '{old_val}' vs '{text}'");
                // subtract previous version count, then add this version
                // self.count -= words_count::count(old_val);  // cannot subtract old count
                if count_words {
                    self.count += words_count::count(text);
                }
            }
        } else if count_words {
            // this is a new entry
            self.count += words_count::count(text);
        }
    }

    pub fn add_empty_line(&mut self) {
        self.lines += 1;
    }

    pub fn filename(&self) -> Cow<'_, str> {
        self.file.to_string_lossy()
    }

    pub fn len(&self) -> usize {
        self.tags.len()
    }

    pub fn lines(&self) -> usize {
        self.lines
    }

    pub fn words(&self) -> usize {
        self.count.words
    }

    pub fn get_line(&self, tag: &str) -> Option<&str> {
        self.tags.get(tag).map(|(t, _c)| t.as_str())
    }

    pub fn get_context(&self, tag: &str) -> Option<&str> {
        self.tags
            .get(tag)
            .map(|(_t, c)| if let Some(c) = c { c.as_str() } else { "" })
    }

    /// Note that this REMOVES a line and its tag from the list
    pub fn remove_line(&mut self, tag: &str) -> Option<(String, Option<String>)> {
        self.tags.remove(tag)
    }

    pub fn tags(&self) -> Vec<String> {
        self.tags.keys().cloned().collect()
    }

    pub fn all_context(&self) -> Vec<String> {
        self.tags
            .values()
            .collect::<Vec<&(String, Option<String>)>>()
            .iter()
            .map(|(_l, c)| {
                if let Some(c) = c {
                    c.clone()
                } else {
                    "".to_string()
                }
            })
            .collect()
    }

    pub fn leftovers(&self) -> HashMap<String, (String, Option<String>)> {
        self.tags.clone()
    }

    fn load(&mut self) -> Result<()> {
        if let Some(ext) = self.file.extension() {
            match File::open(self.file.as_path()) {
                std::result::Result::Ok(file) => match ext.to_string_lossy().as_ref() {
                    "csv" => self.read_csv(file),
                    "json" => self.read_json_context(file),
                    "vrt" => self.read_vrt(file),
                    unknown_ext => {
                        error!(
                            "file with unknown extension [{unknown_ext}]: {}",
                            self.file.to_string_lossy()
                        );
                        Err(anyhow!(
                            "file with unknown extension [{unknown_ext}]: {}",
                            self.file.to_string_lossy()
                        ))
                    }
                },

                Err(e) => {
                    error!("Unable to open [{}]: {e}", self.file.to_string_lossy());
                    Err(anyhow!(
                        "Unable to open [{}]: {e}",
                        self.file.to_string_lossy()
                    ))
                }
            }
        } else {
            error!(
                "no extension found for file: {}",
                self.file.to_string_lossy()
            );
            Err(anyhow!(
                "no extension found for file: {}",
                self.file.to_string_lossy()
            ))
        }
    }

    fn read_csv(&mut self, csv_file: File) -> Result<()> {
        info!("Reading: {}...", self.file.to_string_lossy());
        let mut inp = ReaderBuilder::new()
            .has_headers(false)
            .from_reader(csv_file);
        for res in inp.records() {
            match res {
                std::result::Result::Ok(record) => {
                    if record.len() == 2 {
                        if let Some(item) = record.get(0) {
                            let val = json::stringify(record.get(1).unwrap());
                            self.add_line(item.trim(), &val, None, true);
                        } else {
                            warn!("found line with no tag: {:?}", record);
                            self.add_empty_line();
                        }
                    } else {
                        warn!("found line with {} fields", record.len());
                    }
                }

                Err(e) => {
                    // report, but continue
                    error!("error reading line: {e}");
                }
            }
        }

        Ok(())
    }

    fn read_json(&mut self, mut json_file: File) -> Result<()> {
        info!("Reading: {}...", self.file.to_string_lossy());
        let mut data = String::new();
        match json_file.read_to_string(&mut data) {
            std::result::Result::Ok(_) => match parse(&data) {
                std::result::Result::Ok(parsed) => {
                    // let mut loader = Loader::new(filename);
                    for (tag, val) in parsed.entries() {
                        self.add_line(tag.trim(), &val.to_string(), None, true);
                    }
                    Ok(())
                }
                Err(e) => {
                    error!("unable to parse json data: {e}");
                    Err(e.into())
                }
            },
            Err(e) => {
                error!("unable to read full json file data: {e}");
                Err(e.into())
            }
        }
    }

    fn read_json_context(&mut self, mut json_file: File) -> Result<()> {
        info!("Reading: {}...", self.file.to_string_lossy());
        let mut data = String::new();
        match json_file.read_to_string(&mut data) {
            std::result::Result::Ok(_) => match parse(&data) {
                std::result::Result::Ok(parsed) => {
                    // let mut loader = Loader::new(filename);
                    for (tag, values) in parsed.entries() {
                        let val = values["value"].to_string();
                        let context = values["context"].to_string();
                        self.add_line(tag.trim(), &val, Some(&context), true);
                    }
                    Ok(())
                }
                Err(e) => {
                    error!("unable to parse json data: {e}");
                    Err(e.into())
                }
            },
            Err(e) => {
                error!("unable to read full json file data: {e}");
                Err(e.into())
            }
        }
    }

    fn read_vrt(&mut self, vrt_file: File) -> Result<()> {
        info!("Reading: {}...", self.file.to_string_lossy());
        let lines = read_vrt(vrt_file)?;
        for (tag, line) in lines {
            self.add_line(tag.trim(), &line, None, true);
        }
        Ok(())
    }

    fn read_vrt_from_zip(&mut self, zip_filepath: &Path, interior_path: &Path) -> Result<()> {
        let mut zfile = open_zip(zip_filepath)?;

        let arc = zfile.by_name(interior_path.to_string_lossy().as_ref())?;

        let lines = read_vrt(arc)?;
        for (tag, line) in lines {
            self.add_line(&tag, &line, None, true);
        }
        Ok(())
    }

    fn read_xlsx_from_zip(&mut self, zip_filepath: &Path, interior_path: &str) -> Result<()> {
        let mut zfile = open_zip(zip_filepath)?;

        self.read_xlsx_from_open_zip(&mut zfile, interior_path)
    }

    pub fn read_xlsx_from_open_zip(
        &mut self,
        zfile: &mut ZipArchive<File>,
        interior_path: &str,
    ) -> Result<()> {
        let interior_path = interior_path.replace("\\", "/");
        let mut arc = zfile.by_name(&interior_path)?;
        let size = arc.size() as usize;
        let mut buffer = vec![0u8; size];
        arc.read_to_end(&mut buffer)?;

        let lines = read_xlsx(Cursor::new(buffer))?;
        for (tag, line) in lines {
            let val = json::stringify(line);
            self.add_line(&tag, &val, None, true);
        }
        Ok(())
    }
}

fn read_vrt<F: Read>(file: F) -> Result<Vec<(String, String)>> {
    let mut ret = Vec::new();
    let buf = BufReader::new(file);
    for line in buf.lines() {
        let line = line?;
        let Some(equal) = line.find(" ") else {
            continue; // ignoring empty? line
        };
        let tag = line[0..equal].trim();
        let val = repair_vrt_line_end(line[equal..].trim(), &line);

        let std::result::Result::Ok(val) = json::parse(val.extract_data()) else {
            error!("can't find string in: {line}");
            continue;
        }; // no string
        ret.push((tag.to_string(), val.to_string()));
        // self.add_line(tag, &val.to_string(), true);
    }
    Ok(ret)
}

fn read_xlsx<F: Read + Seek>(file: F) -> Result<Vec<(String, String)>> {
    const COL_TAG: usize = 0; // A
    const COL_STRING: usize = 3; // D

    let mut ret = Vec::new();
    let mut sheet: Xlsx<_> = open_workbook_from_rs(file)?;
    debug!("found worksheets: {:?}", sheet.sheet_names());
    if let Some(range) = sheet.worksheet_range_at_ref(0) {
        match range {
            std::result::Result::Ok(range) => {
                info!("got range: {} rows", range.rows().len());

                let string_col = {
                    let firstcol = (COL_STRING) as u32;
                    let maxcol = (COL_STRING + 5) as u32;
                    let lastrow = (range.height() - 1) as u32;
                    let cols = range.range((0, firstcol), (lastrow, maxcol));
                    if let Some((_row, col, ..)) = cols.used_cells().next() {
                        debug!("found first data in col {col}");
                        (firstcol as usize) + col
                    } else {
                        COL_STRING
                    } // going to crash
                };

                for row in range.rows() {
                    if let Some(tag) = row[COL_TAG].as_string() {
                        if let Some(string) = row[string_col].as_string() {
                            // debug!("found: {tag} => {string}");
                            ret.push((tag, string));
                        } else {
                            warn!("found tag ({tag}) but no string");
                        }
                    } else {
                        warn!("no tag found on row");
                    }
                }
            }
            Err(e) => {
                warn!("found no range of cells in worksheet 1: {e}");
            }
        }
    } else {
        warn!("found no worksheet 1")
    }
    Ok(ret)
}

// Do we really want to return a String?
// Should we be able to return warnings/errors?
fn repair_vrt_line_end(line_entry: &str, full_line: &str) -> ParseResult {
    if let Some(l) = line_entry[1..].rfind("\"") {
        // debug!("string: [{val}]");
        // !! check for second last char is quote
        // !! check for thrid last char is not \
        // should those be u8 checks?
        // !! should this recurrsively check again and again?
        if line_entry.ends_with("\"\"") && !line_entry.ends_with("\\\"\"") {
            warn!("line ends with an extra quote: {full_line}");
            return ParseResult::ExtraQuote(line_entry[..l + 1].to_string());
        }

        // !! we can safely ignore ; and , as final characters
        if l < line_entry.len() - 2 {
            warn!("line with extra trailing characters: {full_line}");
            return ParseResult::Excess(line_entry[..l + 2].to_string());
        }

        ParseResult::Ok(line_entry.to_string())
    } else {
        // !! if there is no final ", that's a problem we should fix?
        warn!("line with missing final quote: {full_line}");
        ParseResult::NoQuote(format!("{}\"", line_entry))
    }
}

pub fn open_zip(filepath: &Path) -> Result<ZipArchive<File>> {
    debug!("opening file: {}", filepath.to_string_lossy());
    let zfile = match File::open(filepath) {
        std::result::Result::Ok(zfile) => zfile,
        Err(e) => {
            error!("unable to open zip file '{}'", filepath.to_string_lossy());
            return Err(e.into());
        }
    };

    ZipArchive::new(zfile).map_err(|ze| ze.into())
}

fn nom_vrt_line(line: &str) -> Option<(String, String)> {
    if line.is_empty() {
        return None;
    };

    match take_until(" ")
        .parse(line)
        .map_err(|e: nom::Err<nom::error::Error<&str>>| e.to_owned())
    {
        std::result::Result::Ok((input, tag)) => {
            match rest(input).map_err(|e: nom::Err<nom::error::Error<&str>>| e.to_owned()) {
                std::result::Result::Ok((_, line)) => Some((tag.to_string(), line.to_string())), // !! remove "" from start and end, if present
                Err(e) => {
                    error!("found {e} in rest for: {line}");
                    None
                }
            }
        }
        Err(e) => {
            error!("found {e} in take_until for: {line}");
            None
        }
    }
}

enum ParseResult {
    Ok(String),
    Excess(String), // should we note Excess characters?
    NoQuote(String),
    ExtraQuote(String),
}

impl ParseResult {
    pub fn extract_data(&self) -> &str {
        use ParseResult::*;
        match self {
            Ok(line) => line.as_str(),
            Excess(line) => line.as_str(),
            NoQuote(line) => line.as_str(),
            ExtraQuote(line) => line.as_str(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::io::{BufRead, BufReader};
    use std::path::Path;

    use anyhow::Result;
    use log::info;

    use crate::loader::nom_vrt_line;
    use crate::loader::open_zip;
    use crate::setup_logger;

    // #[test]
    // fn read_vrt() {
    //     let _ = setup_logger();

    //     let path = Path::new("../test_loc_data/english.vrt");
    //     let _loader = LoaderOld::new(&path);
    // }

    // #[test]
    // fn read_total_json() -> Result<()> {
    //     let _ = setup_logger();

    //     let path = Path::new("../test_loc_data/Total/English.json");
    //     let loader = LoaderOld::new(&path)?;
    //     let mut c_count = 0;
    //     for con in loader.all_context() {
    //         if !con.is_empty() {
    //             c_count += 1;
    //         }
    //     }

    //     info!("found {c_count} lines with context");

    //     Ok(())
    // }

    // #[test]
    // fn read_vrt_from_zip() -> Result<()> {
    //     let _ = setup_logger();

    //     let zpath = Path::new("../test_loc_data/MMORPG.zip");
    //     let lpath = Path::new("i18n/english.vrt");
    //     let mut loader = LoaderOld::default();
    //     let _ = loader.read_vrt_from_zip(zpath, lpath)?;
    //     info!("Done, found {} tags", loader.len());
    //     Ok(())
    // }

    // #[test]
    // fn read_xlsx_from_zip() -> Result<()> {
    //     let _ = setup_logger();

    //     let zpath = Path::new("../test_loc_data/2026-04-10.zip");
    //     let lpath = "2026-04-10/English_fre.xlsx";
    //     let mut loader = LoaderOld::default();
    //     let _ = loader.read_xlsx_from_zip(zpath, lpath)?;
    //     info!("Done, found {} tags", loader.len());
    //     Ok(())
    // }

    #[test]
    fn nom_vrt() -> Result<()> {
        let _ = setup_logger();

        let zpath = Path::new("../test_loc_data/MMORPG.zip");
        let lpath = Path::new("i18n/english.vrt");
        let mut zfile = open_zip(zpath)?;

        let arc = zfile.by_name(lpath.to_string_lossy().as_ref())?;

        let buf = BufReader::new(arc);
        let mut lines = HashMap::new();
        for line in buf.lines().map_while(Result::ok) {
            if let Some((tag, value)) = nom_vrt_line(line.trim()) {
                // debug!("found tag'{tag}': {value}");
                lines.insert(tag, value);
            }
        }

        info!("found {} entries", lines.len());
        Ok(())
    }
}
