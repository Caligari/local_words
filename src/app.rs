use std::{
    cell::{RefCell, RefMut},
    collections::BTreeMap,
    fmt::Display,
    path::PathBuf,
    sync::Arc,
    thread,
};

use anyhow::Result;
use crossbeam_channel::{Receiver, bounded};
use directories_next::ProjectDirs;
use eframe::{
    CreationContext, Frame,
    egui::{
        Align, Button, CentralPanel, Color32, FontData, FontDefinitions, FontFamily, FontId,
        Layout, MenuBar, Panel, RichText, Separator, Spinner, Ui, Vec2, ViewportCommand,
        style::TextStyle,
    },
};
use log::{debug, error, info, warn};

use crate::{
    app_settings::AppSettings,
    child_windows::{ChildWindows, FileDialogType, FileTarget},
    dictionary::Dictionary,
    languages::{Language, Languages, select_language},
    loader::Loader,
    localize::{arg, fl},
    // translation::Translation,
    // writer::{CategoryTags, FormatVersion, create_externals_zip, create_vrt_zip},
};

pub const UI_PADDING: f32 = 8.0;
const ERROR_SPACE: f32 = 16.0;

const ERROR_BACKGROUND: Color32 = Color32::from_rgb(255, 190, 190);
const ERROR_FOREGROUND: Color32 = Color32::DARK_RED;

const MODE_COLOR: Color32 = Color32::DARK_GREEN;

// todo: localize this
// probably could be one phrase?
pub const HELP_TEXT: &[&str] = &[
    "created by Liam Routt",
    "",
    "This utility allows you to edit and update localizations.",
    "",
];

// ? Can these be localized?
pub const CHANGE_NOTES: &[&str] = &["0.1.0 - initial version"];

#[allow(dead_code)]
pub struct App {
    settings: AppSettings,
    status: AppStatus,
    directories: ProjectDirs,
    data: Option<Dictionary>, // cow? box?
    message: Option<String>,
    child_windows: ChildWindows,
    // do we need todo/undo
    // todo_undo: TodoUndo,
}

#[allow(clippy::match_single_binding)]
#[allow(unused_imports)]
impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, frame: &mut Frame) {
        use AppStatus::*;

        ui.set_visuals(self.settings.theme().default_visuals());

        self.show_top(ui, frame);
        self.show_footer(ui);

        if let Some(new_status) = CentralPanel::default()
            .show(ui, |ui: &mut Ui| {
                // this returns Option<AppStatus>
                match (&self.status, &mut self.data) {
                    (AppStatus::Starting, _) => {
                        // what do we need to do?
                        self.data = None;
                        info!("Starting => Ready");
                        Some(AppStatus::Ready)
                        // this returns Option<AppStatus>
                    }

                    // this returns Option<AppStatus>
                    (AppStatus::Settings, _) => self.settings.show_and_edit(ui),

                    // we have no data, we must create or load some
                    // this returns Option<AppStatus>
                    (_status, None) => self.no_data(ui),

                    // We have data, what do we show?
                    (_status, Some(data)) => {
                        // we have data, so show what info we need
                        data.show(ui)
                        // this returns Option<AppStatus>
                    }
                }
            })
            .inner
        {
            // change status / mode
            self.status = new_status;
            // self.mode = new_mode;
        }

        // if let Some(new_status) = CentralPanel::default()
        //     .show(ctx, |ui: &mut Ui| match self.mode {
        //         AppMode::Package => self.package_mode(ui),
        //         AppMode::Extras => self.extras_mode(ui),
        //     })
        //     .inner
        // {
        //     // change status / mode
        //     self.status = new_status;
        //     // self.mode = new_mode;
        // }
        // }

        self.child_windows.show_windows(ui);
    }

    // This runs automatically when the application closes
    // fn on_exit(&mut self, gl: Option<&glow::Context>) {
    //     if let Some(gl) = gl {
    //         // Pass the glow context to safely free GPU resources
    //         self.gl_painter.destroy(gl);
    //     }
    // }
}

// ---------------------

impl App {
    pub fn new(
        settings: AppSettings,
        directories: ProjectDirs,
        cc: &CreationContext<'_>,
    ) -> Result<Self> {
        configure_fonts(cc, settings.zoom());

        let data = None;
        //     {
        //     match Dictionary::new(&settings.internal_path(), settings.master_language()) {
        //         Ok(dictionary) => Some(dictionary),
        //         Err(e) => {
        //             warn!("No dictionary in new App: {e}");
        //             None
        //         }
        //     }
        // };
        Ok(App {
            settings,
            directories,
            status: AppStatus::default(),
            data,
            message: None,
            child_windows: ChildWindows::default(),
            // todo_undo: TodoUndo::default(),
        })
    }

    fn show_top(&mut self, ui: &mut Ui, _frame: &mut Frame) {
        Panel::top("top").show(ui, |ui| {
            MenuBar::new().ui(ui, |ui| {
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    // when can we save/load?
                    let has_data = self.data.is_some();
                    let load_enabled = matches!(self.status, AppStatus::Ready); // only save from main list
                    let save_as_enabled = load_enabled && has_data; // && !self.data.no_items();
                    let in_settings = matches!(self.status, AppStatus::Settings);

                    ui.menu_button(fl!("menu"), |ui| {
                        if ui.button(fl!("menu_restart")).clicked() {
                            self.status = AppStatus::Starting;
                            self.message = None;
                            info!("Requested Restart");
                        }
                        ui.add(Separator::default().spacing(2.));
                        if ui
                            .add_enabled(load_enabled, Button::new(fl!("menu_load")))
                            .clicked()
                        {
                            info!("Requested Load");
                            // self.child_windows.start_file_dialog(
                            //     FileDialogType::Load,
                            //     FileTarget::Internal,
                            //     self.project_directories.data_dir().to_path_buf(),
                            // );
                            // self.status = AppStatus::Load;
                        }
                        if ui
                            .add_enabled(save_as_enabled, Button::new(fl!("menu_save")))
                            .clicked()
                        {
                            // TODO: save data
                            info!("Requested Save");
                            // if self.data.get_loaded_from().is_some() {
                            //     info!("Doing SaveTo");
                            //     self.status = AppStatus::SaveTo;
                            // } else {
                            //     info!("Forced SaveAs -> no loaded file data present");
                            //     // todo: provide default name?
                            //     self.child_windows.start_file_dialog(
                            //         FileDialogType::Save,
                            //         FileTarget::Internal,
                            //         self.project_directories.data_dir().to_path_buf(),
                            //     );
                            //     self.status = AppStatus::SaveAs;
                            // }
                        }
                        if ui
                            .add_enabled(save_as_enabled, Button::new(fl!("menu_save_as")))
                            .clicked()
                        {
                            // TODO: save as data - provide default name?
                            info!("Requested Save As");
                            // self.child_windows.start_file_dialog(
                            //     FileDialogType::Save,
                            //     FileTarget::Internal,
                            //     self.project_directories.data_dir().to_path_buf(),
                            // );
                            // self.status = AppStatus::SaveAs;
                        }
                        ui.add(Separator::default().spacing(2.));
                        if ui
                            .add_enabled(load_enabled && has_data, Button::new(fl!("menu_import")))
                            .clicked()
                        {
                            // TODO: import data
                            info!("Requested Import");
                            // self.child_windows.start_file_dialog(
                            //     FileDialogType::Load,
                            //     FileTarget::Export,
                            //     self.project_directories.data_dir().to_path_buf(),
                            // );
                            // self.status = AppStatus::Import;
                        }
                        if ui
                            .add_enabled(
                                save_as_enabled && has_data,
                                Button::new(fl!("menu_export")),
                            )
                            .clicked()
                        {
                            // TODO: export data
                            info!("Requested Export");
                            // self.child_windows.start_file_dialog(
                            //     FileDialogType::Save,
                            //     FileTarget::Export,
                            //     self.project_directories.data_dir().to_path_buf(),
                            // );
                            // self.status = AppStatus::Export;
                        }
                        ui.add(Separator::default().spacing(2.));
                        if ui
                            .add_enabled(!in_settings, Button::new(fl!("menu_settings")))
                            .clicked()
                        {
                            self.status = AppStatus::Settings;
                            self.message = None;
                            info!("Requested Settings");
                        }
                        ui.add(Separator::default().spacing(2.));
                        if ui.button(fl!("menu_exit")).clicked() {
                            info!("Requested Exit");
                            ui.send_viewport_cmd(ViewportCommand::Close);
                        }
                    });

                    // ui.add_space(20.0);

                    // for mode in all::<AppMode>() {
                    //     if self.mode == mode {
                    //         let mode_text =
                    //             RichText::from(mode.to_string()).strong().color(MODE_COLOR);
                    //         ui.label(mode_text);
                    //     } else {
                    //         let mode_text = RichText::from(mode.to_string());
                    //         if ui.button(mode_text).clicked() {
                    //             self.mode = mode;
                    //             info!("changd mode to {mode}");
                    //         }
                    //     }
                    // }

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button(fl!("about")).clicked() {
                            self.child_windows.toggle_about();
                            info!("Requested About");
                        }
                    });
                });
            });
        });
    }

    fn show_footer(&mut self, ui: &mut Ui) {
        Panel::bottom("footer").show(ui, |ui| {
            ui.add_space(5.);
            ui.horizontal(|ui| {
                ui.label(self.status.to_string());
                // if let Some(file) = self.data.get_loaded_from()
                //     && let Some(name) = file.file_stem()
                // {
                //     ui.label(RichText::new(format!("({})", name.to_string_lossy())).italics());
                // }
                if let Some(message) = &self.message {
                    let error_message = RichText::new(fl!("app_error_err", arg!("err", message)))
                        .strong()
                        .background_color(ERROR_BACKGROUND)
                        .color(ERROR_FOREGROUND); // todo: check colors with theme
                    ui.label(error_message);
                    // todo: we need an "OK" button to clear the message?
                    ui.add_space(ERROR_SPACE);
                    if ui
                        .button(RichText::new(fl!("acknowledge_error")).color(ERROR_FOREGROUND))
                        .clicked()
                    {
                        self.message = None;
                    }
                }
            });
        });
    }

    fn no_data(&mut self, ui: &mut Ui) -> Option<AppStatus> {
        let mut ret = None;
        let mut child = None;
        let mut message = None;
        let mut data = None;
        // there might be several states that appear in here
        // or maybe there are some states that don't care about data either way?

        ui.horizontal(|ui| {
            ui.add_space(EDGE_COLUMN_WIDTH);

            ui.vertical(|ui| {
                match &self.status {
                    AppStatus::Ready => {
                        ui.add_space(16.0);
                        ui.label(fl!("no_data"));
                        ui.add_space(16.0);

                        ui.indent("buttons", |ui| {
                            ui.horizontal(|ui| {
                                let create_text = RichText::from(fl!("make_new"));
                                if ui.button(create_text).clicked() {
                                    // select master language
                                    // select zip or directories
                                    // create directory

                                    let create_data = CreateData {
                                        primary_language: self.settings.master_language(),
                                        load_translations: true, // load this from somewhere?
                                    };
                                    ret = Some(AppStatus::CreateNew(RefCell::new(
                                        CreateStage::Setup(create_data),
                                    )));
                                }

                                ui.add_space(10.0);
                                let load_text = RichText::from(fl!("load_existing"));
                                if ui.button(load_text).clicked() {
                                    // select project file
                                    // load file
                                    ret = Some(AppStatus::Load);
                                }
                            })
                        });
                    }

                    AppStatus::CreateNew(stage_cell) => {
                        // let mut stage = stage_cell.borrow_mut();
                        (child, ret, message, data) =
                            self.show_create(&mut stage_cell.borrow_mut(), ui);
                    }

                    _ => {
                        ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                            ui.add_space(80.);
                            ui.add(Spinner::default().size(50.));
                        });
                    }
                }
            });
        });

        match child {
            Some(FileDialogType::LoadMasterZip) => {
                self.child_windows.start_file_dialog(
                    FileDialogType::LoadMasterZip,
                    FileTarget::MasterZip,
                    None,
                );
            }

            Some(FileDialogType::LoadMasterDir) => {
                self.child_windows.start_file_dialog(
                    FileDialogType::LoadMasterDir,
                    FileTarget::MasterDir,
                    None,
                );
            }

            None => (),

            _ => {
                warn!("unhandled no_data return!");
            }
        }

        if message.is_some() {
            self.message = message;
        }

        if data.is_some() {
            self.data = data;
        }

        ret
    }

    fn show_create(
        &self,
        stage: &mut RefMut<CreateStage>,
        ui: &mut Ui,
    ) -> (
        Option<FileDialogType>,
        Option<AppStatus>,
        Option<String>,
        Option<Dictionary>,
    ) {
        let mut new_status = None;
        let mut child_needed = None;
        let mut new_stage = None;
        let mut message = None;
        let mut data = None;

        ui.vertical(|ui| {
            let extras_heading = RichText::from(fl!("create_heading")).heading();
            ui.label(extras_heading);

            ui.add_space(16.0);

            let extras_description = RichText::from(fl!("create_description"));
            ui.label(extras_description);

            ui.add_space(20.0);

            ui.horizontal(|ui| {
                ui.add_space(INDENT_COLUMN_WIDTH);

                ui.vertical(|ui| {
                    match **stage {
                        CreateStage::Setup(ref mut create_data) => {
                            ui.horizontal(|ui| {
                                // default master language
                                let language = create_data.primary_language;
                                let master_language_label =
                                    RichText::from(fl!("master_language_label")).strong();
                                ui.label(master_language_label);

                                let mut selected = language.language_index();
                                let before = selected;
                                select_language(&mut selected, ui);

                                if selected != before {
                                    // Handle selection change
                                    if let Some(new_language) = Languages::from_index(selected) {
                                        (*create_data).primary_language = new_language;
                                        info!("selected primary language: {new_language}");
                                        // go to next stage
                                    } else {
                                        warn!("unable to set language {selected}");
                                    }
                                }
                            });

                            ui.add_space(BETWEEN_FIELDS);

                            let load_translations_label =
                                RichText::from(fl!("create_load_translations")).strong();

                            let mut load_trans = create_data.load_translations;
                            let pre_load = load_trans;
                            ui.checkbox(&mut load_trans, load_translations_label);

                            if load_trans != pre_load {
                                (*create_data).load_translations = load_trans;
                                info!("load translations set to: {}", load_trans);
                            }

                            ui.add_space(20.0);

                            ui.horizontal(|ui| {
                                let import_zip_text = RichText::from(fl!("menu_new_zip"));
                                if ui.button(import_zip_text).clicked() {
                                    // request load dialog
                                    info!("requesting new from zip dialog");
                                    child_needed = Some(FileDialogType::LoadMasterZip);

                                    info!("CreateStage Setup => SelectZip");
                                    new_stage = Some(CreateStage::SelectZip(*create_data));
                                }

                                ui.add_space(10.0);

                                let import_dir_text = RichText::from(fl!("menu_new_dir"));
                                if ui.button(import_dir_text).clicked() {
                                    // request load dialog
                                    info!("requesting new from directory dialog");
                                    child_needed = Some(FileDialogType::LoadMasterDir);

                                    info!("CreateStage Setup => SelectDirectory");
                                    new_stage = Some(CreateStage::SelectDirectory(*create_data));
                                }
                            });
                        }

                        CreateStage::SelectZip(create_data) => {
                            if let Some(selected) = self.child_windows.selected_file() {
                                if !selected.as_os_str().is_empty() {
                                    // checks for blank file selected, indicating cancel
                                    // process file
                                    let filename = selected.to_string_lossy().to_string();
                                    // let remote_filename = filename.clone();
                                    info!("selected file: {filename}");

                                    // !! check the zip is valid?

                                    info!("CreateStage SelectZip => Create");
                                    let loader = Loader::zip_loader(PathBuf::from(filename));
                                    new_stage = Some(CreateStage::Create(create_data, loader));
                                } else {
                                    // blank file means we want to cancel
                                    // go back to MasterLanguage
                                    info!("CreateStage SelectZip => Setup");
                                    new_stage = Some(CreateStage::Setup(create_data));
                                }
                            }
                        }

                        CreateStage::SelectDirectory(create_data) => {
                            if let Some(selected) = self.child_windows.selected_file() {
                                if !selected.as_os_str().is_empty() {
                                    // checks for blank file selected, indicating cancel
                                    // process file
                                    let filename = selected.to_string_lossy().to_string();
                                    // let remote_filename = filename.clone();
                                    info!("selected directory: {filename}");

                                    // !! Check the directory is valid?

                                    info!("CreateStage SelectDirectory => Create");
                                    let loader = Loader::dir_loader(PathBuf::from(filename));
                                    new_stage = Some(CreateStage::Create(create_data, loader));
                                } else {
                                    // blank file means we want to cancel
                                    // go back to MasterLanguage
                                    info!("CreateStage SelectDirectory => Setup");
                                    new_stage = Some(CreateStage::Setup(create_data));
                                }
                            }
                        }

                        CreateStage::Create(create_data, ref loader) => {
                            ui.request_repaint();
                            let respond = create_dictionary_thread(&create_data, loader);
                            info!("CreateStage Create => WaitLoad");
                            new_stage = Some(CreateStage::WaitLoad(respond, create_data));
                        }

                        CreateStage::WaitLoad(ref load_result, create_data) => {
                            ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                                ui.add_space(80.);
                                ui.add(Spinner::default().size(50.));
                            });
                            ui.request_repaint();

                            if let Ok(response) = load_result.try_recv() {
                                debug!("got load response");
                                match response {
                                    Ok(dictionary) => {
                                        info!("Dictionary created");
                                        data = Some(dictionary);
                                    }

                                    Err(e) => {
                                        error!("unable to create Dictionary: {e}");
                                        message = Some(format!("Unable to create Dictionary: {e}"));
                                        info!("CreateStage Create => Setup");
                                        new_stage = Some(CreateStage::Setup(create_data));
                                    }
                                }

                                info!("WaitLoad => Ready");
                                new_status = Some(AppStatus::Ready);
                            }
                        } // _ => {
                          //     warn!("create stage not implemented, aborting!");
                          //     new_status = Some(AppStatus::Ready); // drop back to Ready
                          // }
                    }
                });
            });
        });

        if let Some(new_stage) = new_stage {
            **stage = new_stage;
        }

        (child_needed, new_status, message, data)
    }

    // fn extras_mode(&mut self, ui: &mut Ui) -> Option<AppStatus> {
    //     let mut ret = None;

    //     match &self.status {
    //         AppStatus::Ready => {
    //             ui.horizontal(|ui| {
    //                 ui.add_space(EDGE_COLUMN_WIDTH);

    //                 ui.vertical(|ui| {
    //                     let extras_heading = RichText::from(fl!("extras_heading")).heading();
    //                     ui.label(extras_heading);

    //                     ui.add_space(16.0);

    //                     let extras_description = RichText::from(fl!("extras_description"));
    //                     ui.label(extras_description);

    //                     // load master language?
    //                     //

    //                     // load translation(s) from xlsx inside zip
    //                     //
    //                     // show button to import

    //                     ui.add_space(20.0);

    //                     if let Some(data) = &self.data {
    //                         let master_name = data.master_language_name();
    //                         ui.label(format!("{}: {master_name}", fl!("master_language_label")));

    //                         ui.indent("loaded_languages", |ui| {
    //                             for l in data.list_translations() {
    //                                 ui.label(l.to_string());
    //                             }
    //                         });

    //                         ui.add_space(20.0);

    //                         let import_text = RichText::from(fl!("menu_import"));
    //                         if ui.button(import_text).clicked() {
    //                             // request load dialog
    //                             info!("requesting import dialog");
    //                             self.child_windows.start_file_dialog(
    //                                 FileDialogType::ImportZip,
    //                                 FileTarget::ImportZip,
    //                                 None,
    //                             );

    //                             info!("Ready => ImportSelect");
    //                             ret = Some(AppStatus::ImportSelect);
    //                         }

    //                         if !data.translations_empty() {
    //                             // create package (make vrz files, embed in zip)

    //                             // show button to export
    //                             let export_text = RichText::from(fl!("extras_export"));
    //                             if ui.button(export_text).clicked() {
    //                                 // request load dialog
    //                                 info!("requesting export dialog");
    //                                 self.child_windows.start_file_dialog(
    //                                     FileDialogType::ExportZip,
    //                                     FileTarget::ExternalsZip,
    //                                     None,
    //                                 );

    //                                 info!("Ready => ExportSelect");
    //                                 ret = Some(AppStatus::ExportSelect);
    //                             }
    //                         }
    //                     }

    //                     // ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
    //                     //     ui.add_space(80.);
    //                     //     ui.add(Spinner::default().size(50.));
    //                     // });
    //                 });
    //             });
    //         }

    //         AppStatus::ExportSelect => {
    //             ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
    //                 ui.add_space(80.);
    //                 ui.add(Spinner::default().size(50.));
    //             });

    //             if let Some(selected) = self.child_windows.selected_file() {
    //                 if !selected.as_os_str().is_empty() {
    //                     // checks for blank file selected, indicating cancel
    //                     // process file
    //                     let filename = selected.to_string_lossy().to_string();
    //                     let remote_filename = filename.clone();
    //                     // info!("selected file: {filename}");
    //                     //

    //                     if let Some(data) = &self.data {
    //                         let category_tags: CategoryTags =
    //                             [Category::Presence, Category::Achievements]
    //                                 .iter()
    //                                 .map(|cat| (*cat, data.tags().get(cat).cloned()))
    //                                 .collect();
    //                         let (send, recv) = bounded(1);
    //                         let respond = recv.to_owned();
    //                         let translations = category_tags
    //                             .iter()
    //                             .map(|(cat, _cat_string)| {
    //                                 let trans = data
    //                                     .category_translations(cat)
    //                                     .iter()
    //                                     .map(|&t| t.clone())
    //                                     .collect::<Vec<Translation>>(); // should have a limited list
    //                                 (*cat, trans)
    //                             })
    //                             .collect();

    //                         let zipfilename = PathBuf::from(remote_filename.clone());
    //                         let _exporter = thread::spawn(move || {
    //                             // let internal_directory = "i18n";
    //                             match create_externals_zip(
    //                                 &zipfilename,
    //                                 &category_tags,
    //                                 translations,
    //                             ) {
    //                                 Err(e) => {
    //                                     error!(
    //                                         "unable to write zip file [{remote_filename}] for externals export: {e}"
    //                                     );
    //                                     send.send(Err(e)).expect("unable to send export result");
    //                                 }

    //                                 Ok(..) => match send.send(Ok(())) {
    //                                     Err(e) => {
    //                                         error!("unable to send export result: {e}");
    //                                     }

    //                                     _ => (),
    //                                 },
    //                             }
    //                             drop(send);
    //                         });

    //                         info!("ExportSelect => Export({filename})");
    //                         ret = Some(AppStatus::Export(RefCell::new(
    //                             LoadState::ExportingPackage(respond),
    //                         )));

    //                         // if let Some(tags) = self.data.tags().get(&Category::Main) {
    //                         //     let tags = tags.clone();
    //                         // } else {
    //                         //     // no tags
    //                         // }
    //                     } else {
    //                         error!("No dictionary present, when expected!");
    //                     }
    //                 } else {
    //                     info!("no export file selected - ignoring");
    //                     info!("ExportSelect => Ready");
    //                     ret = Some(AppStatus::Ready);
    //                 }
    //             }
    //         }

    //         AppStatus::Export(load_status) => {
    //             ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
    //                 ui.add_space(80.);
    //                 ui.add(Spinner::default().size(50.));
    //             });
    //             // wait for export to complete

    //             let l_state = load_status.borrow().clone();
    //             match l_state {
    //                 LoadState::ExportingPackage(recv) => {
    //                     if let Ok(resp) = recv.try_recv() {
    //                         debug!("got load response");
    //                         match resp {
    //                             Ok(()) => {
    //                                 info!("export package completed");
    //                             }
    //                             Err(e) => {
    //                                 error!("unable to export externals: {e}");
    //                                 self.message = Some(format!("Unable to export externals: {e}"));
    //                             }
    //                         }

    //                         info!("Export => Ready");
    //                         ret = Some(AppStatus::Ready);
    //                     }
    //                 }

    //                 _ => {
    //                     error!("unexpected export state: {:?}", l_state);
    //                     info!("Export => Ready");
    //                     ret = Some(AppStatus::Ready);
    //                 }
    //             }
    //         }

    //         _ => (),
    //     }
    //     ret
    // }

    // fn package_mode(&mut self, ui: &mut Ui) -> Option<AppStatus> {
    //     let mut ret = None;

    //     match &self.status {
    //         AppStatus::MasterSelect => {
    //             ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
    //                 ui.add_space(80.);
    //                 ui.add(Spinner::default().size(50.));
    //             });

    //             if let Some(selected) = self.child_windows.selected_file() {
    //                 if !selected.as_os_str().is_empty() {
    //                     // checks for blank file selected, indicating cancel
    //                     // process file
    //                     let filename = selected.to_string_lossy().to_string();
    //                     // info!("selected file: {filename}");
    //                     info!("MasterSelect => Load Master({filename})");
    //                     // ret = Some(AppStatus::Import(RefCell::new(LoadState::Request {
    //                     //     filename,
    //                     // })));
    //                 } else {
    //                     info!("no master file selected - ignoring");
    //                     info!("MasterSelect => Ready");
    //                     ret = Some(AppStatus::Ready);
    //                 }
    //             }
    //         }

    //         AppStatus::MasterLoad => {
    //             ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
    //                 ui.add_space(80.);
    //                 ui.add(Spinner::default().size(50.));
    //             });
    //             // complete load somehow
    //             // into dictionary
    //             info!("MasterLoad => Ready");
    //             ret = Some(AppStatus::Ready);
    //         }

    //         AppStatus::ImportSelect => {
    //             ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
    //                 ui.add_space(80.);
    //                 ui.add(Spinner::default().size(50.));
    //             });

    //             if let Some(selected) = self.child_windows.selected_file() {
    //                 if !selected.as_os_str().is_empty() {
    //                     // checks for blank file selected, indicating cancel
    //                     // process file
    //                     let filename = selected.to_string_lossy().to_string();
    //                     let remote_filename = filename.clone();
    //                     // info!("selected file: {filename}");
    //                     //

    //                     // load zip
    //                     // load xlsz files

    //                     if let Some(data) = &self.data {
    //                         let master_name = data.master_language_name();
    //                         let tags = data.tags().clone();
    //                         let (send, recv) = bounded(1);
    //                         let respond = recv.to_owned();
    //                         let _loader = thread::spawn(move || {
    //                             let mut trans_map: HashMap<Category, Vec<Translation>> =
    //                                 HashMap::new();
    //                             let filepath = PathBuf::from(remote_filename.clone());

    //                             let zfile = open_zip(&filepath);

    //                             match zfile {
    //                                 Err(e) => {
    //                                     error!(
    //                                         "unable to open zip file [{remote_filename}] for translation import: {e}"
    //                                     );
    //                                     send.send(Err(e)).expect("unable to send load result");
    //                                 }

    //                                 Ok(mut zfile) => {
    //                                     for f in zfile.file_names() {
    //                                         debug!("found file: {f}");
    //                                     }

    //                                     let basename = remote_filename
    //                                         .split_terminator('.')
    //                                         .next()
    //                                         .expect("empty zip filename")
    //                                         .rsplit_terminator(['\\', '/'])
    //                                         .next()
    //                                         .expect("unable to remove path");
    //                                     let basepath = PathBuf::from(basename);
    //                                     // self.read_xlsx_from_open_zip(&mut zfile, interior_path)
    //                                     for lang in Languages::all() {
    //                                         let files = lang.external_file_names(&master_name);
    //                                         for (cat, filenm) in files {
    //                                             let mut filepath = basepath.clone();
    //                                             filepath.push(filenm);
    //                                             filepath.set_extension(XLSX_EXT);
    //                                             let fpath = filepath.to_string_lossy();
    //                                             info!("ready to load: {fpath}");
    //                                             // load from zip
    //                                             let mut loader = LoaderOld::default();
    //                                             match loader.read_xlsx_from_open_zip(
    //                                                 &mut zfile,
    //                                                 &filepath.to_string_lossy(),
    //                                             ) {
    //                                                 Ok(_) => {
    //                                                     debug!(
    //                                                         "loaded {fpath} from {remote_filename}"
    //                                                     );

    //                                                     if let Some(tag) = tags.get(&cat) {
    //                                                         let trans = Translation::from_loader(
    //                                                             LanguageCategory::new(lang, cat),
    //                                                             &mut loader,
    //                                                             tag,
    //                                                             Location::External,
    //                                                         );
    //                                                         trans_map
    //                                                             .entry(cat)
    //                                                             .and_modify(|v| {
    //                                                                 v.push(trans.clone())
    //                                                             })
    //                                                             .or_insert(Vec::from([
    //                                                                 trans.clone()
    //                                                             ]));
    //                                                     } else {
    //                                                         warn!(
    //                                                             "no tags found for category {:?}",
    //                                                             cat
    //                                                         );
    //                                                     }
    //                                                 }

    //                                                 Err(e) => {
    //                                                     error!(
    //                                                         "unable to load {fpath} from {remote_filename}: {e}"
    //                                                     );
    //                                                 }
    //                                             }
    //                                         }
    //                                     }
    //                                     match send.send(Ok(trans_map)) {
    //                                         Err(e) => {
    //                                             error!("unable to send load result: {e}");
    //                                         }

    //                                         _ => (),
    //                                     }
    //                                 }
    //                             }
    //                             drop(send);
    //                         });

    //                         info!("ImportSelect => Import({filename})");
    //                         ret = Some(AppStatus::Import(RefCell::new(LoadState::LoadingImports(
    //                             respond,
    //                         ))));
    //                     } else {
    //                         error!("No dictionary when expected!");
    //                     }
    //                 } else {
    //                     info!("no load file selected - ignoring");
    //                     info!("ImportSelect => Ready");
    //                     ret = Some(AppStatus::Ready);
    //                 }
    //             }
    //         }

    //         AppStatus::Import(load_status) => {
    //             ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
    //                 ui.add_space(80.);
    //                 ui.add(Spinner::default().size(50.));
    //             });
    //             // complete load somehow
    //             // into dictionary

    //             let l_state = load_status.borrow().clone();
    //             match l_state {
    //                 LoadState::LoadingImports(recv) => {
    //                     if let Ok(resp) = recv.try_recv() {
    //                         debug!("got load response");
    //                         match resp {
    //                             Ok(cat_map) => {
    //                                 if let Some(data) = self.data.as_mut() {
    //                                     data.add_translations(cat_map);
    //                                     info!("added imported translations");
    //                                 } else {
    //                                     error!("No dictionary when expected!");
    //                                 }
    //                             }
    //                             Err(e) => {
    //                                 error!("unable to load translations: {e}");
    //                                 self.message =
    //                                     Some(format!("Unable to load translations: {e}"));
    //                             }
    //                         }

    //                         info!("Import => Ready");
    //                         ret = Some(AppStatus::Ready);
    //                     }
    //                 }

    //                 _ => {
    //                     error!("unexpected load state: {:?}", l_state);
    //                     info!("Import => Ready");
    //                     ret = Some(AppStatus::Ready);
    //                 }
    //             }
    //         }

    //         AppStatus::ExportSelect => {
    //             ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
    //                 ui.add_space(80.);
    //                 ui.add(Spinner::default().size(50.));
    //             });

    //             if let Some(selected) = self.child_windows.selected_file() {
    //                 if !selected.as_os_str().is_empty() {
    //                     // checks for blank file selected, indicating cancel
    //                     // process file
    //                     let filename = selected.to_string_lossy().to_string();
    //                     let remote_filename = filename.clone();
    //                     // info!("selected file: {filename}");
    //                     //

    //                     if let Some(data) = &self.data {
    //                         if let Some(tags) = data.tags().get(&Category::Main) {
    //                             let tags = tags.clone();
    //                             let (send, recv) = bounded(1);
    //                             let respond = recv.to_owned();
    //                             let translations = data
    //                                 .main_translations()
    //                                 .iter()
    //                                 .map(|&t| t.clone())
    //                                 .collect(); // should have a limited list
    //                             let zipfilename = PathBuf::from(remote_filename.clone());
    //                             let _exporter = thread::spawn(move || {
    //                                 let internal_directory = "i18n";
    //                                 match create_vrt_zip(
    //                                     &zipfilename,
    //                                     internal_directory,
    //                                     FormatVersion::Version1,
    //                                     &tags,
    //                                     translations,
    //                                 ) {
    //                                     Err(e) => {
    //                                         error!(
    //                                             "unable to write zip file [{remote_filename}] for translation export: {e}"
    //                                         );
    //                                         send.send(Err(e))
    //                                             .expect("unable to send export result");
    //                                     }

    //                                     Ok(..) => match send.send(Ok(())) {
    //                                         Err(e) => {
    //                                             error!("unable to send export result: {e}");
    //                                         }

    //                                         _ => (),
    //                                     },
    //                                 }
    //                                 drop(send);
    //                             });

    //                             info!("ExportSelect => Export({filename})");
    //                             ret = Some(AppStatus::Export(RefCell::new(
    //                                 LoadState::ExportingPackage(respond),
    //                             )));
    //                         } else {
    //                             // no tags
    //                         }
    //                     } else {
    //                         error!("No dictionary when expected!");
    //                     }
    //                 } else {
    //                     info!("no export file selected - ignoring");
    //                     info!("ExportSelect => Ready");
    //                     ret = Some(AppStatus::Ready);
    //                 }
    //             }
    //         }

    //         AppStatus::Export(load_status) => {
    //             ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
    //                 ui.add_space(80.);
    //                 ui.add(Spinner::default().size(50.));
    //             });
    //             // wait for export to complete

    //             let l_state = load_status.borrow().clone();
    //             match l_state {
    //                 LoadState::ExportingPackage(recv) => {
    //                     if let Ok(resp) = recv.try_recv() {
    //                         debug!("got load response");
    //                         match resp {
    //                             Ok(()) => {
    //                                 info!("export package completed");
    //                             }
    //                             Err(e) => {
    //                                 error!("unable to export translations: {e}");
    //                                 self.message =
    //                                     Some(format!("Unable to export translations: {e}"));
    //                             }
    //                         }

    //                         info!("Export => Ready");
    //                         ret = Some(AppStatus::Ready);
    //                     }
    //                 }

    //                 _ => {
    //                     error!("unexpected export state: {:?}", l_state);
    //                     info!("Export => Ready");
    //                     ret = Some(AppStatus::Ready);
    //                 }
    //             }
    //         }

    //         AppStatus::Ready => {
    //             ui.horizontal(|ui| {
    //                 ui.add_space(EDGE_COLUMN_WIDTH);

    //                 ui.vertical(|ui| {
    //                     let package_heading = RichText::from(fl!("package_heading")).heading();
    //                     ui.label(package_heading);

    //                     ui.add_space(16.0);

    //                     let package_description = RichText::from(fl!("package_description"));
    //                     ui.label(package_description);

    //                     // load master language?
    //                     //

    //                     // load translation(s) from xlsx inside zip
    //                     //
    //                     // show button to import

    //                     ui.add_space(20.0);

    //                     if let Some(data) = &self.data {
    //                         let master_name = data.master_language_name();
    //                         ui.label(format!("{}: {master_name}", fl!("master_language_label")));

    //                         ui.indent("loaded_languages", |ui| {
    //                             for l in data.list_translations() {
    //                                 ui.label(l.to_string());
    //                             }
    //                         });

    //                         ui.add_space(20.0);

    //                         let import_text = RichText::from(fl!("menu_import"));
    //                         if ui.button(import_text).clicked() {
    //                             // request load dialog
    //                             info!("requesting import dialog");
    //                             self.child_windows.start_file_dialog(
    //                                 FileDialogType::ImportZip,
    //                                 FileTarget::ImportZip,
    //                                 None,
    //                             );

    //                             info!("Ready => ImportSelect");
    //                             ret = Some(AppStatus::ImportSelect);
    //                         }

    //                         if !data.translations_empty() {
    //                             // create package (make vrz files, embed in zip)

    //                             // show button to export
    //                             let export_text = RichText::from(fl!("package_export"));
    //                             if ui.button(export_text).clicked() {
    //                                 // request load dialog
    //                                 info!("requesting export dialog");
    //                                 self.child_windows.start_file_dialog(
    //                                     FileDialogType::ExportZip,
    //                                     FileTarget::ExportZip,
    //                                     None,
    //                                 );

    //                                 info!("Ready => ExportSelect");
    //                                 ret = Some(AppStatus::ExportSelect);
    //                             }
    //                         }

    //                         // ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
    //                         //     ui.add_space(80.);
    //                         //     ui.add(Spinner::default().size(50.));
    //                         // });
    //                     } else {
    //                         error!("No dictionary when expected!");
    //                     }
    //                 });
    //             });
    //         }

    //         _ => (),
    //     }

    //     ret
    // }
}

/// spawns a thread to create the dictionary and load the relevant translations
fn create_dictionary_thread(
    create_data: &CreateData,
    loader: &Loader,
) -> Receiver<Result<Dictionary>> {
    info!("loading from loader for {}", create_data.primary_language);
    let (mut languages, unused) = loader.list_languages();
    if !languages.remove(&create_data.primary_language) {
        warn!("primary language not found in loader");
    }

    let primary_language = create_data.primary_language;
    let load_translations = create_data.load_translations;
    let loader = loader.clone();
    let (send, recv) = bounded(1);
    let respond = recv.to_owned();
    let _loader = thread::spawn(move || {
        match Dictionary::new(primary_language, &loader) {
            Ok(mut dictionary) => {
                if load_translations {
                    info!("loading existing traslations");
                    for lang in languages {
                        if let Err(e) = dictionary.load_core_translation(lang, &loader) {
                            error!("unable to load translation {lang}: {e}");
                        } else {
                            info!("loaded translation for {lang}");
                        }
                    }
                    for l in unused {
                        warn!("cannot load: {l}");
                    }
                }

                // have dictionary
                match send.send(Ok(dictionary)) {
                    Err(e) => {
                        error!("unable to send create result: {e}");
                    }

                    _ => (),
                }
            }

            Err(e) => {
                error!("unable to make Directory: {e}");
                match send.send(Err(e)) {
                    Err(e) => {
                        error!("unable to send create result: {e}");
                    }

                    _ => (),
                }
            }
        }

        drop(send);
    });

    respond
}

// display functions
// -----------------

// layout constants
pub const EDGE_COLUMN_WIDTH: f32 = 40.0;
pub const INDENT_COLUMN_WIDTH: f32 = 16.0;
pub const BETWEEN_FIELDS: f32 = 8.0;
pub const TINY_SPACE: f32 = 2.0;
pub const SMALL_SPACE: f32 = 5.0;
pub const STRING_WIDTH: f32 = 500.0;
pub const STRING_HEIGHT: f32 = 200.0;
pub const STRING_RECT: Vec2 = Vec2 {
    x: STRING_WIDTH,
    y: STRING_HEIGHT,
};

pub const ACTIVE_COLOR: Color32 = Color32::DARK_GREEN; // should change with theme
pub const MISSING_COLOR: Color32 = Color32::RED;
pub const MOD_MAIN_COLOR: Color32 = Color32::GREEN;
pub const MOD_TRANS_COLOR: Color32 = Color32::DARK_RED;

// ===========================
// AppStatus

#[allow(dead_code, clippy::large_enum_variant)]
#[derive(Default)]
pub enum AppStatus {
    #[default]
    Starting,
    Settings,
    Ready, // (RefCell<Option<usize>>),
    CreateNew(RefCell<CreateStage>),
    // ShowEditDistrict(Option<DistrictRef>, RefCell<District>),
    // ShowEditPerson(Option<PersonRef>, RefCell<Person>),
    // ShowEditFaction(Option<FactionRef>, RefCell<Faction>),
    Load,
    SaveTo, // No file dialog, use existing save file name
    SaveAs, // use file dialog to get file name
            // MasterSelect,
            // MasterLoad,
            // ImportSelect,
            // Import(RefCell<LoadState>),
            // ExportSelect,
            // Export(RefCell<LoadState>),
}

impl Display for AppStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use AppStatus::*;

        write!(
            f,
            "{}",
            match self {
                Starting => fl!("app_starting"),
                Settings => fl!("app_settings"),
                Ready => fl!("app_ready"),
                CreateNew(..) => fl!("app_create_new"),
                // ShowEditDistrict(ind, ..) => {
                //     let item = fl!("main_item_district");
                //     if ind.is_none() {
                //         fl!("app_create_itm", itm = item)
                //     } else {
                //         fl!("app_edit_itm", itm = item)
                //     }
                // }
                // ShowEditPerson(ind, ..) => {
                //     let item = fl!("main_item_person");
                //     if ind.is_none() {
                //         fl!("app_create_itm", itm = item)
                //     } else {
                //         fl!("app_edit_itm", itm = item)
                //     }
                // }
                // ShowEditFaction(ind, ..) => {
                //     let item = fl!("main_item_faction");
                //     if ind.is_none() {
                //         fl!("app_create_itm", itm = item)
                //     } else {
                //         fl!("app_edit_itm", itm = item)
                //     }
                // }
                Load => fl!("app_loading"),
                SaveAs => fl!("app_saving"),
                SaveTo => fl!("app_saving"),
                // MasterSelect => fl!("app_master_selecting"),
                // MasterLoad => fl!("app_master_loading"),
                // Import(..) => fl!("app_importing"),
                // ImportSelect => fl!("app_import_selecting"),
                // Export(..) => fl!("app_exporting"),
                // ExportSelect => fl!("app_export_selecting"),
            }
        )
    }
}

// #[derive(Debug, Clone)]
// pub enum LoadState {
//     // Request { filename: String },
//     Loading(Receiver<Result<()>>), // returns result via message
//     // LoadingImports(Receiver<Result<HashMap<Category, Vec<Translation>>>>), // returns result via message
//     LoadingMaster(Receiver<Result<()>>), // returns result via message
//     ExportingPackage(Receiver<Result<()>>), // is done or is not done
// }

#[derive(Debug, Clone, Copy)]
pub struct CreateData {
    primary_language: Language,
    load_translations: bool,
}

impl Default for CreateData {
    fn default() -> Self {
        CreateData {
            primary_language: Language::default(),
            load_translations: true,
        }
    }
}

#[derive(Debug, Clone)]
pub enum CreateStage {
    Setup(CreateData),
    SelectZip(CreateData),
    SelectDirectory(CreateData),
    Create(CreateData, Loader),
    WaitLoad(Receiver<Result<Dictionary>>, CreateData),
}

// ===========================
// Additional functions

fn configure_fonts(ctx: &CreationContext, _zoom: f32) {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "base".to_string(),
        Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/Noto_Sans/static/NotoSans-Regular.ttf"
        ))),
    );

    Languages::add_language_fonts(&mut fonts);

    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "base".to_owned());

    ctx.egui_ctx.set_fonts(fonts);

    // Redefine text_styles
    use FontFamily::*;
    use TextStyle::*;
    let text_styles: BTreeMap<_, _> = [
        (Heading, FontId::new(20.0, Proportional)),
        // (Name("Heading2".into()), FontId::new(25.0, Proportional)),
        // (Name("Context".into()), FontId::new(23.0, Proportional)),
        (Body, FontId::new(14.0, Proportional)),
        (TextStyle::Monospace, FontId::new(13.0, Proportional)),
        (Button, FontId::new(13.0, Proportional)),
        (Small, FontId::new(12.0, Proportional)),
    ]
    .into();

    // Mutate global styles with new text styles
    ctx.egui_ctx
        .all_styles_mut(move |style| style.text_styles = text_styles.clone());

    // ctx.egui_ctx.set_zoom_factor(zoom);
}
