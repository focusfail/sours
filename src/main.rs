#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui::{self, Color32};
use sours::{winapi_, ytdlp, AudioResource};
use sours::{AudioPlayer, Options};
use std::time::Duration;

fn main() {
    // Load options from JSON
    let options = Options::load_from_json("sours.json");

    // Setup window options
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(options.ui_size.clone())
            .with_resizable(true)
            .with_max_inner_size(egui::Vec2::new(800.0, 550.0))
            .with_min_inner_size(egui::Vec2::new(260.0, 200.0))
            .with_maximize_button(false),
        ..Default::default()
    };

    // Run the application
    let _ = eframe::run_native(
        "sours",
        native_options,
        Box::new(|cc| Box::new(App::new(cc, options))),
    );
}

#[derive(Debug, Default)]
struct State {
    yt_url: String,
    multiselect: Vec<AudioResource>,
    downloader: ytdlp::Downloader,
}

struct App {
    options: Options,
    player: AudioPlayer,
    state: State,
}

impl App {
    fn new(_cc: &eframe::CreationContext<'_>, options: Options) -> Self {
        // Create A player for playback and timing
        let mut player = AudioPlayer::default();

        // Change the players volume to the last saved volume
        player.set_volume_100(options.volume);

        // Set window to always be on top if configured
        if options.always_on_top {
            winapi_::set_window_always_on_top("sours", true);
        }

        Self {
            options,
            player,
            state: State::default(),
        }
    }
    // Display the playlist
    fn playlist(&mut self, ui: &mut egui::Ui) {
        let frame = egui::Frame::default().fill(Color32::from_rgb(35, 35, 35));
        let scroll = egui::ScrollArea::both().max_height(ui.available_height());

        frame.show(ui, |ui| {
            scroll.show(ui, |ui| {
                let playlist_layout = egui::Layout::top_down_justified(egui::Align::LEFT);
                ui.with_layout(playlist_layout, |ui| {
                    let playlist_slice = self.options.playlist.clone();

                    // Iterate through all saved `AudioResaource`'s
                    for (i, resource) in playlist_slice.iter().enumerate() {
                        let checked;

                        // Check if the resource is selected
                        if let Some(selected) = &self.options.selected {
                            if selected == resource {
                                checked = true;
                            } else {
                                checked = false;
                            }
                        } else {
                            checked = false;
                        }

                        // Alternate row colors
                        let mut fill = match i % 2 {
                            0 => Color32::from_rgb(42, 42, 45),
                            _ => Color32::from_rgb(35, 35, 35),
                        };

                        // Different row color if AudioResource is currentlu playing
                        if let Some(current) = &self.player.current {
                            if current == resource {
                                fill = Color32::from_rgb(50, 55, 77);
                            }
                        }

                        let audio_basename =
                            String::from(resource.path.file_name().unwrap().to_string_lossy());

                        // Use `RchText` to allow for red text if the resource is unavailable
                        let mut display_text = egui::RichText::new(audio_basename);
                        if !resource.playable() {
                            display_text = display_text.color(Color32::RED);
                        }

                        egui::Frame::default().fill(fill).show(ui, |ui| {
                            // Add the `AudioResource` to the playlist display as `SelectableLabel`
                            let re = ui.add(egui::SelectableLabel::new(
                                checked && resource.playable(),
                                display_text,
                            ));

                            // If the label is clicked and is playabel ->
                            // set the currently selected resource
                            if re.clicked() && resource.playable() {
                                self.options.selected = Some(resource.clone());
                            }

                            // If double clicked play currently selected resource
                            // Currently selected resource can logically only be the one doubleclicked
                            if re.double_clicked() && resource.playable() {
                                self.player.play(resource.clone());
                            }

                            // Save the RightClickMenu's previous state
                            let mut show_ctx = re.context_menu_opened();

                            // If rightclicked toggle the RightClickMenu
                            if re.clicked_by(egui::PointerButton::Secondary) {
                                show_ctx = !re.context_menu_opened();
                            }

                            if show_ctx {
                                re.context_menu(|ui| {
                                    let play_button = egui::Button::new("Play");

                                    // Add play-button to ctxmenu if the resource is playable
                                    if ui.add_enabled(resource.playable(), play_button).clicked() {
                                        self.options.selected = Some(resource.clone());
                                        self.player.play(resource.clone());
                                        show_ctx = false;
                                        ui.close_menu();
                                    }

                                    // Add open-button that reveals the resource's file in the explorer
                                    if ui
                                        .add_enabled(resource.playable(), egui::Button::new("Open"))
                                        .clicked()
                                    {
                                        let path =
                                            std::fs::canonicalize(&resource.path.parent().unwrap())
                                                .unwrap();
                                        let pathstr = path.to_str().unwrap().replace("\\\\?\\", "");

                                        let _ = std::process::Command::new("explorer.exe")
                                            .arg(&pathstr)
                                            .output()
                                            .unwrap();
                                    }
                                    // Add remove-button that removes the resource from the playlist
                                    if ui
                                        .button(egui::RichText::new("Remove").color(Color32::RED))
                                        .clicked()
                                    {
                                        self.options.remove_resource(resource);
                                        if let Some(selected) = &self.options.selected {
                                            if selected == resource {
                                                self.options.selected = None;
                                            }
                                        }
                                        if let Some(current) = &self.player.current {
                                            if current == resource {
                                                self.player.stop();
                                            }
                                        }
                                        show_ctx = false;
                                        ui.close_menu();
                                    }
                                });
                            }

                            // todo: Change to make `Del`, delete currently selected
                            // If `Del` is pressed delete all multiselected resources
                            if ui.ctx().input(|i| i.key_pressed(egui::Key::Delete)) {
                                if self.options.selected.is_some()
                                    && !self.state.multiselect.is_empty()
                                {
                                    for res in &self.state.multiselect {
                                        self.options.remove_resource(res);
                                        if let Some(selected) = &self.options.selected {
                                            if selected == resource {
                                                self.options.selected = None;
                                            }
                                        }
                                        if let Some(current) = &self.player.current {
                                            if current == resource {
                                                self.player.stop();
                                            }
                                        }
                                    }
                                }
                            }
                        });
                    }
                });
            });
        });
    }
    fn open_sours_json(&self) {
        /*
           Superflous function for opening `sours.json`
           in window's default app for JSON files
        */

        let cwd = std::env::current_dir().unwrap();
        let json_path = cwd.join(std::path::Path::new("sours.json"));
        let pathstr = json_path.to_str().unwrap();

        winapi_::open_file_in_default_application(pathstr);
    }
    fn _handle_multiselect(&mut self) {}
    fn _set_selected(&mut self) {}
    fn ask_open_file(&mut self) {
        /*
           Open a file-dialog and ask to select audio files
           add chosen files to playlist
        */
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("audio", &["wav", "mp3"])
            .pick_file()
        {
            if !self.options.playlist.iter().any(|x| x.path == path) {
                self.options.add_resource(path);
            }
        }
    }
    fn consume_keyboard_shortcuts(&mut self, ui: &mut egui::Ui) {
        /*
            Initiate `ctrl + o` as keyboard shortcut to open files
        */

        const CTRL_O_SHORTCUT: egui::KeyboardShortcut =
            egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::O);

        ui.input_mut(|i| {
            if i.consume_shortcut(&CTRL_O_SHORTCUT) {
                self.ask_open_file();
            }
        });
    }
    fn menu(&mut self, ctx: &egui::Context) {
        /*
            Display TopBar-Menu
        */
        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            self.consume_keyboard_shortcuts(ui);
            egui::menu::bar(ui, |ui| {
                // Media menu category
                ui.menu_button("Media", |ui| {
                    ui.horizontal(|ui| {
                        // Open file-dialog
                        if ui.button("Open").clicked() {
                            self.ask_open_file();
                        };
                        ui.add_space(5.0);
                        ui.separator();

                        // Label displaying the keyboard shortcut
                        ui.label(egui::RichText::new("Ctrl + O").monospace().size(10.0));
                    });

                    // Youtube download menu

                    ui.add_enabled_ui(self.state.downloader.is_finished(), |ui| {
                        ui.menu_button("From Youtube", |ui| {
                            // `n` maximum playlist downloads
                            let n = 10;
                            ui.label("Enter Youtube URL:");
                            let entry = ui
                                .text_edit_singleline(&mut self.state.yt_url)
                                .on_hover_text("Enter Youtube URL");
                            let button_download = ui.button("Download");

                            // If `Enter` or the download button were pressed, downloda the video
                            if (entry.lost_focus() || button_download.clicked())
                                && !self.state.yt_url.is_empty()
                            {
                                self.state.downloader.download(self.state.yt_url.clone(), n);
                                self.state.yt_url.clear();
                            }
                        });
                    });

                    ui.separator();
                });

                // Playback menu category
                ui.menu_button("Playback", |ui| {
                    let play = egui::Button::new("Play");
                    let pause = egui::Button::new("Pause");
                    let stop = egui::Button::new("Stop");

                    // Show play-button
                    if ui.add_enabled(!self.player.is_playing(), play).clicked() {
                        // Play selected if Some
                        if let Some(resource) = self.options.selected.clone() {
                            self.player.play(resource);
                        }
                    }
                    // Pause-button
                    if ui.add_enabled(self.player.is_playing(), pause).clicked() {
                        self.player.pause();
                    };
                    // Stop-button
                    if ui.add(stop).clicked() {
                        self.player.stop();
                    };
                    ui.separator();

                    // Autoplay checkbox
                    ui.checkbox(&mut self.options.autoplay, "Autoplay");
                });

                // Playlist menu category
                ui.menu_button("Playlist", |ui| {
                    // If playlist is not empty show shuffle button
                    if ui
                        .add_enabled(
                            !self.options.playlist.is_empty(),
                            egui::Button::new("🔀 Shuffle"),
                        )
                        .clicked()
                    {
                        // Shuffle playlist
                        self.options.shuffle();
                    }
                    // If something is selected show remove button
                    if ui
                        .add_enabled(
                            self.options.selected.is_some(),
                            egui::Button::new("Remove Selected"),
                        )
                        .clicked()
                    {
                        // Remove selected
                        self.options
                            .playlist
                            .retain(|r| r != self.options.selected.as_ref().unwrap());

                        if self.options.selected == self.player.current {
                            self.player.stop();
                        }

                        /*
                            If another resource was playing, set the next selected
                            resource after the removed one to the playing one
                        */
                        self.options.selected = self.player.current.clone();
                    }

                    if ui
                        .add_enabled(
                            self.options.selected.is_some(),
                            egui::Button::new("Move Up"),
                        )
                        .clicked()
                    {
                        // move the selected resource up
                        let index = self
                            .options
                            .playlist
                            .iter()
                            .position(|r| r == self.options.selected.as_ref().unwrap())
                            .unwrap();

                        let new_index = if index > 0 {
                            index - 1
                        } else {
                            (index + self.options.playlist.len() - 1) % self.options.playlist.len()
                        };
                        self.options.playlist.swap(index, new_index as usize);
                    }
                    if ui
                        .add_enabled(
                            self.options.selected.is_some(),
                            egui::Button::new("Move Down"),
                        )
                        .clicked()
                    {
                        // move the selected resource down
                        let index = self
                            .options
                            .playlist
                            .iter()
                            .position(|r| r == self.options.selected.as_ref().unwrap())
                            .unwrap();

                        let new_index = (index + 1) % self.options.playlist.len();
                        self.options.playlist.swap(index, new_index);
                    }
                    // Clear button to clear the whole playlist
                    // todo: prompt the user to confirm deletion
                    if ui
                        .add(egui::Button::new(
                            egui::RichText::new("Clear").color(Color32::RED),
                        ))
                        .clicked()
                    {
                        self.options.playlist_clear();
                        self.player.stop();
                        self.options.selected = None;
                    }
                });

                // Debug menu category
                ui.menu_button("Debug", |ui| {
                    let before_aot = self.options.always_on_top.clone();
                    ui.checkbox(&mut self.options.always_on_top, "Always on Top");
                    ui.checkbox(&mut self.options.show_debug, "Debug Menu");

                    if before_aot != self.options.always_on_top {
                        sours::winapi_::set_window_always_on_top(
                            "sours",
                            self.options.always_on_top,
                        );
                    }

                    if ui.button("Open sours.json").clicked() {
                        self.open_sours_json();
                    }

                    if ui.button("Select none").clicked() {
                        self.options.selected = None;
                    }
                    ui.separator();

                    ui.menu_button("Info", |ui| ui.label("Version"));
                });

                // Youtube download indicator in the topmenubar
                if !self.state.downloader.is_finished() {
                    ui.monospace("Downloading ");
                    ui.spinner();
                }
            })
        });
    }
    fn debug_window(&mut self, ctx: &egui::Context) {
        /*
            Floating window displaying information about the app's state
        */
        if self.options.show_debug {
            egui::Window::new("Debug Menu").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Selected: ");
                    let index = if let Some(selected) = self.options.selected.as_ref() {
                        self.options
                            .playlist
                            .iter()
                            .position(|r| r == selected)
                            .unwrap()
                    } else {
                        0
                    };

                    ui.label(format!("{:?}", index));
                });
                ui.horizontal(|ui| {
                    ui.label("Playing: ");
                    let mut text = "None";
                    if let Some(resource) = &self.player.current {
                        text = resource.path.file_name().unwrap().to_str().unwrap();
                    }
                    ui.label(format!("{:?}", text));
                });
                ui.horizontal(|ui| {
                    ui.label("Volume: ");
                    ui.label(format!("{:?}", self.options.volume));
                });
                ui.horizontal(|ui| {
                    ui.label("UI Size: ");
                    ui.label(format!("{:?}", self.options.ui_size));
                });
            });
        }
    }
    fn controls(&mut self, ui: &mut egui::Ui) {
        /*
            Media controls like Play/Pause/Stop
        */
        let frame = egui::Frame::default().fill(Color32::from_rgb(35, 35, 35));

        // Controls are only active if a resource is selected to act upon
        ui.add_enabled_ui(self.options.selected.is_some(), |ui| {
            frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    if self.player.is_playing() {
                        // Pause button
                        if ui.button("⏸").clicked() {
                            self.player.pause();
                        }
                    } else {
                        // Play button
                        if ui.button("▶").clicked() {
                            self.player.play(self.options.selected.clone().unwrap());
                        }
                    }

                    // Stop button
                    if ui.button("⏹").clicked() {
                        self.player.stop();
                    }
                });
            });
        });
    }
    fn volume(&mut self, ui: &mut egui::Ui) {
        /*
            Context with volume slider
        */

        // Convert volume from 0.0 - 1.0 range to 0 - 100
        // Format the volume for display
        let vol_label = format!("🔊 {:>3}%", self.options.volume.to_string());

        ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
            egui::menu::menu_button(ui, vol_label, |ui| {
                let slider = egui::Slider::new(&mut self.options.volume, 0..=100)
                    .trailing_fill(true)
                    .show_value(true)
                    .handle_shape(egui::style::HandleShape::Rect {
                        aspect_ratio: (0.2),
                    });
                ui.add(slider);
            });
        });
    }
    fn time(&mut self, ui: &mut egui::Ui) {
        /*
            Display's playing resourc's elapsed/total duration
        */

        // If nothing is playing or the player finished playing,
        // don't show the time
        if self.player.current.is_none() || self.player.just_finished() {
            return;
        }

        // While playing repaint the ui every 250ms instead of on ui-change
        // otherwise the time-display would only update if the window changes
        ui.ctx().request_repaint_after(Duration::from_millis(250));

        // get the currently playing resource's total duration
        let time = match &self.player.current {
            Some(resource) => resource.formatted_duration(),
            None => "00:00".to_string(),
        };

        // get the elapsed time from the player
        let elapsed = self.player.elapsed().unwrap();
        // format the time to MM:SS format
        let elapsed_string = format!(
            "{:02}:{:02}",
            elapsed.as_secs() / 60,
            elapsed.as_secs() % 60
        );

        ui.label(format!("{}/{}", elapsed_string, time));
    }
    fn handle_drop(&mut self, ctx: &egui::Context) {
        /*
            Handle dropped files on window
        */
        if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
            ctx.input(|i| {
                for item in i.raw.hovered_files.iter() {
                    // Add dropped files to playlist
                    self.options.add_resource(item.path.clone().unwrap());
                }
            });
        }
    }
    fn handle_keys(&mut self, ctx: &egui::Context) {
        /*
            Handle key presses
        */
        ctx.input(|input| {
            // Press space to Plau / Pause
            if input.key_pressed(egui::Key::Space) {
                self.play_pause();
            }

            // `alt` + `-/+` to decrease / increase volume
            if input.modifiers.alt && input.key_pressed(egui::Key::Equals) {
                self.options.volume = u8::clamp(self.options.volume + 1, 0, 100);
            }
            if input.modifiers.alt && input.key_pressed(egui::Key::Minus) {
                self.options.volume = i8::clamp(self.options.volume as i8 - 1, 0, 100) as u8;
            }
        });
    }
    fn play_pause(&mut self) {
        /*
            Function that handles logic of playing / pausing
        */

        // Get the resource to play / stop
        let to_play: Option<AudioResource> = {
            if self.player.current.is_some() {
                // If the player has a rsource, play / pause that
                Some(self.player.current.as_ref().unwrap().clone())
            } else if self.options.selected.is_some() {
                // If a rource is selected, play / pause that
                Some(self.options.selected.as_ref().unwrap().clone())
            } else if !self.options.playlist.is_empty() {
                // Else, play the first resource in the playlist
                Some(self.options.playlist[0].clone())
            } else {
                None
            }
        };

        if to_play.is_none() {
            return;
        }

        if self.player.is_playing() {
            self.player.pause();
        } else {
            self.player.play(to_play.unwrap());
        }
    }
    fn central_panel(&mut self, ctx: &egui::Context) {
        /*
            Central panel
        */
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    self.controls(ui);
                    self.time(ui);
                    self.volume(ui);
                });
                ui.add_space(5.0);
                self.playlist(ui);
            });
        });
    }
    fn handle_autoplay(&mut self) {
        if self.options.autoplay && self.player.just_finished() {
            // get the index of the selected resource
            let index = self
                .options
                .playlist
                .iter()
                .position(|r| r == self.options.selected.as_ref().unwrap())
                .unwrap();

            // get the next resource in the playlist
            let next = self
                .options
                .playlist
                .get((index + 1) % self.options.playlist.len());

            // Select and play the next resource
            self.options.selected = next.cloned();
            self.player.play(next.unwrap().clone());
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle events
        self.handle_drop(ctx);
        self.handle_keys(ctx);
        self.handle_autoplay();

        // Render ui elements
        self.menu(ctx);
        self.central_panel(ctx);

        if self.options.show_debug {
            self.debug_window(ctx);
        }

        // if the downloader is finished downloading add the files to the playlist
        // todo: improve keeping track of downloaded resources
        if self.state.downloader.is_finished() {
            self.options.add_downloads();
        }

        //  Change volume if changed in ui
        if self.options.volume != self.player.volume() as u8 * 100 {
            self.player.set_volume_100(self.options.volume);
        }

        // save the windows size to the options
        self.options.ui_size = ctx.screen_rect().max.into();
    }
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.options.save_to_json("sours.json");
    }
}
