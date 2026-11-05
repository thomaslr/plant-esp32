// Plant Monitor ESP32-C3 Client-Side Controller
let ws = null;
let pollTimer = null;
let appConfig = null;
let lastTelemetry = null;

// Default config fallback if /api/config is delayed
const DEFAULT_CONFIG = {
    gauges: {
        soil_moisture: { trigger_threshold: 40, dry_adc: 2800, wet_adc: 1100, optimal_min: 35, optimal_max: 75 },
        water_level: { tank_height_cm: 30.0, sensor_offset_cm: 3.0, min_safe_level_cm: 2.0, refill_warning_pct: 25.0 },
        voltage: { min_warning: 4.5, max_warning: 5.5 },
        battery: { min_warning: 3.3, optimal_min: 3.7 },
        temperature: { optimal_max: 55.0, high_warning: 70.0 }
    },
    safety: {
        max_continuous_pump_sec: 60,
        min_time_between_cycles_sec: 300,
        daily_watering_limit: { enabled: true, max_seconds_per_day: 60 }
    }
};

document.addEventListener('DOMContentLoaded', () => {
    appConfig = DEFAULT_CONFIG;

    // Tabs setup
    document.getElementById('tab-btn-dash').addEventListener('click', () => switchTab('dashboard'));
    document.getElementById('tab-btn-cfg').addEventListener('click', () => switchTab('settings'));

    // Buttons
    document.getElementById('btn-water').addEventListener('click', handleWaterClick);
    document.getElementById('btn-sensor-reset').addEventListener('click', handleResetUltrasonic);
    document.getElementById('btn-wifi-reset').addEventListener('click', handleResetWifi);
    document.getElementById('btn-refresh-logs').addEventListener('click', fetchLogs);
    document.getElementById('btn-ota-check').addEventListener('click', handleOtaCheck);
    document.getElementById('file-ota-upload').addEventListener('change', handleOtaUpload);

    // Settings Form
    document.getElementById('config-form').addEventListener('submit', handleConfigSubmit);
    document.getElementById('btn-reload-cfg').addEventListener('click', fetchConfig);

    // Initial Data Fetches
    fetchConfig();
    initWebSocket();
    fetchStatus();
    fetchLogs();
    fetchOtaStatus();
});

// =============================================================================
// Tab Switching
// =============================================================================
function switchTab(tab) {
    const btnDash = document.getElementById('tab-btn-dash');
    const btnCfg = document.getElementById('tab-btn-cfg');
    const viewDash = document.getElementById('view-dashboard');
    const viewCfg = document.getElementById('view-settings');

    if (tab === 'dashboard') {
        btnDash.classList.add('active');
        btnCfg.classList.remove('active');
        viewDash.style.display = 'block';
        viewCfg.style.display = 'none';
        if (lastTelemetry) updateTelemetryUI(lastTelemetry);
    } else {
        btnCfg.classList.add('active');
        btnDash.classList.remove('active');
        viewDash.style.display = 'none';
        viewCfg.style.display = 'block';
        loadConfigToForm();
    }
}

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
// API Requests
// =============================================================================
async function fetchConfig() {
    try {
        const res = await fetch('/api/config');
        if (!res.ok) throw new Error("HTTP " + res.status);
        const cfg = await res.json();
        appConfig = cfg;
        renderAllDialZones();
        if (lastTelemetry) {
            updateTelemetryUI(lastTelemetry);
        }
        loadConfigToForm();
    } catch (err) {
        console.warn("Config fetch failed, using defaults:", err);
        renderAllDialZones();
    }
}

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
// Settings Form Logic & Persistence
// =============================================================================
function loadConfigToForm() {
    if (!appConfig) return;
    const g = appConfig.gauges || {};
    const sm = g.soil_moisture || {};
    const wl = g.water_level || {};
    const v = g.voltage || {};
    const b = g.battery || {};
    const t = g.temperature || {};
    const s = appConfig.safety || {};
    const d = s.daily_watering_limit || {};

    if (sm.trigger_threshold !== undefined) document.getElementById('cfg-soil-trigger').value = sm.trigger_threshold;
    if (sm.optimal_min !== undefined) document.getElementById('cfg-soil-opt-min').value = sm.optimal_min;
    if (sm.optimal_max !== undefined) document.getElementById('cfg-soil-opt-max').value = sm.optimal_max;

    if (wl.tank_height_cm !== undefined) document.getElementById('cfg-tank-height').value = wl.tank_height_cm;
    if (wl.min_safe_level_cm !== undefined) document.getElementById('cfg-tank-min-safe').value = wl.min_safe_level_cm;
    if (wl.refill_warning_pct !== undefined) document.getElementById('cfg-water-refill-warn').value = wl.refill_warning_pct;

    if (v.min_warning !== undefined) document.getElementById('cfg-volt-min').value = v.min_warning;
    if (v.max_warning !== undefined) document.getElementById('cfg-volt-max').value = v.max_warning;

    if (b.min_warning !== undefined) document.getElementById('cfg-batt-min').value = b.min_warning;
    if (b.optimal_min !== undefined) document.getElementById('cfg-batt-opt').value = b.optimal_min;

    if (t.optimal_max !== undefined) document.getElementById('cfg-temp-opt-max').value = t.optimal_max;
    if (t.high_warning !== undefined) document.getElementById('cfg-temp-high-warn').value = t.high_warning;

    if (s.max_continuous_pump_sec !== undefined) document.getElementById('cfg-safe-max-pump').value = s.max_continuous_pump_sec;
    if (s.min_time_between_cycles_sec !== undefined) document.getElementById('cfg-safe-cooldown').value = s.min_time_between_cycles_sec;
    if (d.max_seconds_per_day !== undefined) document.getElementById('cfg-safe-daily-limit').value = d.max_seconds_per_day;
}

async function handleConfigSubmit(e) {
    e.preventDefault();
    const btn = document.getElementById('btn-save-cfg');
    const msg = document.getElementById('cfg-status-msg');

    btn.disabled = true;
    btn.innerText = 'Saving to Flash...';
    msg.style.display = 'none';

    const patch = {
        gauges: {
            soil_moisture: {
                trigger_threshold: parseInt(document.getElementById('cfg-soil-trigger').value, 10),
                optimal_min: parseInt(document.getElementById('cfg-soil-opt-min').value, 10),
                optimal_max: parseInt(document.getElementById('cfg-soil-opt-max').value, 10)
            },
            water_level: {
                tank_height_cm: parseFloat(document.getElementById('cfg-tank-height').value),
                min_safe_level_cm: parseFloat(document.getElementById('cfg-tank-min-safe').value),
                refill_warning_pct: parseFloat(document.getElementById('cfg-water-refill-warn').value)
            },
            voltage: {
                min_warning: parseFloat(document.getElementById('cfg-volt-min').value),
                max_warning: parseFloat(document.getElementById('cfg-volt-max').value)
            },
            battery: {
                min_warning: parseFloat(document.getElementById('cfg-batt-min').value),
                optimal_min: parseFloat(document.getElementById('cfg-batt-opt').value)
            },
            temperature: {
                optimal_max: parseFloat(document.getElementById('cfg-temp-opt-max').value),
                high_warning: parseFloat(document.getElementById('cfg-temp-high-warn').value)
            }
        },
        safety: {
            max_continuous_pump_sec: parseInt(document.getElementById('cfg-safe-max-pump').value, 10),
            min_time_between_cycles_sec: parseInt(document.getElementById('cfg-safe-cooldown').value, 10),
            daily_watering_limit: {
                enabled: true,
                max_seconds_per_day: parseInt(document.getElementById('cfg-safe-daily-limit').value, 10)
            }
        }
    };

    try {
        const res = await fetch('/api/config', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(patch)
        });
        const resJson = await res.json();
        if (!res.ok) throw new Error(resJson.error || "Failed to commit configuration");

        // Merge patch into memory
        if (!appConfig.gauges) appConfig.gauges = {};
        Object.assign(appConfig.gauges.soil_moisture, patch.gauges.soil_moisture);
        Object.assign(appConfig.gauges.water_level, patch.gauges.water_level);
        Object.assign(appConfig.gauges.voltage, patch.gauges.voltage);
        Object.assign(appConfig.gauges.battery, patch.gauges.battery);
        Object.assign(appConfig.gauges.temperature, patch.gauges.temperature);
        Object.assign(appConfig.safety, patch.safety);

        // Immediately redraw dashboard dials with new zone thresholds
        renderAllDialZones();
        if (lastTelemetry) updateTelemetryUI(lastTelemetry);

        msg.className = 'cfg-status-msg status-ok';
        msg.innerText = '✅ Settings saved to NVS flash and applied immediately!';
        msg.style.display = 'inline-block';
        setTimeout(() => { msg.style.display = 'none'; }, 6000);
    } catch (err) {
        msg.className = 'cfg-status-msg status-err';
        msg.innerText = '❌ Save error: ' + err.message;
        msg.style.display = 'inline-block';
    } finally {
        btn.disabled = false;
        btn.innerText = '💾 Save Settings to Flash';
    }
}

// =============================================================================
// SVG Dial Drawing & Zone Geometry
// =============================================================================
/**
 * Traces an arc on a semicircle centered at (cx, cy) with radius r.
 * angle 0 is left (9 o'clock), angle 90 is top (12 o'clock), angle 180 is right (3 o'clock).
 */
function describeDialArc(cx, cy, r, angle1, angle2) {
    const rad1 = Math.PI - (angle1 * Math.PI / 180);
    const rad2 = Math.PI - (angle2 * Math.PI / 180);
    const x1 = (cx + r * Math.cos(rad1)).toFixed(2);
    const y1 = (cy - r * Math.sin(rad1)).toFixed(2);
    const x2 = (cx + r * Math.cos(rad2)).toFixed(2);
    const y2 = (cy - r * Math.sin(rad2)).toFixed(2);
    const largeArc = (angle2 - angle1) > 180 ? 1 : 0;
    return `M ${x1} ${y1} A ${r} ${r} 0 ${largeArc} 1 ${x2} ${y2}`;
}

/**
 * Builds SVG paths for colored zones given a min/max scale and array of { from, to, color }.
 */
function renderDialZones(containerId, min, max, zones) {
    const container = document.getElementById(containerId);
    if (!container) return;

    const cx = 100;
    const cy = 105;
    const r = 75;
    const range = max - min;

    let svgHtml = '';
    zones.forEach((z) => {
        const from = Math.max(min, Math.min(max, z.from));
        const to = Math.max(min, Math.min(max, z.to));
        if (to <= from) return;

        let a1 = ((from - min) / range) * 180;
        let a2 = ((to - min) / range) * 180;

        // Apply a tiny 0.5-degree notch gap between adjacent colored bands for clear instrument visuals
        if (a1 > 0) a1 += 0.4;
        if (a2 < 180) a2 -= 0.4;

        if (a2 > a1) {
            const d = describeDialArc(cx, cy, r, a1, a2);
            svgHtml += `<path class="dial-zone-arc" stroke="${z.color}" d="${d}" />`;
        }
    });

    container.innerHTML = svgHtml;
}

/**
 * Redraws colored zones on all 5 dials based on current appConfig.
 */
function renderAllDialZones() {
    const g = (appConfig && appConfig.gauges) ? appConfig.gauges : DEFAULT_CONFIG.gauges;

    // 1. Soil Moisture (0% - 100%)
    const sm = g.soil_moisture || DEFAULT_CONFIG.gauges.soil_moisture;
    const soilTrigger = sm.trigger_threshold;
    const soilOptMin = sm.optimal_min || 35;
    const soilOptMax = sm.optimal_max || 75;
    const soilZones = [
        { from: 0, to: soilTrigger, color: '#ef4444' }, // Red (Dry / Water Needed)
        { from: soilTrigger, to: Math.max(soilTrigger, soilOptMin), color: '#f59e0b' }, // Amber (Low Moisture)
        { from: Math.max(soilTrigger, soilOptMin), to: soilOptMax, color: '#10b981' }, // Green (Optimal)
        { from: soilOptMax, to: 100, color: '#3b82f6' } // Blue (Saturated)
    ];
    renderDialZones('zones-soil', 0, 100, soilZones);

    // 2. Reservoir Level (0% - 100%)
    const wl = g.water_level || DEFAULT_CONFIG.gauges.water_level;
    const refillWarn = wl.refill_warning_pct || 25.0;
    const waterZones = [
        { from: 0, to: 10, color: '#ef4444' }, // Red (Critical Empty)
        { from: 10, to: refillWarn, color: '#f59e0b' }, // Amber (Low / Refill Soon)
        { from: refillWarn, to: 85, color: '#10b981' }, // Green (Adequate)
        { from: 85, to: 100, color: '#3b82f6' } // Blue (Full Tank)
    ];
    renderDialZones('zones-water', 0, 100, waterZones);

    // 3. Supply Rail 5V (4.0V - 6.0V)
    const v = g.voltage || DEFAULT_CONFIG.gauges.voltage;
    const vMin = v.min_warning || 4.5;
    const vMax = v.max_warning || 5.5;
    const voltZones = [
        { from: 4.0, to: vMin, color: '#ef4444' }, // Red (Weak Power / Undervoltage)
        { from: vMin, to: vMax, color: '#10b981' }, // Green (Healthy 5V Rail)
        { from: vMax, to: 6.0, color: '#ef4444' }  // Red (Overvoltage Risk)
    ];
    renderDialZones('zones-voltage', 4.0, 6.0, voltZones);

    // 4. 18650 Battery (3.0V - 4.2V)
    const b = g.battery || DEFAULT_CONFIG.gauges.battery;
    const bMin = b.min_warning || 3.3;
    const bOpt = b.optimal_min || 3.7;
    const battZones = [
        { from: 3.0, to: bMin, color: '#ef4444' }, // Red (Discharged Cutoff)
        { from: bMin, to: bOpt, color: '#f59e0b' }, // Amber (Discharging)
        { from: bOpt, to: 4.2, color: '#10b981' }  // Green (Good Charge)
    ];
    renderDialZones('zones-battery', 3.0, 4.2, battZones);

    // 5. Chip Temperature (20°C - 85°C)
    const t = g.temperature || DEFAULT_CONFIG.gauges.temperature;
    const tOptMax = t.optimal_max || 55.0;
    const tHighWarn = t.high_warning || 70.0;
    const tempZones = [
        { from: 20, to: tOptMax, color: '#10b981' }, // Green (Normal Silicon Temp)
        { from: tOptMax, to: tHighWarn, color: '#f59e0b' }, // Amber (Warm Silicon)
        { from: tHighWarn, to: 85, color: '#ef4444' }  // Red (Overheating)
    ];
    renderDialZones('zones-temp', 20, 85, tempZones);
}

/**
 * Updates needle position, text, and badge for a specific dial card.
 */
function setDialState(id, val, min, max, displayText, badgeInfo, subText) {
    const fraction = Math.min(1.05, Math.max(-0.05, (val - min) / (max - min)));
    const angleDeg = -90 + (fraction * 180);

    const needle = document.getElementById(`needle-${id}`);
    if (needle) {
        needle.style.transform = `rotate(${angleDeg.toFixed(1)}deg)`;
        needle.setAttribute('transform', `rotate(${angleDeg.toFixed(1)}, 100, 105)`);
    }

    const valEl = document.getElementById(`val-${id}`);
    if (valEl) valEl.innerText = displayText;

    const badgeEl = document.getElementById(`badge-${id}`);
    if (badgeEl && badgeInfo) {
        badgeEl.className = `dial-badge ${badgeInfo.className}`;
        badgeEl.innerText = badgeInfo.text;
    }

    const subEl = document.getElementById(`sub-${id}`);
    if (subEl && subText !== undefined) subEl.innerText = subText;
}

// =============================================================================
// UI Telemetry Update
// =============================================================================
function updateTelemetryUI(t) {
    lastTelemetry = t;
    const g = (appConfig && appConfig.gauges) ? appConfig.gauges : DEFAULT_CONFIG.gauges;

    // 1. Soil Moisture
    const sm = g.soil_moisture || DEFAULT_CONFIG.gauges.soil_moisture;
    const soilVal = t.soil_percent !== undefined ? t.soil_percent : 0;
    let soilBadge = { className: 'badge-success', text: 'Optimal Moisture' };
    if (soilVal < sm.trigger_threshold) {
        soilBadge = { className: 'badge-danger', text: 'Dry - Water Needed' };
    } else if (sm.optimal_min && soilVal < sm.optimal_min) {
        soilBadge = { className: 'badge-warning', text: 'Low Moisture' };
    } else if (sm.optimal_max && soilVal > sm.optimal_max) {
        soilBadge = { className: 'badge-info', text: 'Wet / Saturated' };
    }
    setDialState('soil', soilVal, 0, 100, `${Math.round(soilVal)}%`, soilBadge, `Raw: ${t.soil_raw || 0} mV`);

    // 2. Reservoir Level
    const wl = g.water_level || DEFAULT_CONFIG.gauges.water_level;
    const waterVal = t.water_percent !== undefined ? t.water_percent : 0;
    const refillWarn = wl.refill_warning_pct || 25.0;
    let waterBadge = { className: 'badge-success', text: 'Adequate Level' };
    if (waterVal <= 10) {
        waterBadge = { className: 'badge-danger', text: 'Empty - Pump Inhibited' };
    } else if (waterVal <= refillWarn) {
        waterBadge = { className: 'badge-warning', text: 'Low - Refill Soon' };
    } else if (waterVal > 85) {
        waterBadge = { className: 'badge-info', text: 'Full Reservoir' };
    }
    setDialState('water', waterVal, 0, 100, `${Math.round(waterVal)}%`, waterBadge, `Depth: ${t.water_level_cm || 0} cm`);

    // 3. Supply Rail (5V)
    const vg = g.voltage || DEFAULT_CONFIG.gauges.voltage;
    const vMin = vg.min_warning || 4.5;
    const vMax = vg.max_warning || 5.5;
    const supplyVal = t.supply_voltage !== undefined ? t.supply_voltage : 0.0;
    let voltBadge = { className: 'badge-success', text: '5V Rail Healthy' };
    if (supplyVal < vMin) {
        voltBadge = { className: 'badge-danger', text: `Weak Power (<${vMin}V)` };
    } else if (supplyVal > vMax) {
        voltBadge = { className: 'badge-danger', text: `High Voltage (>${vMax}V)` };
    }
    setDialState('voltage', supplyVal, 4.0, 6.0, `${supplyVal.toFixed(2)} V`, voltBadge, supplyVal < vMin ? 'Under-voltage' : 'Healthy (5V USB/Solar)');

    // 4. 18650 Battery
    const bg = g.battery || DEFAULT_CONFIG.gauges.battery;
    const bMin = bg.min_warning || 3.3;
    const bOpt = bg.optimal_min || 3.7;
    const battVal = t.battery_voltage !== undefined ? t.battery_voltage : 0.0;
    let battBadge = { className: 'badge-neutral', text: 'Checking...' };
    let battSub = 'Status: OK';

    if (battVal < 0.5) {
        battBadge = { className: 'badge-neutral', text: 'No Battery (USB Only)' };
        battSub = 'External 5V Power';
        setDialState('battery', 3.0, 3.0, 4.2, `${battVal.toFixed(2)} V`, battBadge, battSub);
    } else {
        const battPct = Math.min(100, Math.max(0, Math.round(((battVal - 3.0) / 1.2) * 100)));
        if (battVal < bMin) {
            battBadge = { className: 'badge-danger', text: `Low Battery (<${bMin}V)` };
            battSub = `Discharged (~${battPct}%)`;
        } else if (battVal < bOpt) {
            battBadge = { className: 'badge-warning', text: `Discharging (~${battPct}%)` };
            battSub = 'Nominal Output';
        } else {
            battBadge = { className: 'badge-success', text: `Charged (~${battPct}%)` };
            battSub = 'Healthy Cell';
        }
        setDialState('battery', battVal, 3.0, 4.2, `${battVal.toFixed(2)} V`, battBadge, battSub);
    }

    // 5. Chip Temperature
    const tg = g.temperature || DEFAULT_CONFIG.gauges.temperature;
    const tOptMax = tg.optimal_max || 55.0;
    const tHighWarn = tg.high_warning || 70.0;
    const tempVal = t.chip_temp !== undefined ? t.chip_temp : 0.0;
    let tempBadge = { className: 'badge-success', text: 'Normal Temp' };
    if (tempVal > tHighWarn) {
        tempBadge = { className: 'badge-danger', text: `Overheating (>${tHighWarn}°C)` };
    } else if (tempVal > tOptMax) {
        tempBadge = { className: 'badge-warning', text: `Warm Silicon (>${tOptMax}°C)` };
    }
    setDialState('temp', tempVal, 20, 85, `${tempVal.toFixed(1)} °C`, tempBadge, 'ESP32-C3 Silicon');

    // Pump state badge
    const badge = document.getElementById('pump-state-badge');
    if (badge) {
        if (t.pump_active) {
            badge.className = 'badge badge-active';
            badge.innerText = 'PUMP RUNNING';
        } else {
            badge.className = 'badge badge-idle';
            badge.innerText = 'PUMP IDLE';
        }
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
