use std::{ffi::c_void, io::Error};

use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, PlatformConfig};
use windows::{core::HSTRING, Win32::UI::WindowsAndMessaging::FindWindowW};

pub struct PlatformControls;

impl PlatformControls {
    pub fn new() -> Result<Self, std::io::Error> {
        let hwnd = unsafe { FindWindowW(None, &HSTRING::from("sours")) };

        if hwnd.0 == 0 {
            return Err(Error::last_os_error());
        }

        let config = PlatformConfig {
            dbus_name: "soursplayer",
            display_name: "sours",
            hwnd: Some(hwnd.0 as *mut c_void),
        };

        let mut controls = match MediaControls::new(config) {
            Ok(controls) => controls,
            Err(error) => {
                println!("{:?}", error);
                panic!("{error:?}")
            }
        };

        controls
            .attach(|event: MediaControlEvent| println!("Event received: {:?}", event))
            .unwrap();
        controls
            .set_metadata(MediaMetadata {
                title: Some("Sours test"),
                artist: Some("test"),
                album: Some("test"),
                ..Default::default()
            })
            .unwrap();

        Ok(Self {})
    }
}
