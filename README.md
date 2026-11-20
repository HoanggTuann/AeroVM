# 🚀 AeroVM (Phong cách UTM cho Windows - 100% Native Desktop)

> **Trình quản lý máy ảo hiện đại, siêu nhẹ, viết 100% bằng Rust với giao diện Native Desktop phong cách UTM.**

---

## ✨ Điểm nổi bật phiên bản v0.1.5

* 🖥️ **100% Native Win32/DirectX Desktop GUI**: Hoàn toàn không phụ thuộc vào trình duyệt web Edge/Chrome, không dùng máy chủ localhost (`127.0.0.1:45678`), loại bỏ triệt để lỗi kết nối trình duyệt.
* 🦀 **100% Rust Backend & Native GUI**: Tốc độ phản hồi tức thì, tiêu thụ cực ít RAM (< 25 MB).
* 🍏 **Bố cục UTM Studio**: Thanh bên danh sách máy ảo, thanh công cụ trên cùng (Khởi động, Dừng, Cài đặt, Thư mục, Xóa), bảng Cài đặt chi tiết với nút `❌ Thoát` chuyên biệt.
* ⚙️ **Chuyển đổi và tùy biến cài đặt máy ảo linh hoạt**: Dễ dàng thay đổi số nhân CPU, dung lượng RAM, đĩa ảo, card màn hình VGA, âm thanh và file ISO ngay trong giao diện, lưu trực tiếp vào cấu hình.
* ❌ **Nút Thoát chuyên biệt**: Đóng màn hình cài đặt bất cứ lúc nào, hỗ trợ khung viền và nút điều khiển chuẩn Windows (không dùng chấm tròn macOS).
* 📦 **Trình cài đặt riêng biệt (`install.bat`)**: Cài đặt ứng dụng vào thư mục riêng biệt `%LOCALAPPDATA%\Programs\AeroVM`, tạo lối tắt Desktop và Start Menu 1 chạm.
* ⚡ **Gộp tiến trình & Chạy ngầm 100%**: 0 cửa sổ console đen cmd phiền toái trên Desktop, kiểm soát toàn bộ tiến trình ảo hóa ngầm.
* 🐢 **Chế độ máy cấu hình yếu (Low-Spec Mode)**: Tự động tối ưu hóa tài nguyên cho máy tính RAM/CPU thấp chạy mượt mà không bị treo đơ.
* ⚡ **Tăng tốc phần cứng WHPX & TCG Mode**: Tận dụng trực tiếp **Windows Hypervisor Platform** (`-accel whpx,kernel-irqchip=off`) chống lỗi thoát VP exit code 4, đồng thời hỗ trợ chuyển đổi tức thì sang chế độ **TCG Software Emulation** tương thích 100% cho mọi máy.
* 🖥️ **Hiển thị trực tiếp SDL mượt mà**: Chuyển đổi sang backend hiển thị SDL (`-display sdl`), mở màn hình đồ họa trực tiếp.
* 🧙‍♂️ **Wizard tạo máy ảo thông minh**:
  * Hỗ trợ cài đặt nhanh cho **Linux** (Ubuntu, Debian, Arch...), **Windows** (Win 10, Win 11), **Android** (Android-x86, BlissOS), **SliTaz**.
  * Tùy chỉnh thanh trượt số nhân CPU, dung lượng RAM, kích thước ổ đĩa.
* 💾 **Ổ cứng ảo QCOW2 động**: Tự động tạo đĩa nén dung lượng động qua `qemu-img` (cấp 64 GB ban đầu chỉ tốn vài trăm KB thực tế).
* 🖱️ **Không bị kẹt chuột**: Tích hợp sẵn chuẩn USB Tablet VirtIO, di chuyển chuột giữa máy tính thực và máy ảo mượt mà tự nhiên.

---

## 📂 Cấu trúc thư mục

* `src/main.rs`: Điểm khởi chạy ứng dụng, khởi tạo Native GUI qua eframe.
* `src/ui/app.rs`: Giao diện Native Desktop UTM Studio, font Segoe UI tiếng Việt, bảng điều khiển và cài đặt.
* `src/config.rs`: Định nghĩa cấu hình máy ảo, phần cứng, ổ đĩa.
* `src/manager.rs`: Quản lý tiến trình VM (Start, Stop, Delete, trạng thái Running).
* `src/qemu/`:
  * `args.rs`: Trình sinh tham số QEMU tối ưu hóa cho WHPX & VirtIO.
  * `img.rs`: Công cụ tạo đĩa QCOW2 động.
* `~/.aerovm/`:
  * `qemu/`: Bộ công cụ QEMU Portable 64-bit.
  * `vms/`: Nơi lưu trữ cấu hình và ổ đĩa của từng máy ảo.

---

## 🏃 Cách chạy

Nhấp đúp vào tệp `install.bat` để cài đặt vào máy hoặc chạy trực tiếp `run.bat` (hoặc `aerovm.exe`).
Cửa sổ Native Desktop AeroVM sẽ mở ra ngay lập tức!
