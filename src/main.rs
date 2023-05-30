#![feature(panic_info_message)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] use std::{fs::File, io};

// hide console window on Windows in release
use eframe::egui;
use egui::Color32;
use egui_extras::{TableBuilder, Column};
use serde_json::{Result, Value};
use rfd::FileDialog;

struct SongData {
    song_name: String,
    state: String,
    level_name: String,
    download_link: String,
    song_id: String
}

enum Status {
    Waiting,
    Downloading,
    Finished,
    NoResults,
    ResultsFound
}

fn setup_panic_hook() {
    std::panic::set_hook(Box::new(|panic_info| {
        let err = panic_info.payload().downcast_ref::<&str>();
        let location = panic_info.location();

        let mut error = "Unknown";
        let mut file = "Unknown";
        let mut line: String = "Unknown".to_string();
        let mut column: String = "Unknown".to_string();

        if err.is_some() {
            error = err.unwrap();
        }

        if location.is_some() {
            file = location.unwrap().file();
            line = location.unwrap().line().to_string();
            column = location.unwrap().column().to_string();
        }

        let error_message = format!
        ("Error: {}.\nMessage: {:?}.\nLocation: {}.\nLine: {}.\nColumn: {}.", 
        error,
        panic_info.message(),
        file, 
        line, 
        column);
        msgbox::create("Error has occured", &error_message, msgbox::IconType::Error).unwrap();
    }));
}

fn main() {
    setup_panic_hook();
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(660.0, 280.0)),
        
        ..Default::default()
    };
    eframe::run_native(
        "NoNG Manager",
        options,
        Box::new(|_cc| Box::<MyApp>::default()),
    ).unwrap();
}

struct MyApp {
    search_query: String,
    songs: Vec<SongData>,
    settings: bool,
    song_path: String,
    status: Status
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            search_query: "".to_owned(),
            songs: vec![],
            settings: false,
            song_path: format!("C:\\Users\\{}\\AppData\\Local\\GeometryDash", whoami::username()).to_owned(),
            status: Status::Waiting
        }
    }

}

fn download(download_link: String, song_path: String, song_id: String) {
    println!("{}", download_link);
    let mut resp = reqwest::blocking::get(download_link).expect("request failed");
    let mut out = File::create(song_path + "\\" + &song_id + ".mp3").expect("failed to create file");
    io::copy(&mut resp, &mut out).expect("failed to copy content");
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.search_query);
                if ui.button("Search").clicked() {
                    self.songs.clear();
                    let resp = reqwest::blocking::get(format!("https://songfilehub.com/api/v1/nongs?id={}", self.search_query)).unwrap().text().unwrap();
                    let v: Value = serde_json::from_str(&resp.to_owned()).unwrap();
                    let a = v["songs"].as_array().unwrap();
                    for x in a {
                        self.songs.push({
                            SongData { song_name: x["songName"].as_str().unwrap().to_string(), 
                            state: x["state"].as_str().unwrap().to_string(), 
                            level_name: x["name"].as_str().unwrap().to_string(), 
                            download_link: x["downloadUrl"].as_str().unwrap().to_string(),
                            song_id: x["songID"].as_str().unwrap().to_string()
                            }
                        });
                    }
                    if self.songs.len() == 0 {
                        self.status = Status::NoResults;
                    } else {
                        self.status = Status::ResultsFound;
                    }
                }

                if ui.button("Settings").clicked() {
                    self.settings = true;
                }

                match self.status {
                    Status::Downloading=>ui.label("Downloading..."),
                    Status::Finished=>ui.label("Finished"),
                    Status::NoResults=>ui.label("Nothing found"),
                    Status::ResultsFound=>ui.label(format!("Found {} NoNG(s) with matching song ID", self.songs.len())),
                    Status::Waiting=>ui.label("Waiting")
                }
            });
            ui.separator();

            let mut table = TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::remainder())
                .column(Column::remainder())
                .column(Column::remainder())
                .column(Column::remainder())
                .min_scrolled_height(0.0);
            table
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong("Song name");
                });
                header.col(|ui| {
                    ui.strong("Level name");
                });
                header.col(|ui| {
                    ui.strong("State");
                });
                header.col(|ui| {
                    ui.strong("Download");
                });
            })
            .body(|mut body|{
                let a = &self.songs;
                for x in a.iter() {
                    body.row(20.0, |mut row| {
                        row.col(|ui| {
                            ui.label(format!("{}", x.song_name));
                        });
                        row.col(|ui| {
                            ui.label(format!("{}", x.level_name));
                        });
                        row.col(|ui| {
                            ui.label(format!("{}", x.state));
                        });
                        row.col(|ui| {
                            if ui.button("Download").clicked() {
                                self.status = Status::Downloading;
                                download(x.download_link.to_owned(), self.song_path.to_owned(), x.song_id.to_owned());
                                self.status = Status::Finished;
                            };
                        });
                    });
                }
            });
        });

        if self.settings {
            egui::Window::new("Settings")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {

                ui.horizontal(|ui| {
                    let song_path_label = ui.label("Song path");
                    ui.text_edit_singleline(&mut self.song_path).labelled_by(song_path_label.id);
                    if ui.button("Browse").clicked() {
                        self.song_path = FileDialog::new().pick_folder().unwrap_or(self.song_path.to_owned().into()).as_path().display().to_string();
                    }
                });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.heading("NoNG Manager");
                    ui.strong("by Alexander Simonov");
                });

                ui.label("Version: 1.0.1");

                ui.horizontal(|ui| {
                    ui.label("Powered by");
                    ui.hyperlink_to("Song File Hub", "https://songfilehub.com");
                });

                ui.hyperlink_to("Feedback, support and contribution", "https://github.com/adarift/nong-manager");

                ui.separator();

                if ui.button("Close").clicked() {
                    self.settings = false;
                }
            });
        }


    }
}