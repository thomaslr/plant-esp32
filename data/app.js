// Plant Monitor ESP32-C3 Client-Side Controller
let ws = null;
let pollTimer = null;

document.addEventListener('DOMContentLoaded', () => {
    initWebSocket();
    fetchStatus();
    fetchLogs();

    document.getElementById('btn-water').addEventListener('click', handleWaterClick);
    document.getElementById('btn-sensor-reset').addEventListener('click', handleResetUltrasonic);
    document.getElementById('btn-wifi-reset').addEventListener('click', handleResetWifi);
    document.getElementById('btn-refresh-logs').addEventListener('click', fetchLogs);
    document.getElementById('btn-ota-check').addEventListener('click', handleOtaCheck);
    document.getElementById('file-ota-upload').addEventListener('change', handleOtaUpload);
    fetchOtaStatus();
});

// =============================================================================
// WebSocket & HTTP Fallback
// =============================================================================
function initWebSocket() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${window.location.host}/ws`;

    try {
        ws = new WebSocket(wsUrl);

        ws.onopen = () => {
            setConnectionStatus(true);
            if (pollTimer) {
                clearInterval(pollTimer);
                pollTimer = null;
            }
        };

        ws.onmessage = (event) => {
            try {
                const data = JSON.parse(event.data);
                if (data.type === 'telemetry') {
                    updateTelemetryUI(data);
                }
            } catch (err) {
                console.error("WS Parse Error:", err);
            }
        };

        ws.onclose = () => {
            setConnectionStatus(false);
            // Fallback to HTTP polling every 5s if WebSocket drops
            if (!pollTimer) {
                pollTimer = setInterval(fetchStatus, 5000);
            }
            setTimeout(initWebSocket, 5000);
        };

        ws.onerror = () => {
            ws.close();
        };
    } catch (e) {
        setConnectionStatus(false);
        if (!pollTimer) pollTimer = setInterval(fetchStatus, 5000);
    }
}

function setConnectionStatus(connected) {
    const dot = document.getElementById('conn-status');
    const text = document.getElementById('conn-text');
    if (connected) {
        dot.className = 'dot dot-connected';
        text.innerText = 'Real-time (WS)';
    } else {
        dot.className = 'dot dot-disconnected';
        text.innerText = 'Polling (HTTP)';
    }
}

// =============================================================================
// API Requests (Strictly Relative Paths for Reverse Proxy Compatibility)
// =============================================================================
async function fetchStatus() {
    try {
        const res = await fetch('/api/status');
        if (!res.ok) throw new Error("HTTP " + res.status);
        const data = await res.json();
        updateFullStatusUI(data);
    } catch (err) {
        console.warn("Status fetch failed:", err);
    }
}

async function fetchLogs() {
    try {
        const res = await fetch('/api/sensor-log?n=15');
        if (!res.ok) throw new Error("HTTP " + res.status);
        const logs = await res.json();
        renderLogTable(logs);
    } catch (err) {
        console.warn("Log fetch failed:", err);
    }
}

async function handleWaterClick() {
    const durationInput = document.getElementById('water-duration');
    const duration = parseInt(durationInput.value, 10) || 10;
    const btn = document.getElementById('btn-water');

    btn.disabled = true;
    btn.innerText = 'Sending...';

    try {
        const res = await fetch('/api/water', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ duration: duration })
        });
        const result = await res.json();
        if (!res.ok) {
            alert("Watering Inhibited: " + (result.message || "Unknown error"));
        }
        fetchStatus();
    } catch (err) {
        alert("Watering request error: " + err.message);
    } finally {
        btn.disabled = false;
        btn.innerText = 'Water Now';
    }
}

async function handleResetUltrasonic() {
    const btn = document.getElementById('btn-sensor-reset');
    btn.disabled = true;
    btn.innerText = 'Resetting...';

    try {
        const res = await fetch('/api/sensor/ultrasonic/reset', { method: 'POST' });
        const result = await res.json();
        alert(`Sensor pin latch cleared! Preventive resets: ${result.preventive_resets}`);
        fetchStatus();
    } catch (err) {
        alert("Reset failed: " + err.message);
    } finally {
        btn.disabled = false;
        btn.innerText = 'Force Ultrasonic Reset';
    }
}

async function handleResetWifi() {
    if (!confirm("Reset Wi-Fi credentials? The device will clear saved network credentials and restart in Setup Mode.")) {
        return;
    }
    const btn = document.getElementById('btn-wifi-reset');
    btn.disabled = true;
    btn.innerText = 'Resetting Wi-Fi...';

    try {
        const res = await fetch('/api/wifi/reset', { method: 'POST' });
        const result = await res.json();
        alert(result.message || "Wi-Fi credentials cleared! Connect to the PlantMonitor-Setup hotspot.");
    } catch (err) {
        alert("Wi-Fi reset failed: " + err.message);
        btn.disabled = false;
        btn.innerText = 'Reset Wi-Fi (Setup Mode)';
    }
}

async function fetchOtaStatus() {
    try {
        const res = await fetch('/api/ota/status');
        if (!res.ok) return;
        const data = await res.json();
        if (data.firmware_version) {
            document.getElementById('diag-ota-ver').innerText = `v${data.firmware_version}`;
        }
        if (data.running_slot) {
            document.getElementById('diag-ota-slot').innerText = `${data.running_slot} (Next: ${data.next_update_slot})`;
        }
    } catch (_) {}
}

async function handleOtaCheck() {
    if (!confirm("Initiate OTA update from GitHub Releases? The device will stream the binary, flash the standby partition, and reboot.")) {
        return;
    }
    const btn = document.getElementById('btn-ota-check');
    const msg = document.getElementById('ota-msg');
    btn.disabled = true;
    btn.innerText = 'Checking...';
    msg.style.display = 'block';
    msg.style.color = '#58a6ff';
    msg.innerText = 'Initiating GitHub OTA download...';

    try {
        const res = await fetch('/api/ota/update', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: '{}' });
        const result = await res.json();
        msg.innerText = result.message || 'OTA update started! Device will reboot in ~30s.';
        setTimeout(() => {
            msg.innerText = 'Device rebooting into new firmware. Refreshing page in 15s...';
            setTimeout(() => window.location.reload(), 15000);
        }, 15000);
    } catch (err) {
        msg.style.color = '#ef4444';
        msg.innerText = 'OTA update error: ' + err.message;
        btn.disabled = false;
        btn.innerText = 'Check GitHub Update';
    }
}

async function handleOtaUpload(event) {
    const file = event.target.files[0];
    if (!file) return;

    if (!confirm(`Flash firmware binary '${file.name}' (${(file.size / 1024).toFixed(0)} KB)? The device will flash the standby partition and reboot.`)) {
        event.target.value = '';
        return;
    }

    const msg = document.getElementById('ota-msg');
    msg.style.display = 'block';
    msg.style.color = '#58a6ff';
    msg.innerText = `Uploading and flashing ${file.name}... Please wait.`;

    try {
        const res = await fetch('/api/ota/upload', {
            method: 'POST',
            body: file
        });
        const result = await res.json();
        if (!res.ok) throw new Error(result.error || 'Upload failed');
        msg.style.color = '#10b981';
        msg.innerText = result.message || 'Firmware flashed successfully! Rebooting in a moment...';
        setTimeout(() => window.location.reload(), 12000);
    } catch (err) {
        msg.style.color = '#ef4444';
        msg.innerText = 'Upload failed: ' + err.message;
    } finally {
        event.target.value = '';
    }
}

// =============================================================================
// UI Update Functions
// =============================================================================
function updateTelemetryUI(t) {
    // Soil
    document.getElementById('val-soil').innerText = `${t.soil_percent}%`;
    document.getElementById('sub-soil').innerText = `Raw: ${t.soil_raw} mV`;
    document.getElementById('bar-soil').style.width = `${Math.min(100, Math.max(0, t.soil_percent))}%`;

    // Water
    document.getElementById('val-water').innerText = `${t.water_percent}%`;
    document.getElementById('sub-water').innerText = `Depth: ${t.water_level_cm} cm`;
    document.getElementById('bar-water').style.width = `${Math.min(100, Math.max(0, t.water_percent))}%`;

    // Voltage (5V Supply Rail)
    document.getElementById('val-voltage').innerText = `${t.supply_voltage.toFixed(2)} V`;
    const voltBar = document.getElementById('bar-voltage');
    const voltHealth = document.getElementById('sub-voltage');
    const voltPct = Math.min(100, Math.max(0, ((t.supply_voltage - 4.0) / 1.5) * 100));
    voltBar.style.width = `${voltPct}%`;

    if (t.supply_voltage < 4.5) {
        voltHealth.innerText = 'Warning: Weak Power (<4.5V)';
        voltHealth.style.color = '#ef4444';
    } else {
        voltHealth.innerText = 'Health: Healthy (5V USB/Solar)';
        voltHealth.style.color = '#10b981';
    }

    // Battery Voltage (18650 Cell)
    if (t.battery_voltage !== undefined && document.getElementById('val-battery')) {
        document.getElementById('val-battery').innerText = `${t.battery_voltage.toFixed(2)} V`;
        const battBar = document.getElementById('bar-battery');
        const battHealth = document.getElementById('sub-battery');
        const battPct = Math.min(100, Math.max(0, ((t.battery_voltage - 3.0) / 1.2) * 100));
        battBar.style.width = `${battPct}%`;
        if (t.battery_voltage < 0.5) {
            battHealth.innerText = 'No Battery (USB Only)';
            battHealth.style.color = '#768390';
        } else if (t.battery_voltage < 3.3) {
            battHealth.innerText = 'Low Battery (<3.3V)';
            battHealth.style.color = '#ef4444';
        } else {
            battHealth.innerText = `Charge: ~${Math.round(battPct)}%`;
            battHealth.style.color = '#10b981';
        }
    }

    // Temp
    document.getElementById('val-temp').innerText = `${t.chip_temp.toFixed(1)} °C`;
    document.getElementById('bar-temp').style.width = `${Math.min(100, Math.max(0, (t.chip_temp / 70) * 100))}%`;

    // Pump state badge
    const badge = document.getElementById('pump-state-badge');
    if (t.pump_active) {
        badge.className = 'badge badge-active';
        badge.innerText = 'PUMP RUNNING';
    } else {
        badge.className = 'badge badge-idle';
        badge.innerText = 'PUMP IDLE';
    }
}

function updateFullStatusUI(data) {
    if (data.sensors) {
        updateTelemetryUI({
            ...data.sensors,
            pump_active: data.pump ? data.pump.active : false
        });
    }

    if (data.pump) {
        document.getElementById('pump-inhibit-reason').innerText = data.pump.status_message || 'Normal';
        document.getElementById('pump-cooldown').innerText = `${data.pump.cooldown_remaining_sec}s`;
        document.getElementById('pump-daily').innerText = `${data.pump.daily_used_sec}s / 60s max`;
    }

    // Host & System Info
    if (data.hostname) {
        document.getElementById('host-badge').innerText = `${data.hostname}.local`;
        document.getElementById('diag-mdns').innerText = `${data.hostname}.local`;
    }
    document.getElementById('diag-ip').innerText = data.ip || '--';
    document.getElementById('diag-rssi').innerText = `${data.wifi_rssi || '--'} dBm`;

    const uptime = data.uptime_sec || 0;
    const hrs = Math.floor(uptime / 3600);
    const mins = Math.floor((uptime % 3600) / 60);
    const secs = uptime % 60;
    document.getElementById('diag-uptime').innerText = `${hrs}h ${mins}m ${secs}s`;
}

function renderLogTable(logs) {
    const tbody = document.getElementById('log-body');
    if (!logs || logs.length === 0) {
        tbody.innerHTML = '<tr><td colspan="5" style="text-align:center;">No log data recorded yet.</td></tr>';
        return;
    }

    tbody.innerHTML = logs.map(r => `
        <tr>
            <td>+${r.timestamp}s</td>
            <td>${r.soil_percent}%</td>
            <td>${r.water_level_cm} cm</td>
            <td>${r.supply_voltage.toFixed(2)}V${r.battery_voltage > 0.5 ? ' / ' + r.battery_voltage.toFixed(2) + 'V Bat' : ''}</td>
            <td>${r.chip_temp.toFixed(1)}°C</td>
        </tr>
    `).join('');
}
