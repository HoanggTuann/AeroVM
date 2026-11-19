use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn generate_id(prefix: &str) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{}-{:016x}", prefix, now)
}

fn current_timestamp_str() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{}", secs)
}

fn default_machine() -> String {
    "pc".to_string()
}

fn default_vga() -> String {
    "std".to_string()
}

fn default_true() -> bool {
    true
}

fn default_display() -> String {
    "sdl".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmDrive {
    pub id: String,
    pub path: String,
    pub size_gb: u32,
    pub format: String, // "qcow2", "raw"
    pub is_cdrom: bool,
    pub boot_index: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmConfig {
    pub id: String,
    pub name: String,
    pub os_type: String, // "linux", "windows", "android", "slitaz", "custom"
    pub os_variant: String, // e.g. "ubuntu", "win11", "android-x86", "slitaz"
    pub arch: String, // "x86_64", "aarch64"
    pub cpu_cores: u32,
    pub memory_mb: u32,
    pub accel: String, // "whpx", "tcg"
    
    #[serde(default = "default_machine")]
    pub machine_type: String, // "pc" (i440fx - nhẹ cho máy yếu) hoặc "q35" (PCIe hiện đại)

    #[serde(default = "default_vga")]
    pub vga_card: String, // "std", "cirrus", "virtio", "qxl"

    #[serde(default)]
    pub low_spec_mode: bool, // Tối ưu máy yếu: 1 CPU, 512MB RAM, tắt audio phức tạp

    #[serde(default = "default_true")]
    pub sound_enabled: bool,

    pub drives: Vec<VmDrive>,
    pub iso_path: Option<String>,
    pub shared_dir: Option<String>,

    #[serde(default = "default_display")]
    pub display_type: String, // "sdl", "gtk"

    pub qmp_port: u16,
    pub created_at: String,
    pub notes: String,
}

impl VmConfig {
    pub fn new_template(name: &str, os_type: &str, iso: Option<String>) -> Self {
        let id = generate_id("vm");
        let created_at = current_timestamp_str();

        let (os_variant, cpu_cores, memory_mb, disk_size_gb, accel, machine_type, low_spec, sound) = match os_type {
            "windows" => ("win11", 4, 4096, 64, "whpx", "q35", false, true),
            "android" => ("android-x86", 2, 2048, 16, "whpx", "pc", false, true),
            "slitaz" => ("slitaz", 1, 512, 4, "tcg", "pc", true, false),
            "linux" | _ => ("ubuntu", 2, 2048, 32, "whpx", "pc", false, true),
        };

        let safe_name = name
            .to_lowercase()
            .replace(' ', "-")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect::<String>();
        let disk_filename = format!("{}.qcow2", if safe_name.is_empty() { "disk".to_string() } else { safe_name });

        let drive_id = generate_id("disk");
        let drives = vec![VmDrive {
            id: drive_id,
            path: disk_filename,
            size_gb: disk_size_gb,
            format: "qcow2".to_string(),
            is_cdrom: false,
            boot_index: 1,
        }];

        Self {
            id,
            name: name.to_string(),
            os_type: os_type.to_string(),
            os_variant: os_variant.to_string(),
            arch: "x86_64".to_string(),
            cpu_cores,
            memory_mb,
            accel: accel.to_string(),
            machine_type: machine_type.to_string(),
            vga_card: "std".to_string(),
            low_spec_mode: low_spec,
            sound_enabled: sound,
            drives,
            iso_path: iso,
            shared_dir: None,
            display_type: "sdl".to_string(),
            qmp_port: 0,
            created_at,
            notes: "Tạo bởi AeroVM".to_string(),
        }
    }

    pub fn primary_disk_path(&self, base_dir: &Path) -> Option<PathBuf> {
        let vm_dir = base_dir.join(&self.id);
        self.drives.iter().find(|d| !d.is_cdrom).map(|d| {
            if Path::new(&d.path).is_absolute() {
                PathBuf::from(&d.path)
            } else {
                vm_dir.join(&d.path)
            }
        })
    }
}
