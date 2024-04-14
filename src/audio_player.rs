use std::{
    fmt::Debug,
    time::{Duration, Instant},
};

use crate::audio_resource::AudioResource;

#[derive(Debug, Clone, PartialEq)]
pub enum PlayerAction {
    Play,
    Pause,
    Stop,
    Next,
    Previous,
    None,
}

impl Default for PlayerAction {
    fn default() -> Self {
        Self::None
    }
}

pub struct AudioPlayer {
    pub current: Option<AudioResource>,
    last_action: PlayerAction,
    start_time: Option<Instant>,
    pause_time: Option<std::time::Instant>,
    paused_duration: Duration,
    sink: rodio::Sink,
    _stream: rodio::OutputStream,
    _stream_handle: rodio::OutputStreamHandle,
}

impl AudioPlayer {
    pub fn play(&mut self, resource: AudioResource) {
        if let Some(current) = &self.current {
            if current == &resource {
                if let Some(pause_time) = self.pause_time {
                    self.paused_duration += pause_time.elapsed();
                }
                self.pause_time = None;
                self.sink.play();
                return;
            } else if self.start_time.is_none() {
                self.restart();
            }
        }
        self.stop();
        self.restart();
        self.current = Some(resource.clone());
        self.sink.append(resource.decoder());
        self.sink.play();
        self.last_action = PlayerAction::Play;
    }
    fn restart(&mut self) {
        self.start_time = Some(Instant::now());
        self.pause_time = Option::None;
        self.paused_duration = Duration::new(0, 0);
    }
    pub fn pause(&mut self) {
        self.sink.pause();
        self.last_action = PlayerAction::Pause;
        self.pause_time = Some(std::time::Instant::now());
    }
    pub fn stop(&mut self) {
        self.pause_time = None;
        self.start_time = None;
        self.last_action = PlayerAction::Stop;
        self.current = None;
        self.sink.pause();
        self.sink.clear();
    }
    pub fn set_volume(&mut self, volume: f32) {
        let vol = f32::clamp(volume, 0.0, 1.0);
        self.sink.set_volume(vol);
    }
    pub fn set_volume_100(&mut self, volume: u8) {
        let vol = f32::clamp(volume as f32 / 100.0, 0.0, 1.0);
        self.sink.set_volume(vol);
    }
    pub fn volume(&self) -> f32 {
        self.sink.volume()
    }
    pub fn just_finished(&self) -> bool {
        self.last_action == PlayerAction::Play && !self.is_playing()
    }
    pub fn is_playing(&self) -> bool {
        !self.sink.empty() && !self.sink.is_paused() && self.current.is_some()
    }
    pub fn elapsed(&self) -> Result<Duration, std::io::Error> {
        if let Some(start) = self.start_time {
            let mut elapsed_time = start.elapsed();
            if let Some(paused_time) = self.pause_time {
                elapsed_time -= self.paused_duration + paused_time.elapsed();
            } else {
                elapsed_time -= self.paused_duration;
            }

            Ok(elapsed_time)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Timer not started",
            ))
        }
    }
}

impl Default for AudioPlayer {
    fn default() -> Self {
        let (_stream, _stream_handle) = rodio::OutputStream::try_default().unwrap();
        let sink = rodio::Sink::try_new(&_stream_handle).unwrap();
        Self {
            current: None,
            last_action: PlayerAction::None,
            start_time: None,
            pause_time: None,
            paused_duration: std::time::Duration::from_secs(0),
            sink,
            _stream,
            _stream_handle,
        }
    }
}

impl Debug for AudioPlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioPlayer")
            .field("current", &self.current)
            .field("last_action", &self.last_action)
            .field("start_time", &self.start_time)
            .field("pause_time", &self.pause_time)
            .field("paused_duration", &self.paused_duration)
            .finish()
    }
}
