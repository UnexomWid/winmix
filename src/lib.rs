//! WinMix: Change Windows Mixer Volume via Rust
//!
//! This is a rust library that allows you to individually change the volume of each program in the Windows Mixer.
//!
//! For example, you can set the volume of `chrome.exe` to `0` while leaving other apps alone.
//!
//! ⚠ This libary uses **unsafe** functions from the [windows](https://crates.io/crates/windows) crate. ⚠
//!
//! # Usage
//!
//! ```no_run
//! use winmix::WinMix;
//!
//! unsafe {
//!     let winmix = WinMix::default();
//!
//!     // Enumerate all audio sessions, one for each program
//!     let sessions = winmix.enumerate()?;
//!
//!     for session in sessions {
//!         // You get the PID and path of the process that controls this audio session
//!         println!("pid: {}   path: {}", session.pid, session.path);
//!
//!         // You can mute or change the volume
//!         session.vol.set_mute(true)?;
//!         session.vol.set_mute(false)?;
//!
//!         // 50% volume
//!         session.vol.set_master_volume(0.5)?;
//!         // Back to 100% volume
//!         session.vol.set_master_volume(1.0)?;
//!
//!         // You can also get the current volume, or see if it's muted
//!         let vol = session.vol.get_master_volume()?;
//!         let is_muted = session.vol.get_mute()?;
//!
//!         println!("Vol: {}   Muted: {}", vol, is_muted);
//!         println!();
//!     }
//! }
//! ```
//!
use std::ptr;
use windows::{
    core::Interface,
    Win32::{
        Foundation::{CloseHandle, MAX_PATH},
        Media::Audio::{
            eRender, IAudioSessionControl, IAudioSessionControl2, IAudioSessionEnumerator,
            IAudioSessionManager2, IMMDeviceCollection, IMMDeviceEnumerator, ISimpleAudioVolume,
            MMDeviceEnumerator, DEVICE_STATE_ACTIVE,
        },
        System::{
            Com::{CoCreateInstance, CoInitialize, CoUninitialize, CLSCTX_ALL},
            ProcessStatus::GetModuleFileNameExW,
            Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ},
        },
    },
};
use windows_result::Error;

pub struct WinMix {
    // Whether or not we initialized COM; if so, we have to clean up later
    com_initialized: bool,
}

impl WinMix {
    /// Enumerate all audio sessions from all audio endpoints via WASAPI.
    ///
    /// # Safety
    /// This function calls other unsafe functions from the [windows](https://crates.io/crates/windows) crate.
    pub unsafe fn enumerate(&self) -> Result<Vec<Session>, Error> {
        let mut result = Vec::<Session>::new();

        let res: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;

        let collection: IMMDeviceCollection =
            res.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)?;

        let device_count = collection.GetCount()?;

        for device_id in 0..device_count {
            let dev = collection.Item(device_id)?;

            let manager: IAudioSessionManager2 = dev.Activate(CLSCTX_ALL, None)?;
            let enumerator: IAudioSessionEnumerator = manager.GetSessionEnumerator()?;

            let session_count = enumerator.GetCount()?;

            for session_id in 0..session_count {
                let ctrl: IAudioSessionControl = enumerator.GetSession(session_id)?;
                let ctrl2: IAudioSessionControl2 = ctrl.cast()?;

                let pid = ctrl2.GetProcessId()?;

                if pid == 0 {
                    // System sounds session, so we ignore it.
                    //
                    // We use this PID == 0 hack because ctrl2.IsSystemSoundsSession() from the windows crate doesn't work yet.
                    // https://github.com/microsoft/win32metadata/issues/1664
                    continue;
                }

                let proc = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid)?;

                let mut path: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];

                let res = GetModuleFileNameExW(proc, None, &mut path);
                CloseHandle(proc)?;

                if res == 0 {
                    // Failed to get filename from PID (insufficient permissions?)
                    //continue
                }

                let vol: ISimpleAudioVolume = ctrl2.cast()?;

                // Trim trailing \0
                let mut path = String::from_utf16_lossy(&path);
                path.truncate(path.trim_matches(char::from(0)).len());

                result.push(Session {
                    pid,
                    path,
                    vol: SimpleVolume { handle: vol },
                });
            }
        }

        Ok(result)
    }
}

impl Default for WinMix {
    /// Create a default instance of WinMix.
    fn default() -> WinMix {
        unsafe {
            let hres = CoInitialize(None);

            // If we initialized COM, we are responsible for cleaning it up later.
            // If it was already initialized, we don't have to do anything.
            WinMix {
                com_initialized: hres.is_ok(),
            }
        }
    }
}

impl Drop for WinMix {
    fn drop(&mut self) {
        unsafe {
            if self.com_initialized {
                // We initialized COM, so we uninitialize it
                CoUninitialize();
            }
        }
    }
}

pub struct Session {
    /// The PID of the process that controls this audio session.
    pub pid: u32,
    /// The exe path for the process that controls this audio session.
    pub path: String,
    /// A wrapper that lets you control the volume for this audio session.
    pub vol: SimpleVolume,
}

pub struct SimpleVolume {
    handle: ISimpleAudioVolume,
}

impl SimpleVolume {
    /// Get the master volume for this session.
    ///
    /// # Safety
    /// This function calls [ISimpleAudioVolume.GetMasterVolume](https://learn.microsoft.com/en-us/windows/win32/api/audioclient/nf-audioclient-isimpleaudiovolume-getmastervolume) which is unsafe.
    pub unsafe fn get_master_volume(&self) -> Result<f32, Error> {
        self.handle.GetMasterVolume()
    }

    /// Set the master volume for this session.
    ///
    /// * `level` - the volume level, between `0.0` and `1.0`\
    ///
    /// # Safety
    /// This function calls [ISimpleAudioVolume.SetMasterVolume](https://learn.microsoft.com/en-us/windows/win32/api/audioclient/nf-audioclient-isimpleaudiovolume-setmastervolume) which is unsafe.
    pub unsafe fn set_master_volume(&self, level: f32) -> Result<(), Error> {
        self.handle.SetMasterVolume(level, ptr::null())
    }

    /// Check if this session is muted.
    ///
    /// # Safety
    /// This function calls [ISimpleAudioVolume.GetMute](https://learn.microsoft.com/en-us/windows/win32/api/audioclient/nf-audioclient-isimpleaudiovolume-getmute) which is unsafe.
    pub unsafe fn get_mute(&self) -> Result<bool, Error> {
        match self.handle.GetMute() {
            Ok(val) => Ok(val.as_bool()),
            Err(e) => Err(e),
        }
    }

    /// Mute or unmute this session.
    ///
    /// * `val` - `true` to mute, `false` to unmute
    ///
    /// # Safety
    /// This function calls [ISimpleAudioVolume.SetMute](https://learn.microsoft.com/en-us/windows/win32/api/audioclient/nf-audioclient-isimpleaudiovolume-setmute) which is unsafe.
    pub unsafe fn set_mute(&self, val: bool) -> Result<(), Error> {
        self.handle.SetMute(val, ptr::null())
    }
}
