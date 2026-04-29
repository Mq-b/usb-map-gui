fn main() {
    let compiler_config = slint_build::CompilerConfiguration::new().with_style("fluent".into());

    slint_build::compile_with_config("ui/app_window.slint", compiler_config)
        .expect("failed to compile Slint UI");
}
