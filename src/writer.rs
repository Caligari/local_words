use std::{
    ffi::OsStr,
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::{Ok, Result};
use csv::WriterBuilder;
use log::{debug, info};
use zip::{ZipWriter, write::SimpleFileOptions};

use crate::{
    dictionary::{ContextList, TagList},
    translation::Translation,
};

pub type MainTags = Vec<String>;

pub fn create_externals_zip(
    filename: &Path,
    main_tags: &MainTags,
    translations: Vec<Translation>,
) -> Result<()> {
    assert_eq!(filename.extension(), Some(OsStr::new("zip")));
    debug!("creating externals {}", filename.to_string_lossy());
    let mut file = File::create(filename)?;
    let mut archive = ZipWriter::new(&mut file);

    let options = SimpleFileOptions::default();
    let internal_path = PathBuf::from_str("")?;
    // archive.add_directory_from_path(&internal_path, options)?;

    for trans in translations {
        let filename = trans.language().internal_category_file_name();
        let mut filepath = internal_path.clone();
        filepath.push(filename);
        debug!("write of {} started", filepath.to_string_lossy());
        archive.start_file_from_path(&filepath, options)?;
        for (tag, l_data) in main_tags.iter().zip(trans.lines()) {
            let (item, _ctype) = l_data.line(); // assuming we want the current version
            writeln!(archive, "{tag} {item}")?;
        }
        debug!("write of {} completed", filepath.to_string_lossy());
    }

    debug!("all contents complete, finishing externals zip");

    archive.finish()?;
    info!(
        "contents of externals zip {} complete",
        filename.to_string_lossy()
    );

    Ok(())
}

pub fn create_vrt_zip(
    filename: &Path,
    internal_directory: &str,
    format: FormatVersion,
    tags: &TagList,
    translations: Vec<Translation>,
) -> Result<()> {
    assert_eq!(filename.extension(), Some(OsStr::new("zip")));
    debug!("creating export {}", filename.to_string_lossy());
    let mut file = File::create(filename)?;
    let mut archive = ZipWriter::new(&mut file);

    let options = SimpleFileOptions::default();
    let internal_path = PathBuf::from_str(internal_directory)?;
    archive.add_directory_from_path(&internal_path, options)?;

    for trans in translations {
        let filename = trans.language().internal_main_file_name(format);
        let mut filepath = internal_path.clone();
        filepath.push(filename);
        debug!("write of {} started", filepath.to_string_lossy());
        archive.start_file_from_path(&filepath, options)?;
        for (tag, l_data) in tags.iter().zip(trans.lines()) {
            let (item, _ctype) = l_data.line(); // assuming we want the current version
            writeln!(archive, "{tag} {item}")?;
        }
        debug!("write of {} completed", filepath.to_string_lossy());
    }

    debug!("all contents complete, finishing export zip");

    archive.finish()?;
    info!(
        "contents of export zip {} complete",
        filename.to_string_lossy()
    );

    Ok(())
}

/// Write strings for a language out to a vrt file

pub fn write_lang_vrt(filename: &Path, tags: &TagList, lang_strings: &Vec<String>) -> Result<()> {
    assert_eq!(tags.len(), lang_strings.len());
    assert_eq!(filename.extension(), Some(OsStr::new("vrt")));
    debug!("write of {} started", filename.to_string_lossy());
    let file = File::create(filename)?;
    let mut buf = BufWriter::new(file);
    for (tag, item) in tags.iter().zip(lang_strings) {
        writeln!(buf, "{tag} {item}")?;
    }
    debug!("write of {} completed", filename.to_string_lossy());
    Ok(())
}

/// Write strings and context for a language out to a csv file

pub fn write_lang_csv(
    filename: &Path,
    tags: &TagList,
    lang_strings: &Vec<&str>,
    context: &ContextList,
) -> Result<()> {
    assert_eq!(tags.len(), lang_strings.len());
    assert_eq!(tags.len(), context.len());
    assert_eq!(filename.extension(), Some(OsStr::new("csv")));
    info!("writing {}...", filename.to_string_lossy());
    let mut csv_writer = WriterBuilder::new()
        .quote_style(csv::QuoteStyle::NonNumeric)
        .from_path(filename)?;

    for ((tag, item), context) in tags.iter().zip(lang_strings).zip(context) {
        let item = format!("{item}");
        csv_writer.write_record(&[tag, &item, context])?;
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatVersion {
    Version0, // il8n / lang.vrt
    Version1, // il8n / lang / num-filename.vrt
}
