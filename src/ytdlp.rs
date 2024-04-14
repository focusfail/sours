use std::os::windows::process::CommandExt;
use std::thread;

#[derive(Debug, Default)]
pub struct Downloader {
    handle: Option<thread::JoinHandle<()>>,
}

impl Downloader {
    pub fn download(&mut self, url: String, n: u32) {
        let mut command = std::process::Command::new(r".\bin\yt-dlp.exe");

        #[cfg(target_os = "windows")]
        command.creation_flags(0x08000000);

        self.handle = Some(thread::spawn(move || {
            let _ = command
                .arg("--extract-audio")
                .arg("--audio-format")
                .arg("mp3")
                .arg("--playlist-end")
                .arg(format!("{}", n))
                .arg("-o")
                .arg(r".\downloads\%(title)s.%(ext)s")
                .arg(url)
                .status()
                .unwrap();
        }));
    }
    pub fn is_finished(&mut self) -> bool {
        if let Some(handle) = &self.handle {
            if handle.is_finished() {
                self.handle = None;
                return true;
            } else {
                return false;
            }
        }
        self.handle.is_none()
    }
}
