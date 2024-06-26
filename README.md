# About <a href="https://www.rust-lang.org/"><img align="right" src="https://img.shields.io/badge/Rust-1%2E73-f74c00?logo=Rust" alt="Rust 1.73" /></a>

**WinMix** is a rust library that allows you to individually change the volume of each program in the Windows Volume Mixer.

For example, you can set the volume of `chrome.exe` to `0` while leaving other apps alone.

⚠ This libary uses **unsafe** functions from the [windows](https://crates.io/crates/windows) crate. ⚠

# Usage

```rs
use winmix::WinMix;

unsafe {
    let winmix = WinMix::default();

    // Get a list of all programs that have an entry in the volume mixer
    let sessions = winmix.enumerate()?;

    for session in sessions {
        // PID and path of the process
        println!("pid: {}   path: {}", session.pid, session.path);

        // Mute
        session.vol.set_mute(true)?;
        session.vol.set_mute(false)?;

        // 50% volume
        session.vol.set_master_volume(0.5)?;
        // Back to 100% volume
        session.vol.set_master_volume(1.0)?;

        // Get the current volume, or see if it's muted
        let vol = session.vol.get_master_volume()?;
        let is_muted = session.vol.get_mute()?;

        println!("Vol: {}   Muted: {}", vol, is_muted);
        println!();
    }
}
```

# License <a href="https://github.com/UnexomWid/winmix/blob/master/LICENSE"><img align="right" src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License: MIT" /></a>

**WinMix** was created by [UnexomWid](https://uw.exom.dev). It is licensed under [MIT](https://github.com/UnexomWid/winmix/blob/master/LICENSE-MIT) OR [Apache 2](https://github.com/UnexomWid/winmix/blob/master/LICENSE-APACHE).

# References

- [The Microsoft documentation](https://learn.microsoft.com/en-us/windows/win32/coreaudio/programming-guide)