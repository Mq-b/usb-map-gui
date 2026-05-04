use slint::{ModelRc, SharedString, VecModel};

use crate::models::{DeviceListFilter, RuleUpdateRequest, SerialDeviceEntry};
use crate::{DeviceRowData, MainWindow};

pub fn apply_device_rows(
    window: &MainWindow,
    devices: &[SerialDeviceEntry],
    filter: DeviceListFilter,
) {
    let row_data = devices
        .iter()
        .filter(|device| device.matches_filter(filter))
        .map(to_row_data)
        .collect::<Vec<_>>();

    window.set_device_count(row_data.len() as i32);
    window.set_device_rows(ModelRc::new(VecModel::from(row_data)));
    window.set_current_filter_label(filter.label().into());
}

pub fn show_status(window: &MainWindow, message: impl Into<SharedString>, is_error: bool) {
    window.set_status_message(message.into());
    window.set_status_is_error(is_error);
}

pub fn fill_rule_form_from_row(window: &MainWindow, device: &SerialDeviceEntry) {
    if !device.suggested_virtual_name().is_empty() {
        window.set_virtual_name_input(device.suggested_virtual_name().into());
    }

    window.set_physical_id_input(device.interface_id.clone().into());
}

pub fn read_rule_request(window: &MainWindow) -> RuleUpdateRequest {
    RuleUpdateRequest::new(
        window.get_virtual_name_input().trim().to_string(),
        window.get_physical_id_input().trim().to_string(),
        window.get_rule_file_input().trim().to_string(),
    )
}

fn to_row_data(device: &SerialDeviceEntry) -> DeviceRowData {
    DeviceRowData {
        row_kind_label: device.row_kind.label().into(),
        row_kind: match device.row_kind {
            crate::models::DeviceRowKind::VirtualLink => "virtual",
            crate::models::DeviceRowKind::PhysicalDevice => "physical",
        }
        .into(),
        virtual_name: device.virtual_name.clone().into(),
        physical_name: device.physical_name.clone().into(),
        interface_id: device.interface_id.clone().into(),
        can_use_for_rule: device.can_fill_rule(),
        status: "可用".into(),
    }
}
