use crate::config::VmConfig;
use std::path::Path;

pub struct QemuArgs;

impl QemuArgs {
    pub fn build(cfg: &VmConfig, base_dir: &Path) -> Vec<String> {
        let mut args = Vec::new();

        // 1. Tên máy ảo
        args.push("-name".to_string());
        args.push(cfg.name.clone());

        // 2. Loại máy (pc i440fx cho máy yếu / q35 cho máy hiện đại)
        let mach = if cfg.machine_type.trim().is_empty() {
            if cfg.os_type == "slitaz" || cfg.low_spec_mode {
                "pc"
            } else {
                "q35"
            }
        } else if cfg.low_spec_mode {
            "pc"
        } else {
            cfg.machine_type.as_str()
        };

        args.push("-machine".to_string());
        if cfg.accel == "whpx" {
            // WHPX tăng tốc phần cứng
            args.push(format!("{},accel=whpx:tcg", mach));
            args.push("-cpu".to_string());
            args.push("qemu64".to_string()); // qemu64 tương thích 100% với WHPX trên Windows, tránh crash CPU host
        } else {
            // TCG mô phỏng: tương thích 100% không lo lỗi CPU
            args.push(format!("{},accel=tcg", mach));
            args.push("-cpu".to_string());
            args.push("qemu64".to_string());
        }

        // 3. Số nhân CPU & RAM
        let cores = if cfg.low_spec_mode {
            cfg.cpu_cores.min(2).max(1)
        } else {
            cfg.cpu_cores.max(1)
        };
        args.push("-smp".to_string());
        args.push(format!("cores={}", cores));

        let mem = if cfg.low_spec_mode {
            cfg.memory_mb.min(1024).max(256)
        } else {
            cfg.memory_mb.max(256)
        };
        args.push("-m".to_string());
        args.push(format!("{}M", mem));

        // 4. Đồng hồ thời gian thực RTC ổn định, tránh trôi nhịp/lỗi timer trên máy cấu hình yếu
        args.push("-rtc".to_string());
        args.push("base=localtime,clock=host".to_string());

        // 5. Bàn phím & Chuột tuyệt đối (USB Tablet không bị kẹt chuột)
        if mach == "q35" {
            args.push("-device".to_string());
            args.push("nec-usb-xhci".to_string());
        }
        args.push("-device".to_string());
        args.push("usb-tablet".to_string());

        // 6. Card âm thanh (chỉ bật khi không ở chế độ máy yếu)
        if cfg.sound_enabled && !cfg.low_spec_mode {
            args.push("-device".to_string());
            args.push("intel-hda".to_string());
            args.push("-device".to_string());
            args.push("hda-duplex".to_string());
        }

        // 7. Card đồ họa (std, cirrus, virtio, qxl)
        let vga = if cfg.low_spec_mode {
            "std"
        } else if cfg.vga_card.trim().is_empty() {
            "std"
        } else {
            cfg.vga_card.as_str()
        };
        args.push("-vga".to_string());
        args.push(vga.to_string());

        // 8. Mạng (e1000 tương thích tối đa hoặc virtio-net-pci)
        args.push("-netdev".to_string());
        args.push("user,id=net0".to_string());
        args.push("-device".to_string());
        if cfg.low_spec_mode || cfg.os_type == "slitaz" {
            args.push("e1000,netdev=net0".to_string());
        } else {
            args.push("virtio-net-pci,netdev=net0".to_string());
        }

        // 9. Ổ đĩa ảo chính
        if let Some(disk_path) = cfg.primary_disk_path(base_dir) {
            args.push("-drive".to_string());
            let drive_if = if cfg.low_spec_mode || cfg.os_type == "slitaz" {
                "ide"
            } else {
                "virtio"
            };
            args.push(format!(
                "file={},if={},format=qcow2,cache=writeback",
                disk_path.to_string_lossy(),
                drive_if
            ));
        }

        // 10. Ổ đĩa CD-ROM cài đặt (ISO)
        if let Some(ref iso) = cfg.iso_path {
            if !iso.trim().is_empty() && std::path::Path::new(iso).exists() {
                args.push("-cdrom".to_string());
                args.push(iso.clone());
            }
        }

        // 11. Menu Boot & Thứ tự Boot (ưu tiên CD-ROM trước)
        args.push("-boot".to_string());
        args.push("order=dcn,menu=on".to_string());

        // 12. Bộ hiển thị giao diện SDL (DirectX/Win32 Native)
        match cfg.display_type.as_str() {
            "gtk" => {
                args.push("-display".to_string());
                args.push("gtk,zoom-to-fit=on".to_string());
            }
            "sdl" | _ => {
                args.push("-display".to_string());
                args.push("sdl".to_string());
            }
        }

        args
    }
}
