use serde::{Deserialize, Serialize};
use std::path::PathBuf;
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
    // Đơn giản hóa timestamp dạng epoch giây
    format!("{}", secs)
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
    pub os_type: String, // "linux", "windows", "android", "custom"
    pub os_variant: String, // e.g. "ubuntu", "win11", "android-x86"
    pub arch: String, // "x86_64", "aarch64"
    pub cpu_cores: u32,
    pub memory_mb: u32,
    pub accel: String, // "whpx", "tcg"
    pub drives: Vec<VmDrive>,
    pub iso_path: Option<String>,
    pub shared_dir: Option<String>,
    pub display_type: String, // "gtk", "sdl", "vnc"
    pub qmp_port: u16,
    pub created_at: String,
    pub notes: String,
}

impl VmConfig {
    pub fn new_template(name: &str, os_type: &str, iso: Option<String>) -> Self {
        let id = generate_id("vm");
        let created_at = current_timestamp_str();

        let (os_variant, cpu_cores, memory_mb, disk_size_gb) = match os_type {
            "windows" => ("win11", 4, 8192, 64),
            "android" => ("android-x86", 2, 4096, 16),
            "slitaz" => ("slitaz", 1, 1024, 4),
            "linux" | _ => ("ubuntu", 2, 4096, 32),
        };

        let drive_id = generate_id("disk");
        let disk_filename = format!("{}.qcow2", name.to_lowercase().replace(' ', "-"));

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
            accel: "whpx".to_string(),
            drives,
            iso_path: iso,
            shared_dir: None,
            display_type: "gtk".to_string(),
            qmp_port: 0,
            created_at,
            notes: format!("Tạo bởi AeroVM"),
        }
    }

    pub fn vm_dir(&self, base_dir: &std::path::Path) -> PathBuf {
        base_dir.join(&self.id)
    }

    pub fn primary_disk_path(&self, base_dir: &std::path::Path) -> Option<PathBuf> {
        self.drives.iter().find(|d| !d.is_cdrom).map(|d| {
            let p = std::path::Path::new(&d.path);
            if p.is_absolute() {
                p.to_path_buf()
            } else {
                self.vm_dir(base_dir).join(&d.path)
            }
        })
    }
}
