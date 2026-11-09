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
        temperature: { optimal_max: 55.0, high_warning: 70.0 },
        environmental: { sht30_enabled: false, bh1750_enabled: false, ldr_enabled: false }
    },
    timing: {
        sensor_read_interval_sec: 3600,
        automated_water_duration_sec: 30,
        pump_speed_pct: 60
    },
    safety: {
        max_continuous_pump_sec: 60,
        min_time_between_cycles_sec: 300,
        daily_watering_limit: { enabled: true, max_seconds_per_day: 93 }
    }
};

document.addEventListener('DOMContentLoaded', () => {
    appConfig = DEFAULT_CONFIG;

    // Register Service Worker for PWA Standalone App support
    if ('serviceWorker' in navigator) {
        window.addEventListener('load', () => {
            navigator.serviceWorker.register('/sw.js').then((reg) => {
                // Check for service worker updates on page load
                reg.update();
            }).catch((err) => {
                console.warn('[PWA] ServiceWorker registration failed:', err);
            });
        });
        // When new service worker activates and claims the page, reload to display fresh assets
        navigator.serviceWorker.addEventListener('controllerchange', () => {
            window.location.reload();
        });
    }

    // Tabs setup
    document.getElementById('tab-btn-dash').addEventListener('click', () => switchTab('dashboard'));
    document.getElementById('tab-btn-logs').addEventListener('click', () => switchTab('logs'));
    document.getElementById('tab-btn-diag').addEventListener('click', () => switchTab('diagnostics'));
    document.getElementById('tab-btn-cfg').addEventListener('click', () => switchTab('settings'));

    // Buttons
    document.getElementById('btn-water').addEventListener('click', handleWaterClick);
    document.getElementById('btn-sensor-reset').addEventListener('click', handleResetUltrasonic);
    document.getElementById('btn-wifi-reset').addEventListener('click', handleResetWifi);
    document.getElementById('btn-diag-reboot').addEventListener('click', handleReboot);
    document.getElementById('btn-settings-reboot').addEventListener('click', handleReboot);
    document.getElementById('btn-refresh-logs').addEventListener('click', fetchLogs);
    const logLimitSelect = document.getElementById('select-log-limit');
    if (logLimitSelect) logLimitSelect.addEventListener('change', fetchLogs);
    document.getElementById('btn-ota-check').addEventListener('click', handleOtaCheck);
    document.getElementById('file-ota-upload').addEventListener('change', handleOtaUpload);

    const availField = document.getElementById('ota-avail-field');
    if (availField) availField.addEventListener('click', handleOtaCheck);

    const headerOtaBadge = document.getElementById('header-ota-badge');
    if (headerOtaBadge) {
        headerOtaBadge.addEventListener('click', () => {
            const settingsTabBtn = document.querySelector('.nav-tab[data-tab="tab-settings"]');
            if (settingsTabBtn) settingsTabBtn.click();
            handleOtaCheck();
        });
    }

    // Modal Events
    document.getElementById('btn-modal-ota-close').addEventListener('click', closeOtaModal);
    document.getElementById('btn-modal-ota-cancel').addEventListener('click', closeOtaModal);
    document.getElementById('btn-modal-ota-confirm').addEventListener('click', executeOtaUpdate);

    const otaModal = document.getElementById('modal-ota-update');
    if (otaModal) {
        otaModal.addEventListener('click', (e) => {
            if (e.target === otaModal) closeOtaModal();
        });
    }
    window.addEventListener('keydown', (e) => {
        if (e.key === 'Escape') closeOtaModal();
    });

    // Settings & Threshold Forms
    const configForm = document.getElementById('config-form');
    if (configForm) configForm.addEventListener('submit', handleConfigSubmit);
    const reloadCfgBtn = document.getElementById('btn-reload-cfg');
    if (reloadCfgBtn) reloadCfgBtn.addEventListener('click', fetchConfig);

    const netForm = document.getElementById('network-form');
    if (netForm) netForm.addEventListener('submit', handleNetworkSubmit);
    const reloadNetBtn = document.getElementById('btn-reload-net');
    if (reloadNetBtn) reloadNetBtn.addEventListener('click', fetchConfig);
    const pumpSpeedSlider = document.getElementById('cfg-timing-pump-speed');
    if (pumpSpeedSlider) {
        pumpSpeedSlider.addEventListener('input', (e) => {
            const badge = document.getElementById('cfg-timing-pump-speed-val');
            if (badge) badge.textContent = e.target.value + '%';
        });
    }

    // Initial Data Fetches — sequence status first so top-right badge flips to 'Online' immediately
    // without competing with 3 other requests on ESP-IDF's single-threaded HTTP server.
    setupChartToggles();
    initDashboardSequence();
});

async function initDashboardSequence() {
    await fetchStatus();
    if (!pollTimer) {
        pollTimer = setInterval(fetchStatus, 60000);
    }
    await fetchConfig();
    fetchLogs();
    fetchOtaStatus();
}

// =============================================================================
// Tab Switching
// =============================================================================
function switchTab(tab) {
    const btnDash = document.getElementById('tab-btn-dash');
    const btnLogs = document.getElementById('tab-btn-logs');
    const btnDiag = document.getElementById('tab-btn-diag');
    const btnCfg = document.getElementById('tab-btn-cfg');

    const viewDash = document.getElementById('view-dashboard');
    const viewLogs = document.getElementById('view-logs');
    const viewDiag = document.getElementById('view-diagnostics');
    const viewCfg = document.getElementById('view-settings');

    btnDash.classList.toggle('active', tab === 'dashboard');
    btnLogs.classList.toggle('active', tab === 'logs');
    btnDiag.classList.toggle('active', tab === 'diagnostics');
    btnCfg.classList.toggle('active', tab === 'settings');

    viewDash.style.display = tab === 'dashboard' ? 'block' : 'none';
    viewLogs.style.display = tab === 'logs' ? 'block' : 'none';
    viewDiag.style.display = tab === 'diagnostics' ? 'block' : 'none';
    viewCfg.style.display = tab === 'settings' ? 'block' : 'none';

    if (tab === 'dashboard' && lastTelemetry) {
        updateTelemetryUI(lastTelemetry);
    } else if (tab === 'logs') {
        fetchLogs();
        setTimeout(() => renderTelemetryChart(currentLogs), 60);
    } else if (tab === 'diagnostics') {
        fetchOtaStatus();
        loadConfigToForm();
        if (lastTelemetry) updateTelemetryUI(lastTelemetry);
    } else if (tab === 'settings') {
        loadConfigToForm();
    }
}

// =============================================================================
// HTTP Polling
// =============================================================================
function startPolling() {
    fetchStatus();
    if (!pollTimer) {
        pollTimer = setInterval(fetchStatus, 60000);
    }
}

function setConnectionStatus(mode) {
    const dot = document.getElementById('conn-status');
    const text = document.getElementById('conn-text');
    if (mode === 'http') {
        dot.className = 'dot dot-connected';
        text.innerText = 'Online';
    } else {
        dot.className = 'dot dot-disconnected';
        text.innerText = 'Offline';
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
        setConnectionStatus('http');
        updateFullStatusUI(data);
    } catch (err) {
        setConnectionStatus('offline');
        console.warn("Status fetch failed:", err);
    }
}

async function fetchLogs() {
    const limitSelect = document.getElementById('select-log-limit');
    const limit = limitSelect ? limitSelect.value : '50';
    try {
        const res = await fetch(`/api/sensor-log?n=${limit}`);
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
    btn.innerText = 'Resetting...';

    try {
        const res = await fetch('/api/wifi/reset', { method: 'POST' });
        const result = await res.json();
        alert(result.message || "Wi-Fi credentials cleared! Connect to the PlantMonitor-Setup hotspot.");
    } catch (err) {
        alert("Wi-Fi reset failed: " + err.message);
        btn.disabled = false;
        btn.innerText = 'Reset Wi-Fi';
    }
}

async function handleReboot() {
    const targetHost = (appConfig && appConfig.system && appConfig.system.hostname) ? appConfig.system.hostname : 'plants';
    if (!confirm(`Reboot Plant Monitor? The device will restart and reconnect as ${targetHost}.local.`)) {
        return;
    }
    const diagBtn = document.getElementById('btn-diag-reboot');
    const cfgBtn = document.getElementById('btn-settings-reboot');
    if (diagBtn) { diagBtn.disabled = true; diagBtn.innerText = 'Rebooting...'; }
    if (cfgBtn) { cfgBtn.disabled = true; cfgBtn.innerText = 'Rebooting...'; }

    try {
        const res = await fetch('/api/system/reboot', { method: 'POST' });
        const result = await res.json();
        alert(result.message || "Device is rebooting. Page will reload in a few seconds.");
        setTimeout(() => {
            window.location.href = `http://${targetHost}.local`;
        }, 8000);
    } catch (err) {
        alert("Reboot request failed: " + err.message);
        if (diagBtn) { diagBtn.disabled = false; diagBtn.innerText = 'Reboot Device'; }
        if (cfgBtn) { cfgBtn.disabled = false; cfgBtn.innerText = '🔄 Reboot Device'; }
    }
}

// =============================================================================
// OTA & Firmware Version Management (Daily Checks & Pre-Install Review)
// =============================================================================
let currentInstalledVer = 'v0.2.35';
let pendingOtaRelease = null;
let dailyOtaCheckTimer = null;

function compareSemver(v1, v2) {
    const parse = v => (v || '').replace(/^v/, '').split('.').map(n => parseInt(n, 10) || 0);
    const p1 = parse(v1);
    const p2 = parse(v2);
    for (let i = 0; i < Math.max(p1.length, p2.length, 3); i++) {
        const num1 = p1[i] || 0;
        const num2 = p2[i] || 0;
        if (num1 > num2) return 1;
        if (num1 < num2) return -1;
    }
    return 0;
}

function formatCheckTime(ts) {
    if (!ts) return 'never';
    const d = new Date(ts);
    const now = new Date();
    const isToday = d.toDateString() === now.toDateString();
    const pad = n => String(n).padStart(2, '0');
    const timeStr = `${pad(d.getHours())}:${pad(d.getMinutes())}`;
    return isToday ? `today at ${timeStr}` : `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

async function fetchOtaStatus() {
    try {
        const res = await fetch('/api/ota/status');
        if (!res.ok) return;
        const data = await res.json();
        if (data.firmware_version) {
            const ver = `v${data.firmware_version.replace(/^v/, '')}`;
            currentInstalledVer = ver;
            const diagVer = document.getElementById('diag-ota-ver');
            if (diagVer) diagVer.innerText = ver;
            const badge = document.getElementById('version-badge');
            if (badge) badge.innerText = `${ver} 🌿`;
        }
        if (data.running_slot) {
            const nextSlotText = (data.next_update_slot === 'none') 
                ? 'No OTA slot (USB flash required)' 
                : `Next: ${data.next_update_slot}`;
            const diagSlot = document.getElementById('diag-ota-slot');
            if (diagSlot) diagSlot.innerText = `${data.running_slot} (${nextSlotText})`;
        }

        // If the ESP32 firmware ran its autonomous check, immediately reflect it in the UI!
        if (data.latest_firmware_version) {
            const releaseInfo = {
                tag_name: data.latest_firmware_version,
                lastChecked: (data.last_check_unix ? data.last_check_unix * 1000 : Date.now()),
                body: "Release notes available on GitHub."
            };
            cachedReleaseInfo = releaseInfo;
            updateOtaUi(currentInstalledVer, releaseInfo, false);
        }

        // Initialize daily background OTA checks and restore cached state
        initDailyOtaCheck();
    } catch (_) {}
}

function updateOtaUi(installedVer, releaseInfo, isChecking = false) {
    const btn = document.getElementById('btn-ota-check');
    const availField = document.getElementById('ota-avail-field');
    const diagStatus = document.getElementById('diag-ota-status');
    const headerBadge = document.getElementById('header-ota-badge');

    if (isChecking) {
        if (availField) {
            availField.style.display = 'inline-flex';
            availField.style.background = 'rgba(30, 41, 59, 0.6)';
            availField.style.color = '#94a3b8';
            availField.style.border = '1px solid #334155';
            availField.innerHTML = '<span style="margin-right: 6px;">🔄</span> Checking for updates...';
        }
        if (diagStatus) diagStatus.innerHTML = '<span style="color: #94a3b8;">Checking GitHub...</span>';
        return;
    }

    if (!releaseInfo || !releaseInfo.tag_name) {
        if (availField) availField.style.display = 'none';
        if (diagStatus) diagStatus.innerText = 'Unchecked';
        if (headerBadge) headerBadge.style.display = 'none';
        if (btn) btn.innerText = 'Check for Updates';
        return;
    }

    const latestTag = releaseInfo.tag_name;
    const isUpdateAvailable = compareSemver(latestTag, installedVer) > 0;
    const checkTimeStr = formatCheckTime(releaseInfo.lastChecked);

    if (isUpdateAvailable) {
        pendingOtaRelease = releaseInfo;
        if (availField) {
            availField.style.display = 'inline-flex';
            availField.style.background = 'rgba(16, 185, 129, 0.15)';
            availField.style.color = '#34d399';
            availField.style.border = '1px solid rgba(16, 185, 129, 0.4)';
            availField.innerHTML = `✨ Update Available: <strong style="margin-left: 4px; color: #a7f3d0;">${latestTag}</strong>`;
            availField.title = `Click to review what's new in ${latestTag}`;
        }
        if (btn) {
            btn.innerText = 'Review & Apply Update';
        }
        if (diagStatus) {
            diagStatus.innerHTML = `<span style="color: #34d399; font-weight: 600;">${latestTag} available</span> (${checkTimeStr})`;
        }
        if (headerBadge) {
            headerBadge.style.display = 'inline-flex';
            headerBadge.innerHTML = `✨ Update: ${latestTag}`;
            headerBadge.title = `New firmware ${latestTag} available! Click to review and update.`;
        }
    } else {
        pendingOtaRelease = null;
        if (availField) {
            availField.style.display = 'inline-flex';
            availField.style.background = 'rgba(30, 41, 59, 0.6)';
            availField.style.color = '#94a3b8';
            availField.style.border = '1px solid #334155';
            availField.innerHTML = `<span style="color: #10b981; margin-right: 6px;">✓</span> Up to date (${installedVer})`;
            availField.title = `Device is running the latest firmware version`;
        }
        if (btn) {
            btn.innerText = 'Check for Updates';
        }
        if (diagStatus) {
            diagStatus.innerHTML = `<span style="color: #10b981;">Up to date</span> (${checkTimeStr})`;
        }
        if (headerBadge) {
            headerBadge.style.display = 'none';
        }
    }
}

function initDailyOtaCheck() {
    // 1. Immediately render cached release state if available
    const cacheRaw = localStorage.getItem('plant_ota_cache');
    if (cacheRaw) {
        try {
            const cached = JSON.parse(cacheRaw);
            updateOtaUi(currentInstalledVer, cached, false);
        } catch (_) {}
    }

    // 2. Perform daily check if 24 hours have elapsed since last check
    const lastCheckMs = parseInt(localStorage.getItem('plant_ota_last_check_ms'), 10) || 0;
    const elapsed = Date.now() - lastCheckMs;
    const oneDayMs = 24 * 60 * 60 * 1000;

    if (!lastCheckMs || elapsed >= oneDayMs) {
        checkGitHubUpdate({ silent: true, force: true });
    }

    // 3. Hourly timer to ensure once-a-day check if browser session stays open
    if (!dailyOtaCheckTimer) {
        dailyOtaCheckTimer = setInterval(() => {
            const lastMs = parseInt(localStorage.getItem('plant_ota_last_check_ms'), 10) || 0;
            if (Date.now() - lastMs >= oneDayMs) {
                checkGitHubUpdate({ silent: true, force: true });
            }
        }, 3600 * 1000);
    }
}

async function checkGitHubUpdate(options = {}) {
    const { silent = false, force = false, openModal = false } = options;
    const btn = document.getElementById('btn-ota-check');
    const msg = document.getElementById('ota-msg');

    // Check if we can use cached info without re-fetching
    const lastCheckMs = parseInt(localStorage.getItem('plant_ota_last_check_ms'), 10) || 0;
    const cacheRaw = localStorage.getItem('plant_ota_cache');
    const oneDayMs = 24 * 60 * 60 * 1000;

    if (!force && (Date.now() - lastCheckMs) < oneDayMs && cacheRaw) {
        try {
            const cached = JSON.parse(cacheRaw);
            updateOtaUi(currentInstalledVer, cached, false);
            if (openModal && compareSemver(cached.tag_name, currentInstalledVer) > 0) {
                openOtaModal(currentInstalledVer, cached.tag_name, cached.body, cached.published_at);
            }
            return;
        } catch (_) {}
    }

    if (!silent) {
        if (btn) {
            btn.disabled = true;
            btn.innerText = 'Checking...';
        }
        if (msg) {
            msg.style.display = 'block';
            msg.style.color = '#58a6ff';
            msg.innerText = 'Checking GitHub Releases for new firmware...';
        }
        updateOtaUi(currentInstalledVer, null, true);
    }

    try {
        const ghRes = await fetch('https://api.github.com/repos/thomaslr/plant-esp32/releases/latest');
        if (!ghRes.ok) throw new Error(`Could not reach GitHub Releases API (HTTP ${ghRes.status})`);
        const ghData = await ghRes.json();
        const latestTag = ghData.tag_name || currentInstalledVer;

        const releaseInfo = {
            tag_name: latestTag,
            name: ghData.name || latestTag,
            body: ghData.body || 'Performance enhancements, bug fixes, and safety improvements.',
            published_at: ghData.published_at,
            lastChecked: Date.now()
        };

        localStorage.setItem('plant_ota_last_check_ms', String(Date.now()));
        localStorage.setItem('plant_ota_cache', JSON.stringify(releaseInfo));

        updateOtaUi(currentInstalledVer, releaseInfo, false);

        const isUpdateAvailable = compareSemver(latestTag, currentInstalledVer) > 0;
        if (isUpdateAvailable) {
            if (msg) {
                msg.style.color = '#34d399';
                msg.innerText = `New firmware ${latestTag} is available. Click "Review & Apply Update" to review changelog and install.`;
            }
            if (!silent || openModal) {
                if (msg) msg.style.display = 'none';
                openOtaModal(currentInstalledVer, latestTag, releaseInfo.body, releaseInfo.published_at);
            }
        } else {
            if (!silent && msg) {
                msg.style.color = '#10b981';
                msg.innerText = `You are on the latest firmware (${currentInstalledVer})!`;
            }
        }
    } catch (err) {
        console.warn('[OTA] Check failed:', err);
        const cacheRaw = localStorage.getItem('plant_ota_cache');
        if (cacheRaw) {
            try { updateOtaUi(currentInstalledVer, JSON.parse(cacheRaw), false); } catch (_) {}
        } else {
            updateOtaUi(currentInstalledVer, null, false);
        }
        if (!silent && msg) {
            msg.style.color = '#ef4444';
            msg.innerText = 'Update check failed: ' + err.message;
        }
    } finally {
        if (btn) btn.disabled = false;
    }
}

function closeOtaModal() {
    const modal = document.getElementById('modal-ota-update');
    if (modal) modal.style.display = 'none';
}

function renderMarkdownToHtml(md) {
    if (!md) return '<p class="changelog-p">Bug fixes and performance improvements.</p>';
    let escaped = md
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;');

    const lines = escaped.split(/\r?\n/);
    let html = '';
    let inList = false;

    for (let i = 0; i < lines.length; i++) {
        let line = lines[i].trim();
        if (line.startsWith('- ') || line.startsWith('* ') || line.startsWith('• ')) {
            if (!inList) {
                html += '<ul class="changelog-list">';
                inList = true;
            }
            let itemContent = line.replace(/^[-*•]\s+/, '');
            itemContent = itemContent.replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>');
            itemContent = itemContent.replace(/(^|[^*])\*([^*]+?)\*(?!\*)/g, '$1<em>$2</em>');
            itemContent = itemContent.replace(/\[([^\]]+)\]\((https?:\/\/[^\s\)]+)\)/g, '<a href="$2" target="_blank" rel="noopener noreferrer">$1 ↗</a>');
            itemContent = itemContent.replace(/(^|\s)(https?:\/\/[^\s<]+)/g, '$1<a href="$2" target="_blank" rel="noopener noreferrer">$2 ↗</a>');
            html += `<li>${itemContent}</li>`;
        } else {
            if (inList) {
                html += '</ul>';
                inList = false;
            }
            if (line.startsWith('### ')) {
                html += `<h4 class="changelog-heading">${line.substring(4)}</h4>`;
            } else if (line.startsWith('## ')) {
                html += `<h3 class="changelog-heading">${line.substring(3)}</h3>`;
            } else if (line.length > 0) {
                let formatted = line
                    .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
                    .replace(/(^|[^*])\*([^*]+?)\*(?!\*)/g, '$1<em>$2</em>')
                    .replace(/\[([^\]]+)\]\((https?:\/\/[^\s\)]+)\)/g, '<a href="$2" target="_blank" rel="noopener noreferrer">$1 ↗</a>')
                    .replace(/(^|\s)(https?:\/\/[^\s<]+)/g, '$1<a href="$2" target="_blank" rel="noopener noreferrer">$2 ↗</a>');
                html += `<p class="changelog-p">${formatted}</p>`;
            }
        }
    }
    if (inList) html += '</ul>';
    return html || '<p class="changelog-p">Bug fixes and performance improvements.</p>';
}

function openOtaModal(currentVer, newVer, changelog, publishedAt) {
    const curElem = document.getElementById('modal-ver-current');
    const targetElem = document.getElementById('modal-ver-target');
    const dateElem = document.getElementById('modal-ota-date');
    const changelogElem = document.getElementById('modal-ota-changelog');
    const modal = document.getElementById('modal-ota-update');

    if (curElem) curElem.innerText = `Current: ${currentVer}`;
    if (targetElem) targetElem.innerText = `New: ${newVer}`;
    if (dateElem) {
        if (publishedAt) {
            const d = new Date(publishedAt);
            dateElem.innerText = `Released: ${d.toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' })}`;
            dateElem.style.display = 'block';
        } else {
            dateElem.style.display = 'none';
        }
    }
    if (changelogElem) changelogElem.innerHTML = renderMarkdownToHtml(changelog);
    if (modal) modal.style.display = 'flex';
}

async function handleOtaCheck() {
    // If an update is already identified and cached, report updates in modal before asking user to apply
    if (pendingOtaRelease && compareSemver(pendingOtaRelease.tag_name, currentInstalledVer) > 0) {
        openOtaModal(currentInstalledVer, pendingOtaRelease.tag_name, pendingOtaRelease.body, pendingOtaRelease.published_at);
        return;
    }
    // Otherwise force check GitHub now and open modal if an update is found
    await checkGitHubUpdate({ silent: false, force: true, openModal: true });
}

async function executeOtaUpdate() {
    closeOtaModal();
    const btn = document.getElementById('btn-ota-check');
    const msg = document.getElementById('ota-msg');
    if (btn) {
        btn.disabled = true;
        btn.innerText = 'Updating...';
    }
    if (msg) {
        msg.style.display = 'block';
        msg.style.color = '#58a6ff';
        msg.innerText = 'Streaming new firmware binary from GitHub... Please do not unplug device.';
    }

    try {
        const res = await fetch('/api/ota/update', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: '{}' });
        const result = await res.json();
        if (msg) msg.innerText = result.message || 'OTA update started! Device will reboot into new firmware in ~30s.';
        setTimeout(() => {
            if (msg) msg.innerText = 'Device rebooting. Refreshing dashboard in 15s...';
            setTimeout(() => window.location.reload(), 15000);
        }, 15000);
    } catch (err) {
        if (msg) {
            msg.style.color = '#ef4444';
            msg.innerText = 'OTA update error: ' + err.message;
        }
        if (btn) {
            btn.disabled = false;
            btn.innerText = pendingOtaRelease ? 'Review & Apply Update' : 'Check for Updates';
        }
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
    const sys = appConfig.system || {};
    const g = appConfig.gauges || {};
    const sm = g.soil_moisture || {};
    const wl = g.water_level || {};
    const v = g.voltage || {};
    const b = g.battery || {};
    const t = g.temperature || {};
    const tm = appConfig.timing || {};
    const s = appConfig.safety || {};
    const d = s.daily_watering_limit || {};
    const w = appConfig.wifi || {};

    if (sys.hostname !== undefined) {
        document.getElementById('cfg-system-hostname').value = sys.hostname;
    }

    const wifiSleepCheck = document.getElementById('cfg-wifi-modem-sleep');
    if (wifiSleepCheck) wifiSleepCheck.checked = w.modem_sleep !== undefined ? w.modem_sleep : true;
    if (w.tx_power !== undefined) {
        const txSelect = document.getElementById('cfg-wifi-tx-power');
        if (txSelect) txSelect.value = w.tx_power;
    }
    const wifi20Check = document.getElementById('cfg-wifi-force-20mhz');
    if (wifi20Check) wifi20Check.checked = w.force_20mhz !== undefined ? w.force_20mhz : true;

    if (sm.trigger_threshold !== undefined) document.getElementById('cfg-soil-trigger').value = sm.trigger_threshold;
    if (sm.optimal_min !== undefined) document.getElementById('cfg-soil-opt-min').value = sm.optimal_min;
    if (sm.optimal_max !== undefined) document.getElementById('cfg-soil-opt-max').value = sm.optimal_max;
    if (sm.dry_adc !== undefined) document.getElementById('cfg-soil-dry-adc').value = sm.dry_adc;
    if (sm.wet_adc !== undefined) document.getElementById('cfg-soil-wet-adc').value = sm.wet_adc;

    if (wl.tank_height_cm !== undefined) document.getElementById('cfg-tank-height').value = wl.tank_height_cm;
    if (wl.sensor_offset_cm !== undefined) document.getElementById('cfg-sensor-offset').value = wl.sensor_offset_cm;
    if (wl.min_safe_level_cm !== undefined) document.getElementById('cfg-tank-min-safe').value = wl.min_safe_level_cm;
    if (wl.refill_warning_pct !== undefined) document.getElementById('cfg-water-refill-warn').value = wl.refill_warning_pct;

    if (v.min_warning !== undefined) document.getElementById('cfg-volt-min').value = v.min_warning;
    if (v.max_warning !== undefined) document.getElementById('cfg-volt-max').value = v.max_warning;

    if (b.min_warning !== undefined) document.getElementById('cfg-batt-min').value = b.min_warning;
    if (b.optimal_min !== undefined) document.getElementById('cfg-batt-opt').value = b.optimal_min;

    if (t.optimal_max !== undefined) document.getElementById('cfg-temp-opt-max').value = t.optimal_max;
    if (t.high_warning !== undefined) document.getElementById('cfg-temp-high-warn').value = t.high_warning;

    const env = g.environmental || {};
    const sht30Check = document.getElementById('cfg-env-sht30');
    if (sht30Check) sht30Check.checked = !!env.sht30_enabled;
    const bh1750Check = document.getElementById('cfg-env-bh1750');
    if (bh1750Check) bh1750Check.checked = !!env.bh1750_enabled;
    const ldrCheck = document.getElementById('cfg-env-ldr');
    if (ldrCheck) ldrCheck.checked = !!env.ldr_enabled;

    if (s.max_continuous_pump_sec !== undefined) document.getElementById('cfg-safe-max-pump').value = s.max_continuous_pump_sec;
    if (s.min_time_between_cycles_sec !== undefined) document.getElementById('cfg-safe-cooldown').value = s.min_time_between_cycles_sec;
    if (d.max_seconds_per_day !== undefined) document.getElementById('cfg-safe-daily-limit').value = d.max_seconds_per_day;

    const n = s.night_watering_protection || {};
    const nightCheck = document.getElementById('cfg-safe-night-enabled');
    if (nightCheck) nightCheck.checked = n.enabled !== undefined ? n.enabled : true;
    if (n.start_hour !== undefined) document.getElementById('cfg-safe-night-start').value = n.start_hour;
    if (n.end_hour !== undefined) document.getElementById('cfg-safe-night-end').value = n.end_hour;

    if (tm.sensor_read_interval_sec !== undefined) document.getElementById('cfg-timing-read-interval').value = tm.sensor_read_interval_sec;
    if (tm.automated_water_duration_sec !== undefined) document.getElementById('cfg-timing-auto-water-duration').value = tm.automated_water_duration_sec;
    const pSpeed = tm.pump_speed_pct !== undefined ? tm.pump_speed_pct : 60;
    const speedSlider = document.getElementById('cfg-timing-pump-speed');
    const speedVal = document.getElementById('cfg-timing-pump-speed-val');
    if (speedSlider) speedSlider.value = pSpeed;
    if (speedVal) speedVal.textContent = pSpeed + '%';
}

async function handleConfigSubmit(e) {
    e.preventDefault();
    const btn = document.getElementById('btn-save-cfg');
    const msg = document.getElementById('cfg-status-msg');

    btn.disabled = true;
    btn.innerText = 'Saving to Flash...';
    msg.style.display = 'none';

    const rawHostname = document.getElementById('cfg-system-hostname').value.trim();

    const patch = {
        system: {
            hostname: rawHostname
        },
        gauges: {
            soil_moisture: {
                trigger_threshold: parseInt(document.getElementById('cfg-soil-trigger').value, 10),
                optimal_min: parseInt(document.getElementById('cfg-soil-opt-min').value, 10),
                optimal_max: parseInt(document.getElementById('cfg-soil-opt-max').value, 10),
                dry_adc: parseInt(document.getElementById('cfg-soil-dry-adc').value, 10),
                wet_adc: parseInt(document.getElementById('cfg-soil-wet-adc').value, 10)
            },
            water_level: {
                tank_height_cm: parseFloat(document.getElementById('cfg-tank-height').value),
                sensor_offset_cm: parseFloat(document.getElementById('cfg-sensor-offset').value),
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
            },
            environmental: {
                sht30_enabled: document.getElementById('cfg-env-sht30') ? document.getElementById('cfg-env-sht30').checked : false,
                bh1750_enabled: document.getElementById('cfg-env-bh1750') ? document.getElementById('cfg-env-bh1750').checked : false,
                ldr_enabled: document.getElementById('cfg-env-ldr') ? document.getElementById('cfg-env-ldr').checked : false
            }
        },
        safety: {
            max_continuous_pump_sec: parseInt(document.getElementById('cfg-safe-max-pump').value, 10),
            min_time_between_cycles_sec: parseInt(document.getElementById('cfg-safe-cooldown').value, 10),
            daily_watering_limit: {
                enabled: true,
                max_seconds_per_day: parseInt(document.getElementById('cfg-safe-daily-limit').value, 10)
            },
            night_watering_protection: {
                enabled: document.getElementById('cfg-safe-night-enabled') ? document.getElementById('cfg-safe-night-enabled').checked : true,
                start_hour: parseInt(document.getElementById('cfg-safe-night-start').value, 10),
                end_hour: parseInt(document.getElementById('cfg-safe-night-end').value, 10)
            }
        },
        timing: {
            sensor_read_interval_sec: parseInt(document.getElementById('cfg-timing-read-interval').value, 10),
            automated_water_duration_sec: parseInt(document.getElementById('cfg-timing-auto-water-duration').value, 10),
            pump_speed_pct: parseInt(document.getElementById('cfg-timing-pump-speed').value, 10)
        },
        wifi: {
            modem_sleep: document.getElementById('cfg-wifi-modem-sleep') ? document.getElementById('cfg-wifi-modem-sleep').checked : true,
            tx_power: parseInt(document.getElementById('cfg-wifi-tx-power').value, 10),
            force_20mhz: document.getElementById('cfg-wifi-force-20mhz') ? document.getElementById('cfg-wifi-force-20mhz').checked : true
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
        if (!appConfig.system) appConfig.system = {};
        const oldHostname = appConfig.system.hostname;
        Object.assign(appConfig.system, patch.system);
        if (!appConfig.gauges) appConfig.gauges = {};
        Object.assign(appConfig.gauges.soil_moisture, patch.gauges.soil_moisture);
        Object.assign(appConfig.gauges.water_level, patch.gauges.water_level);
        Object.assign(appConfig.gauges.voltage, patch.gauges.voltage);
        Object.assign(appConfig.gauges.battery, patch.gauges.battery);
        Object.assign(appConfig.gauges.temperature, patch.gauges.temperature);
        Object.assign(appConfig.safety, patch.safety);
        if (!appConfig.timing) appConfig.timing = {};
        Object.assign(appConfig.timing, patch.timing);
        if (!appConfig.wifi) appConfig.wifi = {};
        Object.assign(appConfig.wifi, patch.wifi);

        // Immediately redraw dashboard dials with new zone thresholds
        renderAllDialZones();
        if (lastTelemetry) updateTelemetryUI(lastTelemetry);
        loadConfigToForm();

        const hostnameChanged = oldHostname && patch.system && patch.system.hostname && (oldHostname !== patch.system.hostname);
        msg.className = 'cfg-status-msg status-ok';
        if (hostnameChanged) {
            msg.innerText = `✅ Settings saved! Hostname changed to "${patch.system.hostname}". Click "Reboot Device" to activate ${patch.system.hostname}.local.`;
        } else {
            msg.innerText = '✅ Settings saved to NVS flash and applied immediately!';
        }
        msg.style.display = 'inline-block';
        setTimeout(() => { msg.style.display = 'none'; }, 8000);
    } catch (err) {
        msg.className = 'cfg-status-msg status-err';
        msg.innerText = '❌ Save error: ' + err.message;
        msg.style.display = 'inline-block';
    } finally {
        btn.disabled = false;
        btn.innerText = '💾 Save Settings to Flash';
    }
}

async function handleNetworkSubmit(e) {
    e.preventDefault();
    const btn = document.getElementById('btn-save-net');
    const msg = document.getElementById('net-status-msg');

    btn.disabled = true;
    btn.innerText = 'Saving...';
    msg.style.display = 'none';

    const rawHostname = document.getElementById('cfg-system-hostname').value.trim();

    const patch = {
        system: {
            hostname: rawHostname
        },
        gauges: {
            environmental: {
                sht30_enabled: document.getElementById('cfg-env-sht30') ? document.getElementById('cfg-env-sht30').checked : false,
                bh1750_enabled: document.getElementById('cfg-env-bh1750') ? document.getElementById('cfg-env-bh1750').checked : false,
                ldr_enabled: document.getElementById('cfg-env-ldr') ? document.getElementById('cfg-env-ldr').checked : false
            }
        },
        wifi: {
            modem_sleep: document.getElementById('cfg-wifi-modem-sleep') ? document.getElementById('cfg-wifi-modem-sleep').checked : true,
            tx_power: parseInt(document.getElementById('cfg-wifi-tx-power').value, 10),
            force_20mhz: document.getElementById('cfg-wifi-force-20mhz') ? document.getElementById('cfg-wifi-force-20mhz').checked : true
        }
    };

    try {
        const res = await fetch('/api/config', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(patch)
        });
        const resJson = await res.json();
        if (!res.ok) throw new Error(resJson.error || "Failed to commit settings");

        if (!appConfig.system) appConfig.system = {};
        const oldHostname = appConfig.system.hostname;
        Object.assign(appConfig.system, patch.system);
        if (!appConfig.gauges) appConfig.gauges = {};
        if (!appConfig.gauges.environmental) appConfig.gauges.environmental = {};
        Object.assign(appConfig.gauges.environmental, patch.gauges.environmental);
        if (!appConfig.wifi) appConfig.wifi = {};
        Object.assign(appConfig.wifi, patch.wifi);

        loadConfigToForm();
        renderAllDialZones();
        if (lastTelemetry) updateTelemetryUI(lastTelemetry);

        const hostnameChanged = oldHostname && patch.system && patch.system.hostname && (oldHostname !== patch.system.hostname);
        msg.className = 'cfg-status-msg status-ok';
        if (hostnameChanged) {
            msg.innerText = `✅ Settings & Hostname saved! Click "Reboot Device" to activate ${patch.system.hostname}.local.`;
        } else {
            msg.innerText = '✅ Settings saved and applied immediately!';
        }
        msg.style.display = 'inline-block';
        setTimeout(() => { msg.style.display = 'none'; }, 8000);
    } catch (err) {
        msg.className = 'cfg-status-msg status-err';
        msg.innerText = '❌ Save error: ' + err.message;
        msg.style.display = 'inline-block';
    } finally {
        btn.disabled = false;
        btn.innerText = '💾 Save Settings';
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
            svgHtml += `<path class="dial-zone-arc" fill="none" stroke="${z.color}" stroke-width="8" stroke-linecap="butt" d="${d}" />`;
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

    // 6. Air Temperature (0°C - 50°C)
    renderDialZones('zones-air-temp', 0, 50, [
        { from: 0, to: 15, color: '#38bdf8' },  // Blue (Cool)
        { from: 15, to: 30, color: '#10b981' }, // Green (Optimal Plant Air)
        { from: 30, to: 50, color: '#f59e0b' }  // Amber (Hot)
    ]);

    // 7. Relative Humidity (0% - 100%)
    renderDialZones('zones-humidity', 0, 100, [
        { from: 0, to: 35, color: '#f59e0b' },  // Amber (Dry Air)
        { from: 35, to: 75, color: '#10b981' }, // Green (Optimal Humidity)
        { from: 75, to: 100, color: '#38bdf8' } // Blue (High Humidity)
    ]);

    // 8. Ambient Light Lux (0 - 5000 lx)
    renderDialZones('zones-light-lux', 0, 5000, [
        { from: 0, to: 200, color: '#64748b' },    // Dim/Dark
        { from: 200, to: 2500, color: '#10b981' }, // Good Indoor/Filtered Light
        { from: 2500, to: 5000, color: '#eab308' } // Bright Direct Sun
    ]);

    // 9. LDR Light % (0% - 100%)
    renderDialZones('zones-ldr', 0, 100, [
        { from: 0, to: 20, color: '#64748b' },   // Dark
        { from: 20, to: 70, color: '#10b981' },  // Moderate Light
        { from: 70, to: 100, color: '#f97316' }  // Bright Light
    ]);
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

    // 6. Air Temperature (SHT30)
    const cardAir = document.getElementById('card-air-temp');
    const pillAir = document.getElementById('pill-air-temp');
    if (t.air_temp_c !== undefined && t.air_temp_c !== null) {
        if (cardAir) cardAir.style.display = 'block';
        if (pillAir) pillAir.style.display = 'inline-flex';
        let airBadge = { className: 'badge-success', text: 'Comfortable' };
        if (t.air_temp_c < 15) airBadge = { className: 'badge-info', text: 'Cool Air' };
        else if (t.air_temp_c > 32) airBadge = { className: 'badge-danger', text: 'Hot Air' };
        setDialState('air-temp', t.air_temp_c, 0, 50, `${t.air_temp_c.toFixed(1)} °C`, airBadge, 'SHT30 I2C');
    } else {
        if (cardAir) cardAir.style.display = 'none';
        if (pillAir) pillAir.style.display = 'none';
    }

    // 7. Relative Humidity (SHT30)
    const cardHumi = document.getElementById('card-humidity');
    const pillHumi = document.getElementById('pill-humidity');
    if (t.humidity_pct !== undefined && t.humidity_pct !== null) {
        if (cardHumi) cardHumi.style.display = 'block';
        if (pillHumi) pillHumi.style.display = 'inline-flex';
        let humiBadge = { className: 'badge-success', text: 'Optimal RH' };
        if (t.humidity_pct < 35) humiBadge = { className: 'badge-warning', text: 'Dry Air' };
        else if (t.humidity_pct > 80) humiBadge = { className: 'badge-info', text: 'Humid' };
        setDialState('humidity', t.humidity_pct, 0, 100, `${Math.round(t.humidity_pct)}%`, humiBadge, 'SHT30 I2C');
    } else {
        if (cardHumi) cardHumi.style.display = 'none';
        if (pillHumi) pillHumi.style.display = 'none';
    }

    // 8. Ambient Light Lux (BH1750)
    const cardLux = document.getElementById('card-light-lux');
    const pillLux = document.getElementById('pill-light-lux');
    if (t.light_lux !== undefined && t.light_lux !== null) {
        if (cardLux) cardLux.style.display = 'block';
        if (pillLux) pillLux.style.display = 'inline-flex';
        let luxBadge = { className: 'badge-success', text: 'Good Light' };
        if (t.light_lux < 100) luxBadge = { className: 'badge-neutral', text: 'Dim / Shade' };
        else if (t.light_lux > 3000) luxBadge = { className: 'badge-warning', text: 'Bright Sun' };
        setDialState('light-lux', t.light_lux, 0, 5000, `${Math.round(t.light_lux)} lx`, luxBadge, 'BH1750 I2C');
    } else {
        if (cardLux) cardLux.style.display = 'none';
        if (pillLux) pillLux.style.display = 'none';
    }

    // 9. LDR Light % (Analog)
    const cardLdr = document.getElementById('card-ldr');
    const pillLdr = document.getElementById('pill-ldr');
    if (t.ldr_percent !== undefined && t.ldr_percent !== null) {
        if (cardLdr) cardLdr.style.display = 'block';
        if (pillLdr) pillLdr.style.display = 'inline-flex';
        let ldrBadge = { className: 'badge-success', text: 'Moderate' };
        if (t.ldr_percent < 20) ldrBadge = { className: 'badge-neutral', text: 'Dark' };
        else if (t.ldr_percent > 75) ldrBadge = { className: 'badge-warning', text: 'Bright' };
        setDialState('ldr', t.ldr_percent, 0, 100, `${Math.round(t.ldr_percent)}%`, ldrBadge, 'GPIO 1 ADC');
    } else {
        if (cardLdr) cardLdr.style.display = 'none';
        if (pillLdr) pillLdr.style.display = 'none';
    }

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
        const lastWateredElem = document.getElementById('pump-last-watered');
        if (lastWateredElem) {
            lastWateredElem.innerText = data.pump.last_watered_text || 'None since boot';
        }
        document.getElementById('pump-cooldown').innerText = `${data.pump.cooldown_remaining_sec}s`;
        const maxDaily = (appConfig && appConfig.safety && appConfig.safety.daily_watering_limit)
            ? appConfig.safety.daily_watering_limit.max_seconds_per_day
            : 93;
        document.getElementById('pump-daily').innerText = `${data.pump.daily_used_sec}s / ${maxDaily}s max`;
        const quietElem = document.getElementById('pump-quiet-hours');
        if (quietElem) {
            quietElem.innerText = data.pump.quiet_hours_text || 'Disabled';
            if (data.pump.quiet_hours_active) {
                quietElem.style.color = '#f59e0b';
                quietElem.style.fontWeight = '600';
            } else {
                quietElem.style.color = 'inherit';
                quietElem.style.fontWeight = 'normal';
            }
        }
    }

    // Host & System Info
    if (data.hostname) {
        document.getElementById('host-badge').innerText = `${data.hostname}.local`;
        document.getElementById('diag-mdns').innerText = `${data.hostname}.local`;
    }
    document.getElementById('diag-ip').innerText = data.ip || '--';
    document.getElementById('diag-rssi').innerText = `${data.wifi_rssi || '--'} dBm`;

    const uptime = data.uptime_sec || 0;
    latestDeviceUptimeSec = uptime;
    statusFetchWallTime = Date.now();
    const hrs = Math.floor(uptime / 3600);
    const mins = Math.floor((uptime % 3600) / 60);
    const secs = uptime % 60;
    document.getElementById('diag-uptime').innerText = `${hrs}h ${mins}m ${secs}s`;

    // Synchronize client-side clock with ESP32 reported time
    syncDeviceClock(data);
}

let espClockOffsetMs = null;
let espNtpSynced = false;
let clockIntervalTimer = null;

function syncDeviceClock(data) {
    if (data.time_unix !== undefined && data.time_unix !== null && data.time_unix > 0) {
        const fetchTime = Date.now();
        const espTimeMs = data.time_unix * 1000;
        espNtpSynced = !!data.ntp_synced;
        espClockOffsetMs = espTimeMs - fetchTime;

        renderHeaderClock();
        if (!clockIntervalTimer) {
            clockIntervalTimer = setInterval(renderHeaderClock, 1000);
        }
    }
}

function renderHeaderClock() {
    const clockBadge = document.getElementById('clock-badge');
    const diagTime = document.getElementById('diag-time');
    const diagNtp = document.getElementById('diag-ntp');

    if (espClockOffsetMs === null) {
        if (clockBadge) clockBadge.innerText = '🕒 Connecting...';
        return;
    }

    if (!espNtpSynced) {
        if (clockBadge) {
            clockBadge.style.color = '#f59e0b';
            clockBadge.innerHTML = '🕒 Syncing NTP...';
            clockBadge.title = 'ESP32 is waiting for NTP time synchronization';
        }
        if (diagNtp) diagNtp.innerHTML = '<span style="color: #f59e0b;">Syncing with NTP...</span>';
        return;
    }

    const currentEspMs = Date.now() + espClockOffsetMs;
    const d = new Date(currentEspMs);

    const pad = n => String(n).padStart(2, '0');
    const year = d.getFullYear();
    const month = pad(d.getMonth() + 1);
    const day = pad(d.getDate());
    const hours = pad(d.getHours());
    const minutes = pad(d.getMinutes());
    const seconds = pad(d.getSeconds());

    const dateStr = `${year}-${month}-${day}`;
    const timeStr = `${hours}:${minutes}:${seconds}`;

    if (clockBadge) {
        clockBadge.style.color = '#cbd5e1';
        clockBadge.innerHTML = `<span style="color: #38bdf8;">🕒</span> ${dateStr} ${timeStr}`;
        clockBadge.title = `ESP32 Time: ${dateStr} ${timeStr} (NTP Synchronized)`;
    }

    if (diagTime) diagTime.innerText = `${dateStr} ${timeStr}`;
    if (diagNtp) diagNtp.innerHTML = '<span style="color: #10b981;">Synchronized (hourly)</span>';
}

let latestDeviceUptimeSec = null;
let statusFetchWallTime = null;

function getReadingDate(timestampSec) {
    if (!timestampSec && timestampSec !== 0) return null;
    if (timestampSec > 1000000000) {
        return new Date(timestampSec * 1000);
    }
    if (latestDeviceUptimeSec !== null && statusFetchWallTime !== null) {
        const currentEstimatedBoot = latestDeviceUptimeSec + (Date.now() - statusFetchWallTime) / 1000;
        const secondsAgo = Math.max(0, currentEstimatedBoot - timestampSec);
        return new Date(Date.now() - secondsAgo * 1000);
    }
    return null;
}

function formatReadingTime(timestampSec, includeSeconds = true) {
    const d = getReadingDate(timestampSec);
    if (!d) return `+${timestampSec}s`;
    const opts = {
        hour: '2-digit',
        minute: '2-digit',
        hour12: false
    };
    if (includeSeconds) opts.second = '2-digit';
    return d.toLocaleTimeString([], opts);
}

function formatReadingRelative(timestampSec) {
    const d = getReadingDate(timestampSec);
    if (!d) return `+${timestampSec}s`;
    const diffSec = Math.round((Date.now() - d.getTime()) / 1000);
    if (diffSec < 60) return `${diffSec}s ago`;
    if (diffSec < 3600) return `${Math.round(diffSec / 60)}m ago`;
    return `${Math.round(diffSec / 3600)}h ago`;
}

function renderLogTable(logs) {
    currentLogs = logs || [];
    const tbody = document.getElementById('log-body');
    const badge = document.getElementById('log-count-badge');
    if (!logs || logs.length === 0) {
        tbody.innerHTML = '<tr><td colspan="8" style="text-align:center;">No log data recorded yet.</td></tr>';
        if (badge) badge.innerText = '0 / 100 in Flash';
        renderTelemetryChart([]);
        return;
    }

    if (badge) {
        badge.innerText = `${logs.length} / 100 in Flash`;
    }

    tbody.innerHTML = logs.map(r => {
        const timeDisplay = formatReadingTime(r.timestamp, true);
        const relDisplay = formatReadingRelative(r.timestamp);

        const airDisplay = (r.air_temp_c !== undefined && r.air_temp_c !== null)
            ? `<span style="color: #06b6d4; font-weight: 500;">${r.air_temp_c.toFixed(1)}°C</span>`
            : '<span style="color: #64748b;">--</span>';

        const humDisplay = (r.humidity_pct !== undefined && r.humidity_pct !== null)
            ? `<span style="color: #3b82f6; font-weight: 500;">${Math.round(r.humidity_pct)}%</span>`
            : '<span style="color: #64748b;">--</span>';

        let lightDisplay = '<span style="color: #64748b;">--</span>';
        if (r.light_lux !== undefined && r.light_lux !== null) {
            lightDisplay = `<span style="color: #eab308; font-weight: 500;">${Math.round(r.light_lux)} lx</span>`;
        } else if (r.ldr_percent !== undefined && r.ldr_percent !== null) {
            lightDisplay = `<span style="color: #f97316; font-weight: 500;">${Math.round(r.ldr_percent)}%</span>`;
        }

        return `
        <tr>
            <td title="Recorded: ${timeDisplay} (${relDisplay}, boot +${Math.round(r.timestamp)}s)">
                <div style="font-weight: 500; color: #f8fafc;">${timeDisplay}</div>
                <div style="font-size: 0.72rem; color: #64748b;">${relDisplay}</div>
            </td>
            <td>${r.soil_percent}%</td>
            <td>${r.water_level_cm} cm</td>
            <td>${r.supply_voltage.toFixed(2)}V${r.battery_voltage > 0.5 ? ' / ' + r.battery_voltage.toFixed(2) + 'V Bat' : ''}</td>
            <td>${r.chip_temp.toFixed(1)}°C</td>
            <td>${airDisplay}</td>
            <td>${humDisplay}</td>
            <td>${lightDisplay}</td>
        </tr>
    `}).join('');

    renderTelemetryChart(currentLogs);
}

// =============================================================================
// Telemetry Trends Interactive Chart & Series Toggles
// =============================================================================
let currentLogs = [];
let chartHoverIndex = null;
let activeSeries = {
    soil: true,
    water: true,
    supply: true,
    battery: false,
    temp: false,
    air_temp: true,
    humidity: true,
    light_lux: true,
    ldr: false,
};

try {
    const saved = localStorage.getItem('plant_chart_series');
    if (saved) activeSeries = Object.assign(activeSeries, JSON.parse(saved));
} catch (_) {}

const SERIES_DEFS = {
    soil: {
        id: 'soil',
        name: 'Soil Moisture',
        unit: '%',
        color: '#10b981',
        get: r => r.soil_percent,
        min: 0,
        max: 100,
    },
    water: {
        id: 'water',
        name: 'Water Depth',
        unit: ' cm',
        color: '#38bdf8',
        get: r => r.water_level_cm,
        min: 0,
        max: 30,
    },
    supply: {
        id: 'supply',
        name: 'Supply Rail',
        unit: 'V',
        color: '#f59e0b',
        get: r => r.supply_voltage,
        min: 4.0,
        max: 6.0,
    },
    battery: {
        id: 'battery',
        name: 'Battery',
        unit: 'V',
        color: '#a855f7',
        get: r => r.battery_voltage,
        min: 2.8,
        max: 4.4,
    },
    temp: {
        id: 'temp',
        name: 'Chip Temp',
        unit: '°C',
        color: '#f43f5e',
        get: r => r.chip_temp,
        min: 20,
        max: 75,
    },
    air_temp: {
        id: 'air_temp',
        name: 'Air Temp',
        unit: '°C',
        color: '#06b6d4',
        get: r => (r.air_temp_c !== undefined ? r.air_temp_c : null),
        min: 0,
        max: 50,
    },
    humidity: {
        id: 'humidity',
        name: 'Humidity',
        unit: '%',
        color: '#3b82f6',
        get: r => (r.humidity_pct !== undefined ? r.humidity_pct : null),
        min: 0,
        max: 100,
    },
    light_lux: {
        id: 'light_lux',
        name: 'Ambient Light',
        unit: ' lx',
        color: '#eab308',
        get: r => (r.light_lux !== undefined ? r.light_lux : null),
        min: 0,
        max: 5000,
    },
    ldr: {
        id: 'ldr',
        name: 'LDR Light',
        unit: '%',
        color: '#f97316',
        get: r => (r.ldr_percent !== undefined ? r.ldr_percent : null),
        min: 0,
        max: 100,
    },
};

let chartAutoScale = false;
try {
    const savedScale = localStorage.getItem('plant_chart_autoscale');
    if (savedScale !== null) chartAutoScale = savedScale === 'true';
} catch (_) {}

const CHART_PAD = { left: 65, right: 20, top: 20, bottom: 35 };

function setupChartToggles() {
    const container = document.getElementById('graph-toggles');
    if (container) {
        const pills = container.querySelectorAll('.series-pill');
        pills.forEach(pill => {
            const key = pill.dataset.series;
            pill.classList.toggle('active', !!activeSeries[key]);
            pill.addEventListener('click', () => {
                activeSeries[key] = !activeSeries[key];
                pill.classList.toggle('active', activeSeries[key]);
                try {
                    localStorage.setItem('plant_chart_series', JSON.stringify(activeSeries));
                } catch (_) {}
                renderTelemetryChart(currentLogs);
            });
        });
    }

    const autoScaleBtn = document.getElementById('chart-autoscale-btn');
    if (autoScaleBtn) {
        autoScaleBtn.classList.toggle('active', chartAutoScale);
        const label = document.getElementById('autoscale-btn-text');
        if (label) label.innerText = chartAutoScale ? 'Auto-Scale: On' : 'Auto-Scale: Off';

        autoScaleBtn.addEventListener('click', () => {
            chartAutoScale = !chartAutoScale;
            autoScaleBtn.classList.toggle('active', chartAutoScale);
            if (label) label.innerText = chartAutoScale ? 'Auto-Scale: On' : 'Auto-Scale: Off';
            try {
                localStorage.setItem('plant_chart_autoscale', chartAutoScale);
            } catch (_) {}
            renderTelemetryChart(currentLogs);
        });
    }

    const canvas = document.getElementById('telemetry-chart');
    if (canvas) {
        canvas.addEventListener('mousemove', handleChartMouseMove);
        canvas.addEventListener('mouseleave', handleChartMouseLeave);
        canvas.addEventListener('touchmove', handleChartTouchMove, { passive: true });
        canvas.addEventListener('touchend', handleChartMouseLeave);
    }

    window.addEventListener('resize', () => {
        const viewLogs = document.getElementById('view-logs');
        if (viewLogs && viewLogs.style.display !== 'none') {
            renderTelemetryChart(currentLogs);
        }
    });
}

function handleChartMouseMove(e) {
    const canvas = document.getElementById('telemetry-chart');
    if (!canvas || !currentLogs || currentLogs.length === 0) return;
    const rect = canvas.getBoundingClientRect();
    const pad = CHART_PAD;
    const plotWidth = rect.width - pad.left - pad.right;
    const pts = [...currentLogs].reverse();
    if (pts.length < 2) return;

    const mouseX = e.clientX - rect.left - pad.left;
    const ratio = Math.max(0, Math.min(1, mouseX / plotWidth));
    const idx = Math.round(ratio * (pts.length - 1));
    chartHoverIndex = idx;
    renderTelemetryChart(currentLogs);
    showChartTooltip(pts[idx], idx, pts.length, rect);
}

function handleChartTouchMove(e) {
    if (e.touches && e.touches[0]) {
        handleChartMouseMove(e.touches[0]);
    }
}

function handleChartMouseLeave() {
    chartHoverIndex = null;
    const tooltip = document.getElementById('chart-tooltip');
    if (tooltip) tooltip.style.display = 'none';
    renderTelemetryChart(currentLogs);
}

function showChartTooltip(pt, idx, total, canvasRect) {
    const tooltip = document.getElementById('chart-tooltip');
    if (!tooltip) return;

    const pad = CHART_PAD;
    const plotWidth = canvasRect.width - pad.left - pad.right;
    const xPos = pad.left + (idx / (total - 1 || 1)) * plotWidth;

    let rowsHtml = '';
    for (const [key, enabled] of Object.entries(activeSeries)) {
        if (!enabled) continue;
        const def = SERIES_DEFS[key];
        const val = def.get(pt);
        const formatted = (typeof val === 'number')
            ? (def.unit === 'V' ? val.toFixed(2) : (def.unit === '°C' ? val.toFixed(1) : (def.unit === ' cm' ? val.toFixed(1) : val.toFixed(0))))
            : val;
        rowsHtml += `
            <div class="chart-tooltip-row">
                <span style="display:inline-block;width:8px;height:8px;border-radius:50%;background:${def.color};"></span>
                <span style="color:#cbd5e1;">${def.name}:</span>
                <strong style="color:#ffffff;margin-left:auto;">${formatted}${def.unit}</strong>
            </div>
        `;
    }

    if (!rowsHtml) {
        tooltip.style.display = 'none';
        return;
    }

    tooltip.innerHTML = `
        <div class="chart-tooltip-title">${formatReadingTime(pt.timestamp, true)} (${formatReadingRelative(pt.timestamp)})</div>
        ${rowsHtml}
    `;
    tooltip.style.display = 'block';

    const targetX = Math.max(70, Math.min(canvasRect.width - 70, xPos));
    const targetY = 32;

    tooltip.style.left = `${targetX}px`;
    tooltip.style.top = `${targetY}px`;
}

function computeChartScales(pts, activeKeys, isAutoScale) {
    if (!pts || pts.length === 0 || activeKeys.length === 0) {
        return {
            mode: 'empty',
            yLabels: ['100%', '75%', '50%', '25%', '0%'],
            seriesRanges: {},
            badgeText: 'No Sensor Selected'
        };
    }

    const activeUnits = new Set(activeKeys.map(k => SERIES_DEFS[k].unit.trim()));
    const isSingleUnit = activeUnits.size === 1;
    const unitStr = isSingleUnit ? [...activeUnits][0] : null;

    // Gather min & max for each active series from current data
    const seriesDataRanges = {};
    activeKeys.forEach(k => {
        const def = SERIES_DEFS[k];
        const vals = pts.map(pt => def.get(pt)).filter(v => v !== null && !isNaN(v));
        if (vals.length === 0) {
            seriesDataRanges[k] = { min: def.min, max: def.max, span: def.max - def.min };
        } else {
            let minVal = Math.min(...vals);
            let maxVal = Math.max(...vals);
            if (minVal === maxVal) {
                const defaultDelta = (k === 'temp') ? 2 : (k === 'supply' || k === 'battery') ? 0.2 : (k === 'water') ? 1 : 5;
                minVal -= defaultDelta;
                maxVal += defaultDelta;
            }
            seriesDataRanges[k] = { min: minVal, max: maxVal, span: maxVal - minVal };
        }
    });

    const seriesRanges = {};

    if (!isAutoScale) {
        // --- FIXED SCALE MODE ---
        if (isSingleUnit) {
            let fixedMin = Math.min(...activeKeys.map(k => SERIES_DEFS[k].min));
            let fixedMax = Math.max(...activeKeys.map(k => SERIES_DEFS[k].max));
            if (activeKeys.length === 1 && activeKeys[0] === 'temp') {
                fixedMin = 20;
                fixedMax = 80;
            }
            const span = fixedMax - fixedMin || 1;
            activeKeys.forEach(k => {
                seriesRanges[k] = { min: fixedMin, max: fixedMax };
            });

            const yLabels = [];
            const gridSteps = 4;
            for (let i = 0; i <= gridSteps; i++) {
                const val = fixedMax - (i / gridSteps) * span;
                const formatted = (unitStr === 'V') ? val.toFixed(1) : (unitStr === ' cm') ? val.toFixed(0) : Math.round(val);
                yLabels.push(`${formatted}${unitStr}`);
            }
            return {
                mode: 'fixed-unit',
                yLabels,
                seriesRanges,
                badgeText: `Fixed (${fixedMin}–${fixedMax}${unitStr})`
            };
        } else {
            activeKeys.forEach(k => {
                seriesRanges[k] = { min: SERIES_DEFS[k].min, max: SERIES_DEFS[k].max };
            });
            return {
                mode: 'fixed-normalized',
                yLabels: ['100%', '75%', '50%', '25%', '0%'],
                seriesRanges,
                badgeText: 'Fixed (0–100%)'
            };
        }
    }

    // --- AUTO-SCALE MODE ---
    if (isSingleUnit) {
        let dataMin = Math.min(...activeKeys.map(k => seriesDataRanges[k].min));
        let dataMax = Math.max(...activeKeys.map(k => seriesDataRanges[k].max));
        const rawSpan = dataMax - dataMin;
        const pad = Math.max(rawSpan * 0.12, (unitStr === 'V') ? 0.04 : 0.4);
        const autoMin = dataMin - pad;
        const autoMax = dataMax + pad;
        const span = autoMax - autoMin || 1;

        activeKeys.forEach(k => {
            seriesRanges[k] = { min: autoMin, max: autoMax };
        });

        const yLabels = [];
        const gridSteps = 4;
        for (let i = 0; i <= gridSteps; i++) {
            const val = autoMax - (i / gridSteps) * span;
            const decimals = (unitStr === 'V') ? 2 : (span < 5 && unitStr !== '%') ? 1 : 0;
            yLabels.push(`${val.toFixed(decimals)}${unitStr}`);
        }

        return {
            mode: 'auto-unit',
            yLabels,
            seriesRanges,
            badgeText: `Auto (${dataMin.toFixed(1)}–${dataMax.toFixed(1)}${unitStr})`
        };
    } else {
        // Mixed units: each active series is dynamically scaled to fit full vertical span
        activeKeys.forEach(k => {
            const r = seriesDataRanges[k];
            const pad = Math.max(r.span * 0.12, 0.01);
            seriesRanges[k] = { min: r.min - pad, max: r.max + pad };
        });

        const yLabels = ['Max (100%)', '75%', '50%', '25%', 'Min (0%)'];
        return {
            mode: 'auto-mixed',
            yLabels,
            seriesRanges,
            badgeText: 'Auto (Per-Sensor Fit)'
        };
    }
}

function renderTelemetryChart(logs) {
    const canvas = document.getElementById('telemetry-chart');
    if (!canvas) return;

    const rect = canvas.getBoundingClientRect();
    if (rect.width === 0 || rect.height === 0) return;

    const dpr = window.devicePixelRatio || 1;
    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;
    const ctx = canvas.getContext('2d');
    ctx.resetTransform();
    ctx.scale(dpr, dpr);

    const pad = CHART_PAD;
    const plotWidth = rect.width - pad.left - pad.right;
    const plotHeight = rect.height - pad.top - pad.bottom;

    ctx.clearRect(0, 0, rect.width, rect.height);

    if (!logs || logs.length === 0) {
        ctx.fillStyle = '#64748b';
        ctx.font = '14px system-ui, sans-serif';
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        ctx.fillText('No telemetry log data available yet.', rect.width / 2, rect.height / 2);
        return;
    }

    const pts = [...logs].reverse();
    const activeKeys = Object.keys(activeSeries).filter(k => activeSeries[k]);

    const scaleInfo = computeChartScales(pts, activeKeys, chartAutoScale);

    const scaleBadge = document.getElementById('chart-scale-badge');
    if (scaleBadge) scaleBadge.innerText = scaleInfo.badgeText;

    if (activeKeys.length === 0) {
        ctx.fillStyle = '#64748b';
        ctx.font = '14px system-ui, sans-serif';
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        ctx.fillText('Select one or more sensors above to plot telemetry.', rect.width / 2, rect.height / 2);
        return;
    }

    // 1. Draw Grid Lines and Y-Axis Labels
    ctx.lineWidth = 1;
    ctx.strokeStyle = '#1e293b';
    ctx.fillStyle = '#64748b';
    ctx.font = '10px system-ui, sans-serif';
    ctx.textAlign = 'right';
    ctx.textBaseline = 'middle';

    const gridSteps = scaleInfo.yLabels.length - 1;
    for (let i = 0; i <= gridSteps; i++) {
        const y = pad.top + (i / gridSteps) * plotHeight;
        const label = scaleInfo.yLabels[i];

        ctx.beginPath();
        ctx.setLineDash([4, 4]);
        ctx.moveTo(pad.left, y);
        ctx.lineTo(pad.left + plotWidth, y);
        ctx.stroke();

        ctx.setLineDash([]);
        ctx.fillText(label, pad.left - 8, y);
    }

    // 2. X-Axis Time Labels
    ctx.textAlign = 'center';
    ctx.textBaseline = 'top';
    const xLabelCount = Math.min(5, pts.length);
    for (let k = 0; k < xLabelCount; k++) {
        const idx = Math.round((k / (xLabelCount - 1 || 1)) * (pts.length - 1));
        const pt = pts[idx];
        const x = pad.left + (pts.length > 1 ? (idx / (pts.length - 1)) * plotWidth : plotWidth / 2);
        ctx.fillText(formatReadingTime(pt.timestamp, false), x, pad.top + plotHeight + 10);
    }

    // 3. Draw Active Series Curves
    for (const key of activeKeys) {
        const def = SERIES_DEFS[key];
        const range = scaleInfo.seriesRanges[key] || { min: def.min, max: def.max };
        const span = range.max - range.min || 1;

        // Skip plotting if all data points are null/undefined for this series
        const validValues = pts.map(pt => def.get(pt)).filter(v => v !== null && v !== undefined && !isNaN(v));
        if (validValues.length === 0) continue;

        const coords = pts.map((pt, i) => {
            const rawVal = def.get(pt);
            if (rawVal === null || rawVal === undefined || isNaN(rawVal)) {
                return null;
            }
            const clamped = Math.max(range.min, Math.min(range.max, rawVal));
            const norm = (clamped - range.min) / span;
            const x = pad.left + (pts.length > 1 ? (i / (pts.length - 1)) * plotWidth : plotWidth / 2);
            const y = pad.top + (1 - norm) * plotHeight;
            return { x, y, val: rawVal };
        });

        // Draw Line with gap support for missing readings
        ctx.beginPath();
        ctx.setLineDash([]);
        ctx.lineWidth = 2.5;
        ctx.strokeStyle = def.color;
        ctx.lineJoin = 'round';
        ctx.lineCap = 'round';

        let inSegment = false;
        coords.forEach(c => {
            if (!c) {
                inSegment = false;
                return;
            }
            if (!inSegment) {
                ctx.moveTo(c.x, c.y);
                inSegment = true;
            } else {
                ctx.lineTo(c.x, c.y);
            }
        });
        ctx.stroke();

        // Optional Dots on Points if <= 40 readings
        if (pts.length <= 40) {
            ctx.fillStyle = def.color;
            coords.forEach(c => {
                if (!c) return;
                ctx.beginPath();
                ctx.arc(c.x, c.y, 3, 0, Math.PI * 2);
                ctx.fill();
            });
        }
    }

    // 4. Draw Crosshair on Hover
    if (chartHoverIndex !== null && chartHoverIndex >= 0 && chartHoverIndex < pts.length) {
        const hIdx = chartHoverIndex;
        const hoverX = pad.left + (pts.length > 1 ? (hIdx / (pts.length - 1)) * plotWidth : plotWidth / 2);

        // Vertical Guide Line
        ctx.beginPath();
        ctx.setLineDash([3, 3]);
        ctx.strokeStyle = '#94a3b8';
        ctx.lineWidth = 1;
        ctx.moveTo(hoverX, pad.top);
        ctx.lineTo(hoverX, pad.top + plotHeight);
        ctx.stroke();
        ctx.setLineDash([]);

        // Highlight Active Series Dots at Hover Index
        const pt = pts[hIdx];
        for (const key of activeKeys) {
            const def = SERIES_DEFS[key];
            const range = scaleInfo.seriesRanges[key] || { min: def.min, max: def.max };
            const span = range.max - range.min || 1;
            const val = def.get(pt);
            const clamped = Math.max(range.min, Math.min(range.max, val));
            const norm = (clamped - range.min) / span;
            const hy = pad.top + (1 - norm) * plotHeight;

            // Outer Glow Ring
            ctx.beginPath();
            ctx.arc(hoverX, hy, 6, 0, Math.PI * 2);
            ctx.fillStyle = '#0f172a';
            ctx.fill();
            ctx.lineWidth = 2;
            ctx.strokeStyle = def.color;
            ctx.stroke();

            // Inner Dot
            ctx.beginPath();
            ctx.arc(hoverX, hy, 3, 0, Math.PI * 2);
            ctx.fillStyle = def.color;
            ctx.fill();
        }
    }
}
