# About <a href="https://en.wikipedia.org/wiki/C%2B%2B17"><img align="right" src="https://img.shields.io/badge/Rust-1%2E17-f74c00?logo=Rust" alt="Rust 1.73" /></a>

**WinMix** is a rust library that allows you to individually change the volume of each program in the Windows Mixer.

For example, you can set the volume of `chrome.exe` to `0` while leaving other apps alone.

⚠ This libary uses **unsafe** functions from the [windows](https://crates.io/crates/windows) crate. ⚠

# Usage

```rs
use winmix::WinMix;

unsafe {
    let winmix = WinMix::default();

    // Enumerate all audio sessions, one for each program
    let sessions = winmix.enumerate().unwrap();

    for session in sessions {
        // You get the PID and path of the process that controls this audio session
        println!("pid: {}   path: {}", session.pid, session.path);

        // You can mute or change the volume
        session.vol.set_mute(true).unwrap();
        session.vol.set_mute(false).unwrap();

        // 50% volume
        session.vol.set_master_volume(0.5).unwrap();
        // Back to 100% volume
        session.vol.set_master_volume(1.0).unwrap();

        // You can also get the current volume, or see if it's muted
        let vol = session.vol.get_master_volume().unwrap();
        let is_muted = session.vol.get_mute().unwrap();

        println!("Vol: {}   Muted: {}", vol, is_muted);
        println!();
    }
}
```

# License <a href="https://github.com/UnexomWid/winmix/blob/master/LICENSE"><img align="right" src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License: MIT" /></a>

**WinMix** was created by [UnexomWid](https://uw.exom.dev). It is licensed under the [MIT](https://github.com/UnexomWid/winmix/blob/master/LICENSE) license.

# References

- [The Microsoft documentation](https://learn.microsoft.com/en-us/windows/win32/coreaudio/programming-guide)