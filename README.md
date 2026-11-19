# 🚀 AeroVM (Phong cách UTM cho Windows)

> **Trình quản lý máy ảo hiện đại, siêu nhẹ, viết 100% bằng Rust với giao diện phong cách UTM.**

---

## ✨ Điểm nổi bật

* 🦀 **100% Rust Backend**: Tốc độ phản hồi tức thì, file thực thi chỉ **~2 MB**, tiêu thụ cực ít RAM (< 15 MB).
* 🍏 **Giao diện chuẩn phong cách UTM**: Dark glassmorphism, hiệu ứng mờ acrylic mượt mà, sidebar máy ảo trực quan.
* ⚡ **Tăng tốc phần cứng WHPX & TCG Mode**: Tận dụng trực tiếp **Windows Hypervisor Platform** (`-accel whpx`), đồng thời hỗ trợ chuyển đổi tức thì sang chế độ **TCG Software Emulation** tương thích 100% với các bản phân phối Linux siêu nhẹ (SliTaz, TinyCore...) không hỗ trợ ngắt WHPX.
* 🖥️ **Hiển thị trực tiếp SDL mượt mà**: Chuyển đổi sang backend hiển thị SDL (`-display sdl`), khắc phục hoàn toàn lỗi màn hình đen GTK (`Guest has not initialized the display`) trên Windows.
* 🧙‍♂️ **Wizard tạo máy ảo thông minh**:
  * Hỗ trợ cài đặt nhanh cho **Linux** (Ubuntu, Debian, Arch...), **Windows** (Win 10, Win 11), **Android** (Android-x86, BlissOS).
  * Tùy chỉnh thanh trượt số nhân CPU, dung lượng RAM, kích thước ổ đĩa.
* 💾 **Ổ cứng ảo QCOW2 động**: Tự động tạo đĩa nén dung lượng động qua `qemu-img` (cấp 64 GB ban đầu chỉ tốn vài trăm KB thực tế).
* 🖱️ **Không bị kẹt chuột**: Tích hợp sẵn chuẩn USB Tablet VirtIO, di chuyển chuột giữa máy tính thực và máy ảo mượt mà tự nhiên.

---

## 📂 Cấu trúc thư mục

* `src/main.rs`: Điểm khởi chạy ứng dụng, mở cửa sổ UTM Window.
* `src/config.rs`: Định nghĩa cấu hình máy ảo, phần cứng, ổ đĩa.
* `src/manager.rs`: Quản lý tiến trình VM (Start, Stop, Delete, trạng thái Running).
* `src/qemu/`:
  * `args.rs`: Trình sinh tham số QEMU tối ưu hóa cho WHPX & VirtIO.
  * `img.rs`: Công cụ tạo đĩa QCOW2 động.
* `src/ui/`:
  * `server.rs`: Máy chủ REST API tích hợp bằng Rust.
  * `assets/index.html`: Giao diện Dashboard UTM.
* `~/.aerovm/`:
  * `qemu/`: Bộ công cụ QEMU Portable 64-bit (đã tích hợp sẵn).
  * `vms/`: Nơi lưu trữ cấu hình và ổ đĩa của từng máy ảo.

---

## 🏃 Cách chạy

Nhấp đúp vào tệp `run.bat` hoặc chạy:

```bash
cargo run --release
```

Giao diện cửa sổ AeroVM sẽ tự động bật lên và sẵn sàng phục vụ!
