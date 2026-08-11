#![allow(dead_code)]

use anyhow::Result;

use crate::languages::Language;

mod dictionary;
mod languages;
mod loader;
mod translation;
mod writer;

mod app;
mod app_settings;
mod child_windows;
mod localize;

const APP_NAME: &str = "Local Words";
const COMPANY_DOMAIN: &str = "com.au";
const COMPANY_NAME: &str = "VectorStorm";
pub const APP_FILE_NAME: &str = "local_words";

const EXTERNAL_DIR: &str = "../test_loc_data/2026_03_20";
const INTERNAL_DIR: &str = "./";
const MASTER_LANGUAGE: Language = Language::English;

fn main() -> Result<()> {
    setup_logger()?;

    {
        use std::env::current_exe;

        use directories_next::ProjectDirs;
        use eframe::{
            NativeOptions,
            egui::{Vec2, ViewportBuilder},
            run_native,
        };
        use log::info;

        use crate::{app::App, app_settings::AppSettings};

        info!("Starting GUI");

        let Ok(exe_path) = current_exe() else {
            panic!("Unable to find exe path");
        };

        let Some(exe_name) = exe_path.file_stem() else {
            panic!("Unable to find exe name in {}", exe_path.display());
        };

        let base_name = exe_name.to_string_lossy();
        let Some(base_dir) = ProjectDirs::from(COMPANY_DOMAIN, COMPANY_NAME, &base_name) else {
            panic!(
                "Unable to find project directory for {COMPANY_DOMAIN}, {COMPANY_NAME}, {base_name}"
            );
        };

        let app_name = format!("{} (version {})", APP_NAME, env!("CARGO_PKG_VERSION"));
        let initial_window_size = Vec2::new(1200., 720.);

        // should load settings, if they exist
        // !! Should prompt for master language on first run, rather than set it here
        // !! or at least UI language
        let settings = AppSettings::new("", MASTER_LANGUAGE);

        let Ok(exe_path) = current_exe() else {
            panic!("Unable to find exe path");
        };

        let Some(_exe_name) = exe_path.file_stem() else {
            panic!("Unable to find exe name in {}", exe_path.display());
        };

        let win_option = NativeOptions {
            viewport: ViewportBuilder::default()
                .with_title(app_name.clone())
                .with_resizable(false)
                // .with_icon(None)
                .with_active(true)
                .with_inner_size(initial_window_size)
                .with_min_inner_size(initial_window_size)
                .with_maximize_button(false)
                .with_drag_and_drop(false),
            ..Default::default()
        };

        let _res = run_native(
            &app_name,
            win_option,
            Box::new(|cc| {
                Ok(Box::new(
                    App::new(settings, base_dir, cc).expect("unable to create app"),
                ))
            }),
        );
    }

    // {
    //     let dictionary = Dictionary::new(EXTERNAL_DIR, MASTER_LANGUAGE)?; // should be internal dir
    //     // load internal languages (not master)
    //     // (optional) load external langauges
    //     // (optional) load inProgress languages
    //     // (optional) export master to inProgress
    //     // (optional) export inProgress to external
    //     // (optional) export inProgress to internal (not master)
    //     //
    //     // should we handle extracting internal files from MMORPG.zip?
    //     dictionary.export_master_for_translation(INTERNAL_DIR)?;
    // }

    println!("Done");
    Ok(())
}

fn setup_logger() -> Result<()> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {}] {}",
                // "[{} {} {}] {}",
                // humantime::format_rfc3339_seconds(SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Warn)
        .level_for("local_words", log::LevelFilter::Debug)
        .chain(std::io::stdout())
        // .chain(fern::log_file("./output.log")?)
        .apply()?;
    Ok(())
}
