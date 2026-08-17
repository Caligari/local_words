#![allow(dead_code)]

use std::{env::current_exe, sync::LazyLock};

use anyhow::Result;
use directories_next::ProjectDirs;

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

const MASTER_LANGUAGE: Language = Language::English;

pub const APP_FILE_NAME: LazyLock<String> = LazyLock::new(|| {
    let Ok(exe_path) = current_exe() else {
        panic!("Unable to find exe path");
    };

    let Some(exe_name) = exe_path.file_stem() else {
        panic!("Unable to find exe name in {}", exe_path.display());
    };

    let base_name = exe_name.to_string_lossy();
    base_name.to_string()
});

pub const PROJECT_DIRECTORY: LazyLock<ProjectDirs> = LazyLock::new(|| {
    let base_name = &*APP_FILE_NAME;
    let Some(base_dir) = ProjectDirs::from(COMPANY_DOMAIN, COMPANY_NAME, &base_name) else {
        panic!(
            "Unable to find project directory for {COMPANY_DOMAIN}, {COMPANY_NAME}, {base_name}"
        );
    };
    base_dir
});

fn main() -> Result<()> {
    setup_logger()?;

    {
        use eframe::{
            NativeOptions,
            egui::{Vec2, ViewportBuilder},
            run_native,
        };
        use log::info;

        use crate::app::App;

        info!("Starting GUI");

        let app_name = format!("{} (version {})", APP_NAME, env!("CARGO_PKG_VERSION"));
        let initial_window_size = Vec2::new(1200., 720.);

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
            Box::new(|cc| Ok(Box::new(App::new(cc).expect("unable to create app")))),
        );
    }

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
