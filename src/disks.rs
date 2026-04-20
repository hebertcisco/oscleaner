use std::path::{Path, PathBuf};

use indicatif::HumanBytes;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WindowsDisk {
    pub root: PathBuf,
    pub label: String,
    pub is_system_drive: bool,
}

pub fn available_windows_disks(system_drive: Option<&Path>) -> Vec<WindowsDisk> {
    available_windows_disks_impl(system_drive)
}

#[cfg(target_os = "windows")]
fn available_windows_disks_impl(system_drive: Option<&Path>) -> Vec<WindowsDisk> {
    use windows_sys::Win32::Storage::FileSystem::{GetDriveTypeW, GetLogicalDrives};

    const DRIVE_REMOVABLE: u32 = 2;
    const DRIVE_FIXED: u32 = 3;
    const DRIVE_REMOTE: u32 = 4;
    const DRIVE_RAMDISK: u32 = 6;

    let system_root = system_drive.map(normalize_drive_root);
    let mut disks = Vec::new();
    let bitmask = unsafe { GetLogicalDrives() };

    if bitmask == 0 {
        return disks;
    }

    for offset in 0..26 {
        if bitmask & (1 << offset) == 0 {
            continue;
        }

        let letter = (b'A' + offset as u8) as char;
        let root_str = format!("{letter}:\\");
        let root = PathBuf::from(&root_str);
        let root_wide = to_wide_null(&root_str);
        let drive_type = unsafe { GetDriveTypeW(root_wide.as_ptr()) };

        let kind = match drive_type {
            DRIVE_FIXED => "Fixed",
            DRIVE_REMOVABLE => "Removable",
            DRIVE_REMOTE => "Network",
            DRIVE_RAMDISK => "RAM disk",
            _ => continue,
        };

        let volume_label = unsafe { volume_label(root_wide.as_ptr()) };
        let space = unsafe { disk_space(root_wide.as_ptr()) };
        let is_system_drive = system_root
            .as_ref()
            .is_some_and(|system_root| *system_root == root);

        disks.push(WindowsDisk {
            root,
            label: format_disk_label(&root_str, kind, volume_label, space, is_system_drive),
            is_system_drive,
        });
    }

    disks.sort_by(|left, right| {
        right
            .is_system_drive
            .cmp(&left.is_system_drive)
            .then_with(|| left.root.cmp(&right.root))
    });

    disks
}

#[cfg(not(target_os = "windows"))]
fn available_windows_disks_impl(_system_drive: Option<&Path>) -> Vec<WindowsDisk> {
    Vec::new()
}

fn format_disk_label(
    root: &str,
    kind: &str,
    volume_label: Option<String>,
    space: Option<(u64, u64)>,
    is_system_drive: bool,
) -> String {
    let mut parts = vec![root.to_string()];
    if let Some(volume_label) = volume_label {
        parts.push(format!("[{volume_label}]"));
    }
    parts.push(kind.to_string());
    if is_system_drive {
        parts.push("System".to_string());
    }
    if let Some((free_bytes, total_bytes)) = space {
        parts.push(format!(
            "{} free of {}",
            HumanBytes(free_bytes),
            HumanBytes(total_bytes)
        ));
    }
    parts.join(" | ")
}

fn normalize_drive_root(path: &Path) -> PathBuf {
    let raw = path.to_string_lossy().replace('/', "\\");
    let trimmed = raw.trim_end_matches('\\');

    match trimmed.as_bytes() {
        [drive, b':', ..] => PathBuf::from(format!("{}:\\", (*drive as char).to_ascii_uppercase())),
        _ => path.to_path_buf(),
    }
}

#[cfg(target_os = "windows")]
fn to_wide_null(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}

#[cfg(target_os = "windows")]
fn from_wide_nul_terminated(buffer: &[u16]) -> String {
    let len = buffer
        .iter()
        .position(|ch| *ch == 0)
        .unwrap_or(buffer.len());
    String::from_utf16_lossy(&buffer[..len])
}

#[cfg(target_os = "windows")]
unsafe fn volume_label(root: *const u16) -> Option<String> {
    use std::ptr::null_mut;

    use windows_sys::Win32::Storage::FileSystem::GetVolumeInformationW;

    let mut buffer = [0u16; 261];
    let ok = unsafe {
        GetVolumeInformationW(
            root,
            buffer.as_mut_ptr(),
            buffer.len() as u32,
            null_mut(),
            null_mut(),
            null_mut(),
            null_mut(),
            0,
        )
    };

    if ok == 0 {
        return None;
    }

    let label = from_wide_nul_terminated(&buffer);
    if label.trim().is_empty() {
        None
    } else {
        Some(label)
    }
}

#[cfg(target_os = "windows")]
unsafe fn disk_space(root: *const u16) -> Option<(u64, u64)> {
    use std::ptr::null_mut;

    use windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;

    let mut free_bytes = 0u64;
    let mut total_bytes = 0u64;
    let ok = unsafe { GetDiskFreeSpaceExW(root, &mut free_bytes, &mut total_bytes, null_mut()) };
    if ok == 0 {
        None
    } else {
        Some((free_bytes, total_bytes))
    }
}
