use std::path::PathBuf;

use crate::audio_resource::AudioResource;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Options {
    pub playlist: Vec<AudioResource>,
    // pub downloads: Vec<AudioResource>,
    pub autoplay: bool,
    pub selected: Option<AudioResource>,
    pub volume: u8,
    pub ui_size: [f32; 2],
    pub show_debug: bool,
    pub always_on_top: bool,
    // logs: Vec<String>,
}

impl Options {
    pub fn save_to_json(&self, path: &str) {
        let json = serde_json::to_string_pretty(&self).unwrap();
        std::fs::write(path, json).unwrap();
    }

    pub fn load_from_json(path: &str) -> Self {
        let path = std::path::Path::new(path);
        if !path.exists() {
            let options = Options {
                playlist: Vec::new(),
                autoplay: false,
                selected: None,
                volume: 50,
                ui_size: [360.0, 300.0],
                show_debug: false,
                always_on_top: false,
                // downloads: Vec::new(),
                // logs: Vec::new()
            };
            options.save_to_json(path.to_str().unwrap());
        }

        let json = std::fs::read_to_string(path).unwrap();
        serde_json::from_str(&json).unwrap()
    }

    pub fn add_resource(&mut self, path: PathBuf) {
        if path.extension().is_none() {
            return;
        }
        if ["mp3", "wav"].contains(&path.extension().unwrap().to_str().unwrap()) {
            let resource = AudioResource::from_path(path.to_string_lossy().to_string());
            if self.playlist.contains(&resource) {
                return;
            }
            self.playlist.push(resource);
        }
    }

    pub fn remove_resource(&mut self, resource: &AudioResource) {
        let downloaded = std::fs::read_dir("./downloads").unwrap();

        for entry in downloaded {
            let path = entry.unwrap().path();

            if path == resource.path {
                std::fs::remove_file(path).unwrap();
            }
        }

        self.playlist.retain(|r| r != resource);
    }

    pub fn add_downloads(&mut self) {
        let downloaded = std::fs::read_dir("./downloads").unwrap();

        for entry in downloaded {
            let path = entry.unwrap().path();

            self.add_resource(path);
        }
    }

    pub fn playlist_clear(&mut self) {
        let pl_clone = self.playlist.clone();
        for res in &pl_clone {
            self.remove_resource(res);
        }
    }

    pub fn shuffle(&mut self) {
        use rand::{seq::SliceRandom, thread_rng};

        self.playlist.shuffle(&mut thread_rng());
    }
}
