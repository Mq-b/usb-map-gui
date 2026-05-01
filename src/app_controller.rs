use std::cell::RefCell;
use std::rc::Rc;

use slint::{ComponentHandle, PhysicalSize};

use crate::MainWindow;
use crate::device_scan::scan_serial_devices;
use crate::models::{DEFAULT_RULE_FILE_PATH, DeviceListFilter, SerialDeviceEntry};
use crate::rule_file::update_rule_file;
use crate::ui_bindings::{
    apply_device_rows, fill_rule_form_from_row, read_rule_request, show_status,
};

#[derive(Debug, Default)]
struct ApplicationState {
    scanned_devices: Vec<SerialDeviceEntry>,
    current_filter: Option<DeviceListFilter>,
}

const INITIAL_WINDOW_WIDTH: u32 = 1000;
const INITIAL_WINDOW_HEIGHT: u32 = 700;

pub fn run_application() -> Result<(), slint::PlatformError> {
    let window = MainWindow::new()?;
    let state = Rc::new(RefCell::new(ApplicationState {
        scanned_devices: Vec::new(),
        current_filter: Some(DeviceListFilter::AllDevices),
    }));

    window.set_rule_file_input(DEFAULT_RULE_FILE_PATH.into());
    show_status(
        &window,
        "点击“刷新设备列表”读取当前串口映射；点击列表行可把接口 ID 带入下方规则表单。",
        false,
    );

    bind_callbacks(&window, state.clone());
    refresh_device_rows(&window, &state);
    window.show()?;
    window
        .window()
        .set_size(PhysicalSize::new(INITIAL_WINDOW_WIDTH, INITIAL_WINDOW_HEIGHT));
    slint::run_event_loop()?;
    window.hide()
}

fn bind_callbacks(window: &MainWindow, state: Rc<RefCell<ApplicationState>>) {
    let weak_window = window.as_weak();
    let refresh_state = state.clone();
    window.on_request_refresh(move || {
        if let Some(window) = weak_window.upgrade() {
            refresh_device_rows(&window, &refresh_state);
        }
    });

    let weak_window = window.as_weak();
    let all_filter_state = state.clone();
    window.on_select_all_filter(move || {
        if let Some(window) = weak_window.upgrade() {
            all_filter_state.borrow_mut().current_filter = Some(DeviceListFilter::AllDevices);
            refresh_visible_rows(&window, &all_filter_state.borrow());
        }
    });

    let weak_window = window.as_weak();
    let virtual_filter_state = state.clone();
    window.on_select_virtual_filter(move || {
        if let Some(window) = weak_window.upgrade() {
            virtual_filter_state.borrow_mut().current_filter =
                Some(DeviceListFilter::VirtualLinksOnly);
            refresh_visible_rows(&window, &virtual_filter_state.borrow());
        }
    });

    let weak_window = window.as_weak();
    let physical_filter_state = state.clone();
    window.on_select_physical_filter(move || {
        if let Some(window) = weak_window.upgrade() {
            physical_filter_state.borrow_mut().current_filter =
                Some(DeviceListFilter::PhysicalDevicesOnly);
            refresh_visible_rows(&window, &physical_filter_state.borrow());
        }
    });

    let weak_window = window.as_weak();
    let select_state = state.clone();
    window.on_use_row_for_rule(move |row_index| {
        if let Some(window) = weak_window.upgrade() {
            handle_row_selection(&window, &select_state, row_index as usize);
        }
    });

    let weak_window = window.as_weak();
    window.on_save_rule_requested(move || {
        if let Some(window) = weak_window.upgrade() {
            handle_rule_save(&window);
        }
    });
}

fn refresh_device_rows(window: &MainWindow, state: &Rc<RefCell<ApplicationState>>) {
    match scan_serial_devices() {
        Ok(scanned_devices) => {
            {
                let mut state = state.borrow_mut();
                state.scanned_devices = scanned_devices;
            }

            refresh_visible_rows(window, &state.borrow());
            show_status(
                window,
                "设备列表已刷新。点击任意行可以快速带入接口 ID。",
                false,
            );
        }
        Err(error_message) => {
            state.borrow_mut().scanned_devices.clear();
            refresh_visible_rows(window, &state.borrow());
            show_status(window, error_message, true);
        }
    }
}

fn refresh_visible_rows(window: &MainWindow, state: &ApplicationState) {
    let active_filter = state.current_filter.unwrap_or(DeviceListFilter::AllDevices);
    apply_device_rows(window, &state.scanned_devices, active_filter);
}

fn handle_row_selection(
    window: &MainWindow,
    state: &Rc<RefCell<ApplicationState>>,
    row_index: usize,
) {
    let state = state.borrow();
    let active_filter = state.current_filter.unwrap_or(DeviceListFilter::AllDevices);
    let visible_devices = state
        .scanned_devices
        .iter()
        .filter(|device| device.matches_filter(active_filter))
        .collect::<Vec<_>>();

    let Some(device) = visible_devices.get(row_index) else {
        show_status(window, "未找到对应的设备行。", true);
        return;
    };

    if !device.can_fill_rule() {
        show_status(window, "该设备没有可用的接口 ID，无法直接生成规则。", true);
        return;
    }

    fill_rule_form_from_row(window, device);
    show_status(
        window,
        "已将所选设备的接口 ID 带入规则表单，可以直接保存规则。",
        false,
    );
}

fn handle_rule_save(window: &MainWindow) {
    let rule_request = read_rule_request(window);

    match update_rule_file(&rule_request) {
        Ok(result) => {
            show_status(
                window,
                format!(
                    "{}规则。随后执行 `sudo udevadm control --reload-rules && sudo udevadm trigger` 使规则生效。",
                    result.action.label()
                ),
                false,
            );
        }
        Err(error_message) => show_status(window, error_message, true),
    }
}
