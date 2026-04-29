use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use udev::{Device, Udev};

use crate::models::{DeviceRowKind, SerialDeviceEntry, UNKNOWN_INTERFACE_ID};

pub fn scan_serial_devices() -> Result<Vec<SerialDeviceEntry>, String> {
    let udev_context = Udev::new().map_err(|error| format!("无法创建 udev 上下文: {error}"))?;

    let mut devices = Vec::new();
    devices.extend(scan_virtual_link_devices(&udev_context)?);
    devices.extend(scan_physical_devices(&udev_context)?);
    devices.sort_by(device_sort_key);

    Ok(devices)
}

fn scan_virtual_link_devices(udev_context: &Udev) -> Result<Vec<SerialDeviceEntry>, String> {
    let mut virtual_link_devices = Vec::new();

    for directory_entry in read_dev_directory()? {
        let directory_entry = directory_entry.map_err(io_error)?;

        if !directory_entry.file_type().map_err(io_error)?.is_symlink() {
            continue;
        }

        let virtual_path = directory_entry.path();
        let virtual_name = file_name_string(&virtual_path);
        if !virtual_name.starts_with("tty") {
            continue;
        }

        let physical_path = resolve_device_link(&virtual_path)?;
        let physical_name = file_name_string(&physical_path);
        if !is_supported_serial_device_name(&physical_name) {
            continue;
        }

        virtual_link_devices.push(SerialDeviceEntry {
            row_kind: DeviceRowKind::VirtualLink,
            virtual_name,
            physical_name: physical_name.clone(),
            interface_id: read_usb_interface_id(udev_context, &physical_name),
        });
    }

    Ok(virtual_link_devices)
}

fn scan_physical_devices(udev_context: &Udev) -> Result<Vec<SerialDeviceEntry>, String> {
    let mut physical_devices = Vec::new();

    for directory_entry in read_dev_directory()? {
        let directory_entry = directory_entry.map_err(io_error)?;
        let file_type = directory_entry.file_type().map_err(io_error)?;
        if file_type.is_symlink() {
            continue;
        }

        let device_name = directory_entry.file_name().to_string_lossy().into_owned();
        if !is_supported_serial_device_name(&device_name) {
            continue;
        }

        physical_devices.push(SerialDeviceEntry {
            row_kind: DeviceRowKind::PhysicalDevice,
            virtual_name: "-".into(),
            physical_name: device_name.clone(),
            interface_id: read_usb_interface_id(udev_context, &device_name),
        });
    }

    Ok(physical_devices)
}

fn read_dev_directory() -> Result<fs::ReadDir, String> {
    fs::read_dir("/dev").map_err(|error| format!("无法读取 /dev 目录: {error}"))
}

fn resolve_device_link(virtual_path: &Path) -> Result<PathBuf, String> {
    let resolved_path = fs::read_link(virtual_path).map_err(io_error)?;

    if resolved_path.is_absolute() {
        Ok(resolved_path)
    } else {
        Ok(virtual_path
            .parent()
            .unwrap_or_else(|| Path::new("/dev"))
            .join(resolved_path))
    }
}

fn read_usb_interface_id(udev_context: &Udev, tty_device_name: &str) -> String {
    let tty_device = match Device::from_subsystem_sysname_with_context(
        udev_context.clone(),
        "tty".into(),
        tty_device_name.into(),
    ) {
        Ok(device) => device,
        Err(_) => return UNKNOWN_INTERFACE_ID.into(),
    };

    match tty_device.parent_with_subsystem_devtype("usb", "usb_interface") {
        Ok(Some(interface_device)) => os_str_to_string(interface_device.sysname()),
        _ => UNKNOWN_INTERFACE_ID.into(),
    }
}

fn is_supported_serial_device_name(device_name: &str) -> bool {
    device_name.starts_with("ttyUSB") || device_name.starts_with("ttyACM")
}

fn file_name_string(path: &Path) -> String {
    path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

fn os_str_to_string(value: &OsStr) -> String {
    value.to_string_lossy().into_owned()
}

fn device_sort_key(left: &SerialDeviceEntry, right: &SerialDeviceEntry) -> std::cmp::Ordering {
    let left_key = (
        left.row_kind == DeviceRowKind::PhysicalDevice,
        &left.virtual_name,
        &left.physical_name,
        &left.interface_id,
    );
    let right_key = (
        right.row_kind == DeviceRowKind::PhysicalDevice,
        &right.virtual_name,
        &right.physical_name,
        &right.interface_id,
    );

    left_key.cmp(&right_key)
}

fn io_error(error: std::io::Error) -> String {
    error.to_string()
}
