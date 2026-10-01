use std::path::{Path, PathBuf};
use std::time::{Duration, UNIX_EPOCH};

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use walkdir::WalkDir;

const AUDIO_EXTENSIONS: &[&str] = &["mp3", "wav", "m4a", "aac", "flac"];
const POLL_INTERVAL: Duration = Duration::from_secs(3);
const MAX_SCAN_DEPTH: usize = 4;

pub const USB_DEVICE_ATTACHED_EVENT: &str = "usb-device-attached";
pub const USB_DEVICE_DETACHED_EVENT: &str = "usb-device-detached";

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RecorderFile {
    pub path: String,
    pub name: String,
    pub size_bytes: u64,
    pub modified_at: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ConnectedDevice {
    pub mount_path: String,
    pub label: String,
    pub files: Vec<RecorderFile>,
}

fn user_name() -> Option<String> {
    std::env::var("USER")
        .ok()
        .filter(|value| !value.trim().is_empty())
}

fn search_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(user) = user_name() {
        roots.push(PathBuf::from(format!("/media/{user}")));
        roots.push(PathBuf::from(format!("/run/media/{user}")));
    }
    roots
}

fn is_audio_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| AUDIO_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

fn modified_secs(path: &Path) -> Option<u64> {
    std::fs::metadata(path)
        .and_then(|meta| meta.modified())
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs())
}

fn to_recorder_file(path: &Path) -> RecorderFile {
    RecorderFile {
        path: path.to_string_lossy().into_owned(),
        name: path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default(),
        size_bytes: std::fs::metadata(path).map(|meta| meta.len()).unwrap_or(0),
        modified_at: modified_secs(path),
    }
}

fn scan_audio_files(root: &Path) -> Vec<RecorderFile> {
    let mut files: Vec<RecorderFile> = WalkDir::new(root)
        .max_depth(MAX_SCAN_DEPTH)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file() && is_audio_file(entry.path()))
        .map(|entry| to_recorder_file(entry.path()))
        .collect();

    files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    files
}

fn device_from_mount(path: &Path) -> ConnectedDevice {
    ConnectedDevice {
        mount_path: path.to_string_lossy().into_owned(),
        label: path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default(),
        files: scan_audio_files(path),
    }
}

pub fn scan_devices() -> Vec<ConnectedDevice> {
    let mut devices = Vec::new();

    for root in search_roots() {
        let Ok(entries) = std::fs::read_dir(&root) else {
            continue;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            devices.push(device_from_mount(&path));
        }
    }

    devices
}

pub fn current_device() -> Option<ConnectedDevice> {
    scan_devices().into_iter().next()
}

pub fn spawn(app: AppHandle) {
    std::thread::spawn(move || {
        let mut previous = current_device();
        if let Some(device) = &previous {
            let _ = app.emit(USB_DEVICE_ATTACHED_EVENT, device);
        }

        loop {
            std::thread::sleep(POLL_INTERVAL);

            let current = current_device();
            if current == previous {
                continue;
            }

            match &current {
                Some(device) => {
                    let _ = app.emit(USB_DEVICE_ATTACHED_EVENT, device);
                }
                None => {
                    let _ = app.emit(USB_DEVICE_DETACHED_EVENT, ());
                }
            }

            previous = current;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_supported_audio_extensions() {
        for name in ["a.mp3", "b.WAV", "c.m4a", "d.AAC", "e.flac"] {
            assert!(is_audio_file(Path::new(name)), "{name} should be audio");
        }
    }

    #[test]
    fn rejects_non_audio_files() {
        for name in ["a.txt", "b.pdf", "c", "d.ogg", "e.wav.bak"] {
            assert!(!is_audio_file(Path::new(name)), "{name} should not be audio");
        }
    }

    #[test]
    fn scans_audio_files_recursively() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(directory.path().join("one.mp3"), b"x").unwrap();
        std::fs::write(directory.path().join("notes.txt"), b"x").unwrap();
        let nested = directory.path().join("sub");
        std::fs::create_dir(&nested).unwrap();
        std::fs::write(nested.join("two.flac"), b"xx").unwrap();

        let files = scan_audio_files(directory.path());

        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|file| file.name == "one.mp3"));
        assert!(files.iter().any(|file| file.name == "two.flac" && file.size_bytes == 2));
    }

    #[test]
    fn builds_device_from_mount() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(directory.path().join("rec.wav"), b"data").unwrap();

        let device = device_from_mount(directory.path());

        assert_eq!(device.files.len(), 1);
        assert!(device.mount_path.ends_with(directory.path().file_name().unwrap().to_str().unwrap()));
    }
}
