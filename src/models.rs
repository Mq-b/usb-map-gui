use std::path::PathBuf;

pub const DEFAULT_RULE_FILE_PATH: &str = "/etc/udev/rules.d/relia.rules";
pub const UNKNOWN_INTERFACE_ID: &str = "N/A";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceListFilter {
    AllDevices,
    VirtualLinksOnly,
    PhysicalDevicesOnly,
}

impl DeviceListFilter {
    pub fn label(self) -> &'static str {
        match self {
            Self::AllDevices => "全部设备",
            Self::VirtualLinksOnly => "仅虚拟链接",
            Self::PhysicalDevicesOnly => "仅物理设备",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceRowKind {
    VirtualLink,
    PhysicalDevice,
}

impl DeviceRowKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::VirtualLink => "虚拟链接",
            Self::PhysicalDevice => "物理设备",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SerialDeviceEntry {
    pub row_kind: DeviceRowKind,
    pub virtual_name: String,
    pub physical_name: String,
    pub interface_id: String,
}

impl SerialDeviceEntry {
    pub fn matches_filter(&self, filter: DeviceListFilter) -> bool {
        match filter {
            DeviceListFilter::AllDevices => true,
            DeviceListFilter::VirtualLinksOnly => self.row_kind == DeviceRowKind::VirtualLink,
            DeviceListFilter::PhysicalDevicesOnly => self.row_kind == DeviceRowKind::PhysicalDevice,
        }
    }

    pub fn can_fill_rule(&self) -> bool {
        self.interface_id != UNKNOWN_INTERFACE_ID
    }

    pub fn suggested_virtual_name(&self) -> String {
        if self.row_kind == DeviceRowKind::VirtualLink && self.virtual_name != "-" {
            return self.virtual_name.clone();
        }

        String::new()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleUpdateRequest {
    pub virtual_name: String,
    pub physical_id: String,
    pub rule_file_path: PathBuf,
}

impl RuleUpdateRequest {
    pub fn new(virtual_name: String, physical_id: String, rule_file_path: String) -> Self {
        Self {
            virtual_name,
            physical_id,
            rule_file_path: PathBuf::from(rule_file_path),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.virtual_name.trim().is_empty() {
            return Err("虚拟设备名称不能为空。".into());
        }

        if self.virtual_name.contains('/') {
            return Err("虚拟设备名称只需要填写设备名，例如 ttyCAN。".into());
        }

        if self.physical_id.trim().is_empty() || self.physical_id == UNKNOWN_INTERFACE_ID {
            return Err("物理接口 ID 不能为空，且不能是 N/A。".into());
        }

        if self.rule_file_path.as_os_str().is_empty() {
            return Err("规则文件路径不能为空。".into());
        }

        Ok(())
    }
}
