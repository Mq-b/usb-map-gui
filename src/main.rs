mod app_controller;
mod device_scan;
mod models;
mod rule_file;
mod ui_bindings;

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    app_controller::run_application()
}
