let vms = [];
let currentVmId = null;
let selectedWizardOs = 'linux';

function showToast(msg, icon = 'ℹ️') {
  const container = document.getElementById('toastContainer');
  const t = document.createElement('div');
  t.className = 'toast';
  t.innerHTML = `<span>${icon}</span> <span>${msg}</span>`;
  container.appendChild(t);
  setTimeout(() => {
    t.style.opacity = '0';
    t.style.transition = 'opacity 0.3s';
    setTimeout(() => t.remove(), 300);
  }, 3500);
}

async function fetchVms() {
  try {
    const res = await fetch('/api/vms');
    vms = await res.json();
    renderSidebar();
    if (currentVmId) {
      const found = vms.find(v => v.id === currentVmId);
      if (found) selectVm(found.id);
    }
  } catch (e) {
    console.error("Lỗi fetch VMs:", e);
  }
}

function renderSidebar() {
  const c = document.getElementById('vmListContainer');
  c.innerHTML = '';
  if (!vms.length) {
    c.innerHTML = '<div style="padding: 20px; font-size: 12.5px; color: var(--text-muted); text-align: center;">Chưa có máy ảo nào</div>';
    return;
  }

  vms.forEach(vm => {
    const item = document.createElement('div');
    item.className = 'vm-item' + (vm.id === currentVmId ? ' active' : '');
    item.onclick = () => selectVm(vm.id);

    let icon = '🐧';
    if (vm.os_type === 'windows') icon = '🪟';
    else if (vm.os_type === 'android') icon = '🤖';
    else if (vm.os_type === 'slitaz') icon = '⚡';

    const accelClass = vm.accel === 'whpx' ? 'whpx' : 'tcg';
    const accelText = vm.accel === 'whpx' ? 'WHPX' : 'TCG';

    item.innerHTML = `
      <div class="vm-os-icon">${icon}</div>
      <div class="vm-info">
        <div class="vm-name">${vm.name}</div>
        <div class="vm-meta">
          <span class="badge-accel ${accelClass}">${accelText}</span>
          <span>${vm.cpu_cores}C · ${vm.memory_mb}M</span>
        </div>
      </div>
      <div class="status-dot ${vm.running ? 'running' : ''}" title="${vm.running ? 'Đang chạy' : 'Đã tắt'}"></div>
    `;
    c.appendChild(item);
  });
}

function selectVm(id) {
  currentVmId = id;
  renderSidebar();
  const vm = vms.find(v => v.id === id);
  if (!vm) return;

  document.getElementById('emptyStateView').style.display = 'none';
  document.getElementById('topToolbar').style.display = 'flex';
  document.getElementById('navTabs').style.display = 'flex';
  document.getElementById('workspaceContent').style.display = 'block';

  // Toolbar Header
  document.getElementById('vmTitle').innerText = vm.name;
  const badge = document.getElementById('vmStatusBadge');
  const btnStart = document.getElementById('btnStartVm');
  const btnStop = document.getElementById('btnStopVm');

  if (vm.running) {
    badge.className = 'badge-status running';
    badge.innerText = 'Đang chạy';
    btnStart.style.display = 'none';
    btnStop.style.display = 'flex';
  } else {
    badge.className = 'badge-status';
    badge.innerText = 'Đã dừng';
    btnStart.style.display = 'flex';
    btnStop.style.display = 'none';
  }

  // Populate Overview Tab
  const isWhpx = vm.accel === 'whpx';
  document.getElementById('specAccel').innerText = isWhpx ? '⚡ WHPX Hardware' : '🔄 TCG Emulation';
  document.getElementById('specAccelDesc').innerText = isWhpx
    ? 'Tận dụng công nghệ ảo hóa phần cứng Intel VT-x / AMD-V tốc độ tối đa.'
    : 'Mô phỏng phần mềm tương thích 100%, bảo đảm không lỗi ngắt cho SliTaz và Linux cổ.';

  document.getElementById('specCpuRam').innerText = `${vm.cpu_cores} Nhân · ${vm.memory_mb} MB RAM`;
  document.getElementById('specCpuRamDesc').innerText = vm.low_spec_mode
    ? 'Chế độ máy cấu hình yếu đang BẬT: Đã tối ưu hóa CPU và RAM an toàn.'
    : 'Cấp phát chuẩn cho hiệu năng tốt.';

  document.getElementById('specDisk').innerText = `${vm.disk_size_gb} GB QCOW2`;
  document.getElementById('specIso').innerText = vm.iso_path ? vm.iso_path : 'Chưa gắn tệp ISO';
  document.getElementById('specFolder').innerText = vm.folder_path || '~/.aerovm/vms/' + vm.id;

  // Populate Settings Tab
  populateSettingsForm(vm);

  // Load Logs if active
  if (document.getElementById('pane-logs').classList.contains('active')) {
    loadLogs();
  }
}

function populateSettingsForm(vm) {
  document.getElementById('cfgName').value = vm.name;
  document.getElementById('cfgCores').value = vm.cpu_cores;
  document.getElementById('cfgRam').value = vm.memory_mb;
  document.getElementById('cfgMachine').value = vm.machine_type || 'pc';
  document.getElementById('cfgVga').value = vm.vga_card || 'std';
  document.getElementById('cfgLowSpec').checked = !!vm.low_spec_mode;
  document.getElementById('cfgSound').checked = vm.sound_enabled !== false;
  document.getElementById('cfgIso').value = vm.iso_path || '';

  setSettingEngine(vm.accel || 'whpx');
  updateSettingSliders();
}

let activeEngine = 'whpx';
function setSettingEngine(mode) {
  activeEngine = mode;
  const btnWhpx = document.getElementById('btnEngineWhpx');
  const btnTcg = document.getElementById('btnEngineTcg');
  const note = document.getElementById('engineNote');

  if (mode === 'whpx') {
    btnWhpx.className = 'segment-btn active';
    btnTcg.className = 'segment-btn tcg';
    note.innerHTML = '* <b>WHPX</b>: Tăng tốc phần cứng cho Ubuntu, Windows, Android. (Cần Intel VT-x/AMD-V bật trong BIOS)';
  } else {
    btnWhpx.className = 'segment-btn';
    btnTcg.className = 'segment-btn active tcg';
    note.innerHTML = '* <b>TCG</b>: Mô phỏng tương thích 100%. Khuyên dùng tuyệt đối cho <b>SliTaz Live</b> và Linux cổ.';
  }
}

function updateSettingSliders() {
  const cores = document.getElementById('cfgCores').value;
  const ram = document.getElementById('cfgRam').value;
  document.getElementById('badgeCores').innerText = cores + ' Cores';
  document.getElementById('badgeRam').innerText = ram + ' MB';
}

function switchTab(tabId) {
  document.querySelectorAll('.nav-tab').forEach(t => {
    t.classList.toggle('active', t.getAttribute('data-tab') === tabId);
  });
  document.querySelectorAll('.tab-pane').forEach(p => {
    p.classList.toggle('active', p.id === 'pane-' + tabId);
  });
  if (tabId === 'logs') loadLogs();
}

async function saveVmSettings() {
  if (!currentVmId) return;
  const vm = vms.find(v => v.id === currentVmId);
  if (vm && vm.running) {
    showToast('Máy ảo đang chạy! Vui lòng dừng máy ảo trước khi đổi cài đặt.', '⚠️');
    return;
  }

  const payload = {
    name: document.getElementById('cfgName').value.trim() || 'AeroVM Guest',
    cpu_cores: parseInt(document.getElementById('cfgCores').value) || 2,
    memory_mb: parseInt(document.getElementById('cfgRam').value) || 2048,
    accel: activeEngine,
    machine_type: document.getElementById('cfgMachine').value,
    vga_card: document.getElementById('cfgVga').value,
    low_spec_mode: document.getElementById('cfgLowSpec').checked,
    sound_enabled: document.getElementById('cfgSound').checked,
    iso_path: document.getElementById('cfgIso').value.trim() || null
  };

  try {
    const res = await fetch(`/api/vms/${currentVmId}/update`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload)
    });
    const data = await res.json();
    if (data.ok) {
      showToast('Đã lưu thành công cấu hình máy ảo!', '✅');
      await fetchVms();
    } else {
      showToast('Lỗi lưu cấu hình: ' + data.error, '❌');
    }
  } catch (e) {
    showToast('Lỗi kết nối máy chủ: ' + e.message, '❌');
  }
}

function resetSettings() {
  const vm = vms.find(v => v.id === currentVmId);
  if (vm) populateSettingsForm(vm);
  showToast('Đã khôi phục thông số ban đầu', '🔄');
}

async function startCurrentVm() {
  if (!currentVmId) return;
  showToast('Đang khởi động máy ảo qua Native SDL...', '🚀');
  try {
    const res = await fetch(`/api/vms/${currentVmId}/start`, { method: 'POST' });
    const data = await res.json();
    if (data.ok) {
      showToast('Máy ảo đã khởi động thành công!', '✅');
      await fetchVms();
    } else {
      showToast('Lỗi khởi động: ' + data.error, '❌');
    }
  } catch (e) {
    showToast('Lỗi khởi động: ' + e.message, '❌');
  }
}

async function stopCurrentVm() {
  if (!currentVmId) return;
  showToast('Đang tắt máy ảo...', '⏹');
  try {
    const res = await fetch(`/api/vms/${currentVmId}/stop`, { method: 'POST' });
    const data = await res.json();
    if (data.ok) {
      showToast('Máy ảo đã dừng an toàn.', 'ℹ️');
      await fetchVms();
    } else {
      showToast('Lỗi dừng máy ảo: ' + data.error, '❌');
    }
  } catch (e) {
    showToast('Lỗi dừng: ' + e.message, '❌');
  }
}

async function openVmFolder() {
  if (!currentVmId) return;
  try {
    await fetch(`/api/vms/${currentVmId}/open_folder`, { method: 'POST' });
    showToast('Đã mở thư mục máy ảo trong Windows Explorer', '📂');
  } catch (e) {
    showToast('Không thể mở thư mục: ' + e.message, '❌');
  }
}

async function deleteCurrentVm() {
  if (!currentVmId) return;
  if (!confirm('Bạn có chắc chắn muốn xóa máy ảo này cùng toàn bộ ổ đĩa ảo không?')) return;
  try {
    const res = await fetch(`/api/vms/${currentVmId}`, { method: 'DELETE' });
    const data = await res.json();
    if (data.ok) {
      showToast('Đã xóa máy ảo thành công.', '🗑️');
      currentVmId = null;
      document.getElementById('topToolbar').style.display = 'none';
      document.getElementById('navTabs').style.display = 'none';
      document.getElementById('workspaceContent').style.display = 'none';
      document.getElementById('emptyStateView').style.display = 'flex';
      await fetchVms();
    } else {
      showToast('Lỗi xóa máy ảo: ' + data.error, '❌');
    }
  } catch (e) {
    showToast('Lỗi xóa: ' + e.message, '❌');
  }
}

async function loadLogs() {
  if (!currentVmId) return;
  try {
    const res = await fetch(`/api/vms/${currentVmId}/logs`);
    const data = await res.json();
    const box = document.getElementById('logContent');
    box.innerText = data.logs || 'Chưa có nhật ký hoạt động.';
    box.scrollTop = box.scrollHeight;
  } catch (e) {
    document.getElementById('logContent').innerText = 'Không thể nạp nhật ký: ' + e.message;
  }
}

// Process Monitor Modal
async function showProcModal() {
  document.getElementById('procModal').classList.add('open');
  try {
    const res = await fetch('/api/processes');
    const data = await res.json();
    const list = document.getElementById('procList');
    list.innerHTML = '';
    if (!data.processes || !data.processes.length) {
      list.innerHTML = '<div style="font-size: 13px; color: var(--text-muted); padding: 12px; text-align: center;">Hiện không có tiến trình QEMU nào đang chạy ngầm.</div>';
      return;
    }
    data.processes.forEach(p => {
      const row = document.createElement('div');
      row.style.cssText = 'display:flex; justify-content:space-between; align-items:center; background:rgba(0,0,0,0.3); padding:10px 14px; border-radius:8px; border:1px solid var(--card-border);';
      row.innerHTML = `
        <div>
          <div style="font-size: 13.5px; font-weight: 600;">${p.name}</div>
          <div style="font-size: 11.5px; color: var(--text-sub);">PID: ${p.pid} · ${p.status}</div>
        </div>
        <button class="btn-action btn-stop" style="height:28px; font-size:11px;" onclick="stopProc('${p.id}')">Dừng</button>
      `;
      list.appendChild(row);
    });
  } catch (e) {
    console.error("Lỗi lấy tiến trình:", e);
  }
}

function closeProcModal() {
  document.getElementById('procModal').classList.remove('open');
}

async function stopProc(id) {
  try {
    await fetch(`/api/vms/${id}/stop`, { method: 'POST' });
    showToast('Đã dừng tiến trình máy ảo', '⏹');
    showProcModal();
    fetchVms();
  } catch (e) {}
}

async function stopAllProcesses() {
  try {
    await fetch('/api/processes/stop_all', { method: 'POST' });
    showToast('Đã dừng tất cả tiến trình QEMU ngầm', '🛑');
    showProcModal();
    fetchVms();
  } catch (e) {}
}

// Wizard Create VM
const btnOpenNewModal = document.getElementById('btnOpenNewModal');
if (btnOpenNewModal) {
  btnOpenNewModal.onclick = () => {
    document.getElementById('wizardModal').classList.add('open');
  };
}

function closeWizard() {
  document.getElementById('wizardModal').classList.remove('open');
}

function selectWizardOs(el, os) {
  document.querySelectorAll('.os-option').forEach(o => o.classList.remove('selected'));
  el.classList.add('selected');
  selectedWizardOs = os;

  const nameInput = document.getElementById('newVmName');
  const ramSlider = document.getElementById('newVmRam');
  const coreSlider = document.getElementById('newVmCores');
  const diskSlider = document.getElementById('newVmDisk');
  const lowSpec = document.getElementById('newVmLowSpec');

  if (os === 'windows') {
    nameInput.value = 'Windows 11 Pro';
    coreSlider.value = 4;
    ramSlider.value = 4096;
    diskSlider.value = 64;
    lowSpec.checked = false;
  } else if (os === 'android') {
    nameInput.value = 'Android-x86';
    coreSlider.value = 2;
    ramSlider.value = 2048;
    diskSlider.value = 16;
    lowSpec.checked = false;
  } else if (os === 'slitaz') {
    nameInput.value = 'SliTaz GNU/Linux (Live)';
    coreSlider.value = 1;
    ramSlider.value = 512;
    diskSlider.value = 4;
    lowSpec.checked = true;
  } else {
    nameInput.value = 'Ubuntu 24.04 LTS';
    coreSlider.value = 2;
    ramSlider.value = 2048;
    diskSlider.value = 32;
    lowSpec.checked = false;
  }

  document.getElementById('lblNewCores').innerText = coreSlider.value + ' Cores';
  document.getElementById('lblNewRam').innerText = ramSlider.value + ' MB';
  document.getElementById('lblNewDisk').innerText = diskSlider.value + ' GB';
}

function toggleWizardLowSpec(checked) {
  if (checked) {
    document.getElementById('newVmCores').value = 1;
    document.getElementById('newVmRam').value = 512;
    document.getElementById('lblNewCores').innerText = '1 Cores';
    document.getElementById('lblNewRam').innerText = '512 MB';
  }
}

async function submitCreateVm() {
  const payload = {
    name: document.getElementById('newVmName').value.trim() || 'AeroVM Guest',
    os_type: selectedWizardOs,
    iso: document.getElementById('newVmIso').value.trim() || null,
    cpu_cores: parseInt(document.getElementById('newVmCores').value) || 2,
    memory_mb: parseInt(document.getElementById('newVmRam').value) || 2048,
    disk_size_gb: parseInt(document.getElementById('newVmDisk').value) || 32,
    low_spec_mode: document.getElementById('newVmLowSpec').checked
  };

  try {
    const res = await fetch('/api/vms', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload)
    });
    const data = await res.json();
    if (data.ok) {
      closeWizard();
      showToast('Đã tạo máy ảo mới thành công!', '🎉');
      await fetchVms();
      selectVm(data.vm.id);
    } else {
      showToast('Lỗi tạo máy ảo: ' + data.error, '❌');
    }
  } catch (e) {
    showToast('Lỗi kết nối: ' + e.message, '❌');
  }
}

async function exitApp() {
  if (confirm('Bạn có muốn đóng AeroVM và tắt tất cả máy ảo đang chạy không?')) {
    await fetch('/api/app/exit', { method: 'POST' });
    window.close();
  }
}

window.addEventListener('beforeunload', () => {
  navigator.sendBeacon('/api/app/exit');
});

setInterval(() => {
  fetch('/api/app/ping', { method: 'POST' }).catch(() => {});
}, 5000);

// Polling refresh every 3 seconds
fetchVms();
setInterval(fetchVms, 3000);
