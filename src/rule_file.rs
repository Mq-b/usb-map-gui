use std::fs;
use std::path::Path;
use std::process::Command;

use crate::models::RuleUpdateRequest;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuleWriteAction {
    Added,
    Updated,
}

impl RuleWriteAction {
    pub fn label(self) -> &'static str {
        match self {
            Self::Added => "已新增",
            Self::Updated => "已更新",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleWriteResult {
    pub action: RuleWriteAction,
    pub rule_line: String,
}

pub fn update_rule_file(request: &RuleUpdateRequest) -> Result<RuleWriteResult, String> {
    request.validate()?;

    let rule_line = build_rule_line(request);
    let rule_file_path = &request.rule_file_path;
    let mut existing_lines = read_rule_lines(rule_file_path)?;
    let mut matching_rule_found = false;

    for line in &mut existing_lines {
        if line.contains(&format!("KERNELS==\"{}\"", request.physical_id)) {
            *line = rule_line.clone();
            matching_rule_found = true;
        }
    }

    if !matching_rule_found {
        existing_lines.push(rule_line.clone());
    }

    ensure_parent_directory(rule_file_path)?;
    let mut file_content = existing_lines.join("\n");
    if !file_content.ends_with('\n') {
        file_content.push('\n');
    }

    fs::write(rule_file_path, file_content)
        .map_err(|error| format!("无法写入规则文件 {}: {error}", rule_file_path.display()))?;

    Ok(RuleWriteResult {
        action: if matching_rule_found {
            RuleWriteAction::Updated
        } else {
            RuleWriteAction::Added
        },
        rule_line,
    })
}

pub fn build_rule_line(request: &RuleUpdateRequest) -> String {
    // 使用 SUBSYSTEM=="tty" 可以同时覆盖 ttyUSB* 和 ttyACM*，
    // 比原始 C++ 示例里固定匹配 ttyUSB* 更稳妥。
    format!(
        "SUBSYSTEM==\"tty\", KERNELS==\"{}\", MODE:=\"0664\", SYMLINK+=\"{}\"",
        request.physical_id.trim(),
        request.virtual_name.trim()
    )
}

fn read_rule_lines(rule_file_path: &Path) -> Result<Vec<String>, String> {
    if !rule_file_path.exists() {
        return Ok(Vec::new());
    }

    let file_content = fs::read_to_string(rule_file_path)
        .map_err(|error| format!("无法读取规则文件 {}: {error}", rule_file_path.display()))?;

    Ok(file_content
        .lines()
        .map(str::to_owned)
        .filter(|line| !line.trim().is_empty())
        .collect())
}

fn ensure_parent_directory(rule_file_path: &Path) -> Result<(), String> {
    let Some(parent_directory) = rule_file_path.parent() else {
        return Ok(());
    };

    if parent_directory.exists() {
        return Ok(());
    }

    fs::create_dir_all(parent_directory).map_err(|error| {
        format!(
            "无法创建规则文件目录 {}: {error}",
            parent_directory.display()
        )
    })
}

pub fn reload_udev_rules() -> Result<(), String> {
    let output = Command::new("sudo")
        .args(["udevadm", "control", "--reload-rules"])
        .output()
        .map_err(|e| format!("执行 udevadm control --reload-rules 失败: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("udevadm control --reload-rules 失败: {stderr}"));
    }

    let output = Command::new("sudo")
        .args(["udevadm", "trigger"])
        .output()
        .map_err(|e| format!("执行 udevadm trigger 失败: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("udevadm trigger 失败: {stderr}"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;
    use crate::models::RuleUpdateRequest;

    #[test]
    fn add_rule_when_file_does_not_exist() {
        let file_path = unique_test_path("add_rule");
        let request = RuleUpdateRequest::new(
            "ttyCAN".into(),
            "1-4.5:1.0".into(),
            file_path.display().to_string(),
        );

        let result = update_rule_file(&request).expect("rule file should be created");
        let file_content = fs::read_to_string(&file_path).expect("rule file should be readable");

        assert_eq!(result.action, RuleWriteAction::Added);
        assert!(file_content.contains("SYMLINK+=\"ttyCAN\""));
    }

    #[test]
    fn update_existing_rule_with_same_physical_id() {
        let file_path = unique_test_path("update_rule");
        fs::write(
            &file_path,
            "SUBSYSTEM==\"tty\", KERNELS==\"1-4.5:1.0\", MODE:=\"0664\", SYMLINK+=\"old-name\"\n",
        )
        .expect("seed file should be written");

        let request = RuleUpdateRequest::new(
            "ttyLIS".into(),
            "1-4.5:1.0".into(),
            file_path.display().to_string(),
        );

        let result = update_rule_file(&request).expect("rule file should be updated");
        let file_content = fs::read_to_string(&file_path).expect("rule file should be readable");

        assert_eq!(result.action, RuleWriteAction::Updated);
        assert!(file_content.contains("SYMLINK+=\"ttyLIS\""));
        assert!(!file_content.contains("old-name"));
    }

    fn unique_test_path(test_name: &str) -> std::path::PathBuf {
        let unique_suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();

        std::env::temp_dir().join(format!("usb-map-gui-{test_name}-{unique_suffix}.rules"))
    }
}
