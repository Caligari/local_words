#![allow(dead_code)]

use anyhow::Result;

use crate::display::{BETWEEN_COLS, EDGE_COLUMN_WIDTH, STRING_WIDTH};

mod app;
mod app_settings;
mod child_windows;
mod dictionary;
mod display;
mod languages;
mod loader;
mod localize;
mod translation;
mod writer;

const APP_NAME: &str = "Local Words";
const COMPANY_DOMAIN: &str = "com.au";
const COMPANY_NAME: &str = "VectorStorm";

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
        let initial_window_size = Vec2::new(
            EDGE_COLUMN_WIDTH + (STRING_WIDTH * 2.0) + BETWEEN_COLS + EDGE_COLUMN_WIDTH,
            800.,
        );

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
