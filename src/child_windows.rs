use std::{fs::create_dir_all, path::PathBuf, sync::Arc};

use eframe::egui::{
    Align, CentralPanel, Context, Layout, RichText, ScrollArea, Ui, ViewportBuilder, ViewportId,
    mutex::RwLock,
};
use egui_file_dialog::{DialogState, FileDialog, OpeningMode};
use log::{debug, error, info};

use crate::{
    APP_NAME,
    app::{CHANGE_NOTES, HELP_TEXT, UI_PADDING},
    localize::{arg, fl},
};

pub struct ChildWindows {
    show_about: Arc<RwLock<bool>>,
    version: String,
    // file_dialog_internal: FileDialogControl,
    file_dialog_master_zip_load: FileDialogControl,
    file_dialog_master_dir_load: FileDialogControl,
    file_dialog_import_zip: FileDialogControl,
    file_dialog_export_zip: FileDialogControl,
    file_dialog_externals_zip: FileDialogControl,
    // file_dialog_export: FileDialogControl,
    selected_file: Arc<RwLock<Option<PathBuf>>>,
}

impl Default for ChildWindows {
    fn default() -> Self {
        ChildWindows {
            show_about: Arc::default(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            // file_dialog_internal: FileDialogControl::new(FileTarget::Internal),
            file_dialog_master_zip_load: FileDialogControl::new(FileTarget::MasterZip),
            file_dialog_master_dir_load: FileDialogControl::new(FileTarget::MasterDir),
            file_dialog_import_zip: FileDialogControl::new(FileTarget::ImportZip),
            file_dialog_export_zip: FileDialogControl::new(FileTarget::ExportZip),
            file_dialog_externals_zip: FileDialogControl::new(FileTarget::ExternalsZip),
            // file_dialog_export: FileDialogControl::new(FileTarget::Export),
            selected_file: Arc::default(),
        }
    }
}

impl ChildWindows {
    pub fn toggle_about(&mut self) {
        let current_value = *self.show_about.read();
        *self.show_about.write() = !current_value;
    }

    /// show the file dialog
    pub fn start_file_dialog(
        &mut self,
        dialog_type: FileDialogType,
        target_type: FileTarget,
        initial_directory: Option<PathBuf>, // should this be here, or passed at create?
    ) {
        // start dialog
        let dialog_control = match target_type {
            FileTarget::ImportZip => &mut self.file_dialog_import_zip,
            FileTarget::ExportZip => &mut self.file_dialog_export_zip,
            FileTarget::ExternalsZip => &mut self.file_dialog_externals_zip,
            FileTarget::MasterZip => &mut self.file_dialog_master_zip_load,
            FileTarget::MasterDir => &mut self.file_dialog_master_dir_load,
            // FileTarget::Internal => &mut self.file_dialog_internal,
            // FileTarget::Export => &mut self.file_dialog_export,
        };

        let config = dialog_control.dialog.config_mut();

        if let Some(initial_directory) = initial_directory {
            if let Err(e) = create_dir_all(initial_directory.clone()) {
                // can fail; do we need Result?
                error!(
                    "unable to create initial directory for file dialog [{}]: {}",
                    initial_directory.to_string_lossy(),
                    e
                );
            } else {
                config.initial_directory = initial_directory;
            }
        }

        info!("starting file dialog");

        match dialog_type {
            FileDialogType::ImportZip => dialog_control.dialog.pick_file(),
            FileDialogType::LoadMasterZip => dialog_control.dialog.pick_file(),
            FileDialogType::LoadMasterDir => dialog_control.dialog.pick_directory(),
            // FileDialogType::Load => dialog_control.dialog.pick_file(),
            // FileDialogType::Save => dialog_control.dialog.save_file(),
            FileDialogType::ExportZip => dialog_control.dialog.save_file(),
        }

        dialog_control.current_mode = Some(dialog_type);
        *dialog_control.show_dialog.write() = true;
        *self.selected_file.write() = None;
    }

    pub fn selected_file(&self) -> Option<PathBuf> {
        self.selected_file.read().clone()
    }

    pub fn show_windows(&mut self, ui: &Ui) {
        // Note: this needs to update its own show/hide, if appropriate

        let about_value = *self.show_about.read();
        if about_value {
            self.about(ui);
        }

        if let Some(new_file) = self.file_dialog_master_zip_load.update(ui) {
            *self.selected_file.write() = new_file;
        }

        if let Some(new_file) = self.file_dialog_master_dir_load.update(ui) {
            *self.selected_file.write() = new_file;
        }

        if let Some(new_file) = self.file_dialog_import_zip.update(ui) {
            *self.selected_file.write() = new_file;
        }

        if let Some(new_file) = self.file_dialog_export_zip.update(ui) {
            *self.selected_file.write() = new_file;
        }

        if let Some(new_file) = self.file_dialog_externals_zip.update(ui) {
            *self.selected_file.write() = new_file;
        }

        // if let Some(new_file) = self.file_dialog_internal.update(ctx) {
        //     *self.selected_file.write() = new_file;
        // }
        // if let Some(new_file) = self.file_dialog_export.update(ctx) {
        //     *self.selected_file.write() = new_file;
        // }
    }

    fn about(&self, ctx: &Context) {
        // !! Add credit for Noto fonts!
        //
        //
        //
        let window_title = fl!("about");
        let title = format!("{APP_NAME} - {window_title}"); // can we make this a const?
        let v_id = ViewportId::from_hash_of(&title);
        let show_about = self.show_about.clone();
        let ver_num = self.version.clone();
        ctx.show_viewport_deferred(
            v_id,
            ViewportBuilder::default().with_title(&title),
            move |ctx, _class| {
                CentralPanel::default().show(ctx, |ui| {
                    ui.with_layout(Layout::top_down(Align::Center), |ui| {
                        const HELP_WIDTH: f32 = 0.75;
                        const SCROLL_HEIGHT: f32 = 160.0;
                        let all_width = ui.available_width();
                        ui.set_max_width(all_width * HELP_WIDTH);

                        ui.add_space(UI_PADDING * 3.5);
                        let app_name = RichText::new(APP_NAME).heading();
                        ui.label(app_name);
                        ui.label(fl!("version_ver", arg!("ver", ver_num)));
                        ui.add_space(UI_PADDING);
                        ui.label(HELP_TEXT.join("\n"));
                        ui.add_space(UI_PADDING * 1.15);
                        ui.label(RichText::new(fl!("changenotes")).underline());
                        ScrollArea::vertical()
                            .max_height(SCROLL_HEIGHT)
                            .show(ui, |ui| {
                                ui.label(CHANGE_NOTES.join("\n"));
                            });
                        // ui.label(CHANGE_NOTES.join("\n"));
                        ui.add_space(UI_PADDING * 1.15);
                        ui.separator();
                        ui.add_space(UI_PADDING);
                        ui.label(fl!("about_fonts"));
                        ui.add_space(UI_PADDING * 2.);
                    })
                });

                if ctx.input(|i| i.viewport().close_requested()) {
                    debug!("{} requested close", &title);
                    let mut show_me = show_about.write();
                    *show_me = false;
                }
            },
        );
    }
}

// ---------------------
// FileDialogType

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileDialogType {
    // Load,
    // Save,
    LoadMasterZip,
    LoadMasterDir,
    ImportZip,
    ExportZip,
}

// ---------------------
// FileDialogType

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileTarget {
    // Internal,
    MasterZip,
    MasterDir,
    ImportZip,
    ExportZip,
    ExternalsZip,
    // Export,
}

// ---------------------
// File Dialog Control

struct FileDialogControl {
    show_dialog: Arc<RwLock<bool>>,
    dialog: FileDialog,
    target: FileTarget,
    current_mode: Option<FileDialogType>,
}

const IMPORT_ZIP_EXTENSION: &str = "zip";
const MASTER_EXTENSION: &str = "csv";

impl FileDialogControl {
    pub fn new(target: FileTarget) -> Self {
        // todo: position, etc
        // default_pos
        // default_size
        // max_size
        // anchor
        // resizeable
        //
        let dialog = match target {
            // FileTarget::Internal => FileDialog::new()
            //     .opening_mode(OpeningMode::AlwaysInitialDir)
            //     .default_file_name(fl!("default_internal_file").as_str())
            //     .allow_file_overwrite(false)
            //     .add_file_filter_extensions(
            //         fl!("file_dialog_save_files").as_str(),
            //         vec![SAVE_EXTENSION],
            //     )
            //     .add_save_extension(fl!("file_dialog_save_file").as_str(), SAVE_EXTENSION)
            //     .default_file_filter(fl!("file_dialog_save_files").as_str())
            //     .default_save_extension(fl!("file_dialog_save_file").as_str())
            //     .allow_path_edit_to_save_file_without_extension(true)
            //     .load_via_thread(true),
            // FileTarget::Export => FileDialog::new()
            //     .opening_mode(OpeningMode::LastPickedDir)
            //     .default_file_name(fl!("default_export_file").as_str())
            //     .allow_file_overwrite(true)
            //     .add_file_filter_extensions(
            //         fl!("file_dialog_export_files").as_str(),
            //         vec![JSON_EXTENSION],
            //     )
            //     .add_save_extension(fl!("file_dialog_export_file").as_str(), JSON_EXTENSION)
            //     .default_file_filter(fl!("file_dialog_export_files").as_str())
            //     .default_save_extension(fl!("file_dialog_export_file").as_str())
            //     .allow_path_edit_to_save_file_without_extension(true)
            //     .load_via_thread(true),
            // FileTarget::Master => FileDialog::new()
            //     .opening_mode(OpeningMode::AlwaysInitialDir)
            //     .default_file_name(fl!("default_master_file").as_str())
            //     .allow_file_overwrite(false)
            //     .add_file_filter_extensions(
            //         fl!("file_dialog_master_files").as_str(),
            //         vec![MASTER_EXTENSION],
            //     )
            //     .add_save_extension(fl!("file_dialog_master_file").as_str(), MASTER_EXTENSION)
            //     .default_file_filter(fl!("file_dialog_master_files").as_str())
            //     .default_save_extension(fl!("file_dialog_master_file").as_str())
            //     .allow_path_edit_to_save_file_without_extension(true)
            //     .load_via_thread(true),
            FileTarget::MasterDir => FileDialog::new() // !! working on this
                .opening_mode(OpeningMode::LastPickedDir)
                .default_file_name(fl!("load_master_directory").as_str())
                .allow_file_overwrite(true)
                // .add_file_filter_extensions(
                //     fl!("file_dialog_master_files").as_str(),
                //     vec![MASTER_EXTENSION],
                // )
                // .add_save_extension(fl!("file_dialog_master_file").as_str(), MASTER_EXTENSION)
                // .default_file_filter(fl!("file_dialog_master_files").as_str())
                // .default_save_extension(fl!("file_dialog_master_file").as_str())
                .allow_path_edit_to_save_file_without_extension(true)
                .load_via_thread(true),
            FileTarget::MasterZip => FileDialog::new() // !! working on this
                .opening_mode(OpeningMode::LastPickedDir)
                .default_file_name(fl!("load_master_zip").as_str())
                .allow_file_overwrite(true)
                .add_file_filter_extensions(
                    fl!("file_dialog_import_zip_files").as_str(),
                    vec![IMPORT_ZIP_EXTENSION],
                )
                .add_save_extension(
                    fl!("file_dialog_import_zip_file").as_str(),
                    IMPORT_ZIP_EXTENSION,
                )
                .default_file_filter(fl!("file_dialog_import_zip_files").as_str())
                .default_save_extension(fl!("file_dialog_import_zip_file").as_str())
                .allow_path_edit_to_save_file_without_extension(true)
                .load_via_thread(true),

            FileTarget::ImportZip => FileDialog::new()
                .opening_mode(OpeningMode::LastPickedDir)
                .default_file_name(fl!("default_import_zip_file").as_str())
                .allow_file_overwrite(true)
                .add_file_filter_extensions(
                    fl!("file_dialog_import_zip_files").as_str(),
                    vec![IMPORT_ZIP_EXTENSION],
                )
                .add_save_extension(
                    fl!("file_dialog_import_zip_file").as_str(),
                    IMPORT_ZIP_EXTENSION,
                )
                .default_file_filter(fl!("file_dialog_import_zip_files").as_str())
                .default_save_extension(fl!("file_dialog_import_zip_file").as_str())
                .allow_path_edit_to_save_file_without_extension(true)
                .load_via_thread(true),

            FileTarget::ExportZip => FileDialog::new()
                .opening_mode(OpeningMode::LastPickedDir)
                .default_file_name(fl!("default_export_zip_file").as_str())
                .allow_file_overwrite(true)
                .add_file_filter_extensions(
                    fl!("file_dialog_import_zip_files").as_str(),
                    vec![IMPORT_ZIP_EXTENSION],
                )
                .add_save_extension(
                    fl!("file_dialog_import_zip_file").as_str(),
                    IMPORT_ZIP_EXTENSION,
                )
                .default_file_filter(fl!("file_dialog_import_zip_files").as_str())
                .default_save_extension(fl!("file_dialog_import_zip_file").as_str())
                .allow_path_edit_to_save_file_without_extension(true)
                .load_via_thread(true),

            FileTarget::ExternalsZip => FileDialog::new()
                .opening_mode(OpeningMode::LastPickedDir)
                .default_file_name(fl!("default_externals_zip_file").as_str())
                .allow_file_overwrite(true)
                .add_file_filter_extensions(
                    fl!("file_dialog_import_zip_files").as_str(),
                    vec![IMPORT_ZIP_EXTENSION],
                )
                .add_save_extension(
                    fl!("file_dialog_import_zip_file").as_str(),
                    IMPORT_ZIP_EXTENSION,
                )
                .default_file_filter(fl!("file_dialog_import_zip_files").as_str())
                .default_save_extension(fl!("file_dialog_import_zip_file").as_str())
                .allow_path_edit_to_save_file_without_extension(true)
                .load_via_thread(true),
        };

        FileDialogControl {
            show_dialog: Arc::default(),
            dialog,
            target,
            current_mode: None,
        }
    }

    fn update(&mut self, ctx: &Context) -> Option<Option<PathBuf>> {
        if *self.show_dialog.read() {
            self.dialog.update(ctx);

            if let Some(path) = self.dialog.take_picked() {
                info!("selected file, closing file dialog");
                // add extension if needed
                // let path = if self.current_mode == Some(FileDialogType::Save)
                //     && path.extension().is_none()
                // {
                //     path.with_extension(match self.target {
                //         // FileTarget::Internal => SAVE_EXTENSION,
                //         // FileTarget::Export => JSON_EXTENSION,
                //         FileTarget::ImportZip => IMPORT_ZIP_EXTENSION,
                //     })
                // } else {
                //     path
                // };
                *self.show_dialog.write() = false;
                self.current_mode = None;
                Some(Some(path))
            } else {
                use DialogState::*;
                if *self.dialog.state() == Cancelled {
                    *self.show_dialog.write() = false;
                    self.current_mode = None;
                    Some(Some(PathBuf::new()))
                } else {
                    Some(None)
                }
            }
        } else {
            None
        }
    }
}
