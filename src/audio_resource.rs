use lofty::{AudioFile, Probe};
use rodio::Decoder;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    fmt::Debug,
    fs::File,
    path::{Path, PathBuf},
    time::Duration,
};

#[derive(Clone)]
pub struct AudioResource {
    pub path: PathBuf,
    pub duration: Duration,
    playable: bool,
}

impl AudioResource {
    pub fn new(path: PathBuf, duration: Duration) -> Self {
        let playable = path.exists();
        Self {
            path,
            duration,
            playable,
        }
    }

    pub fn decoder(&self) -> Decoder<File> {
        Decoder::new(File::open(self.path.clone()).unwrap()).unwrap()
    }

    pub fn from_path(path: String) -> Self {
        let path = Path::new(&path).to_path_buf();
        let (playable, duration);
        if path.exists() {
            duration = Self::get_duration(path.to_str().unwrap());
            playable = true;
        } else {
            duration = Duration::from_secs(0);
            playable = false;
        }

        Self {
            path,
            duration,
            playable,
        }
    }

    pub fn formatted_duration(&self) -> String {
        let secs = self.duration.as_secs();
        let mins = secs / 60;
        let secs = secs % 60;

        format!("{:02}:{:02}", mins, secs)
    }

    fn get_duration(path: &str) -> Duration {
        let tagged = Probe::open(path).unwrap().read().unwrap();
        tagged.properties().duration()
    }

    pub fn playable(&self) -> bool {
        if !self.playable {
            self.playable
        } else {
            self.path.exists()
        }
    }
}

impl PartialEq for AudioResource {
    fn eq(&self, other: &Self) -> bool {
        self.path.file_name() == other.path.file_name() && self.duration == other.duration
    }
}

impl<'de> Deserialize<'de> for AudioResource {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = AudioResource;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a path to an audio file")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(AudioResource::from_path(value.to_string()))
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

impl Serialize for AudioResource {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.path.to_str().unwrap())
    }
}

impl Debug for AudioResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioResource")
            .field("path", &self.path)
            .field("duration", &self.duration)
            .finish()
    }
}
