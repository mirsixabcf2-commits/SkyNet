// SkyNet 60fps RFC 6455 WebSocket Telemetry Client & Worker Earnings Studio (Feature 006)
(function initWorkerStudioModule() {
  const NANO_PER_SKN = 1e9;
  const apiOrigin = window.location.protocol.startsWith('http')
    ? window.location.origin
    : 'http://127.0.0.1:8888';
  const wsProto = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  const wsHost = window.location.protocol.startsWith('http')
    ? window.location.host
    : '127.0.0.1:8888';
  const wsUrl = `${wsProto}//${wsHost}/ws/telemetry`;

  let wsSocket = null;
  let fallbackTimer = null;
  let framesThisSecond = 0;
  let measuredFps = 0;
  let lastSweptNano = 0;

  const localFallback = {
    governor_state: 'DeepIdle',
    preempt_latency_ms: 3.8,
    gpu_util_pct: 16,
    cpu_util_pct: 10,
    vram_used_pct: 24,
    temp_c: 49,
    on_battery: false,
    active_game_detected: false,
    fountain_shards_completed: 4,
    pouw_score_bps: 5200,
    slashed_count: 0,
    earned_nano_skn: 12_500_000_000,
    swept_to_cold_nano_skn: 0,
    earning_rate_skn_per_hr: 14.5,
    updated_at_sec: Math.floor(Date.now() / 1000),
    sweep_log: []
  };

  setInterval(() => {
    measuredFps = framesThisSecond;
    framesThisSecond = 0;
    const fpsEl = document.getElementById('wsFpsCounter');
    if (fpsEl) fpsEl.textContent = window.__skynetWsActive ? `${measuredFps} fps` : 'HTTP poll';
  }, 1000);

  function setStreamBadge(mode) {
    const el = document.getElementById('wsStreamBadge');
    if (!el) return;
    if (mode === 'ws') {
      el.textContent = '● 60FPS WEBSOCKET RFC 6455';
      el.className = 'px-2.5 py-0.5 rounded-full text-[11px] font-mono border border-emerald-500/40 bg-emerald-500/15 text-emerald-300';
    } else {
      el.textContent = '● HTTP FALLBACK POLLING';
      el.className = 'px-2.5 py-0.5 rounded-full text-[11px] font-mono border border-amber-500/40 bg-amber-500/15 text-amber-300';
    }
  }

  function governorBadgeStyle(state) {
    if (state === 'DeepIdle') return 'bg-emerald-500/20 text-emerald-300 border-emerald-500/40';
    if (state === 'LightIdle' || state === 'Throttled') return 'bg-sky-500/20 text-sky-300 border-sky-500/40';
    return 'bg-rose-500/20 text-rose-300 border-rose-500/50';
  }

  function setGauge(txtId, barId, val, suffix, maxVal) {
    const t = document.getElementById(txtId);
    const b = document.getElementById(barId);
    if (t) t.textContent = `${val}${suffix}`;
    if (b) b.style.width = `${Math.min(100, Math.max(4, Math.round((val * 100) / maxVal)))}%`;
  }

  function renderWorkerStudio(st, sweepEntries) {
    if (!st) return;
    const govEl = document.getElementById('wsGovernorBadge');
    if (govEl) {
      govEl.textContent = st.governor_state;
      govEl.className = `px-2.5 py-1 rounded-lg text-xs font-mono font-bold border ${governorBadgeStyle(st.governor_state)}`;
    }
    const latEl = document.getElementById('wsPreemptLatency');
    if (latEl) latEl.textContent = `${Number(st.preempt_latency_ms || 0).toFixed(2)} ms`;

    setGauge('wsGpuPct', 'wsGpuBar', st.gpu_util_pct || 0, '%', 100);
    setGauge('wsVramPct', 'wsVramBar', st.vram_used_pct || 0, '%', 100);
    setGauge('wsCpuPct', 'wsCpuBar', st.cpu_util_pct || 0, '%', 100);
    setGauge('wsTempC', 'wsTempBar', st.temp_c || 0, '°C', 100);

    const pwrEl = document.getElementById('wsBatteryBadge');
    if (pwrEl) {
      const gameTag = st.active_game_detected ? ' • 🎮 GAME ACTIVE' : '';
      pwrEl.textContent = (st.on_battery ? '🔋 Pin (Battery Guard)' : '⚡ Nguồn Sạc AC') + gameTag;
    }

    const setTxt = (id, v) => { const e = document.getElementById(id); if (e) e.textContent = v; };
    setTxt('wsFountainShards', Number(st.fountain_shards_completed || 0).toLocaleString());
    setTxt('wsPouwScore', `${st.pouw_score_bps} bps`);
    setTxt('wsSlashedCount', String(st.slashed_count || 0));
    setTxt('wsEarningRate', `${Number(st.earning_rate_skn_per_hr || 0).toFixed(2)} SKN/giờ`);
    setTxt('wsEarnedSkn', `${(Number(st.earned_nano_skn || 0) / NANO_PER_SKN).toFixed(3)} SKN`);
    setTxt('wsSweptColdSkn', `${(Number(st.swept_to_cold_nano_skn || 0) / NANO_PER_SKN).toFixed(3)} SKN`);

    if (Number(st.swept_to_cold_nano_skn || 0) > lastSweptNano && (!sweepEntries || !sweepEntries.length)) {
      const deltaSkn = ((Number(st.swept_to_cold_nano_skn) - lastSweptNano) / NANO_PER_SKN).toFixed(3);
      localFallback.sweep_log.unshift(`+${deltaSkn} SKN -> Ví Lạnh Chủ Nhân (0% burn)`);
      localFallback.sweep_log = localFallback.sweep_log.slice(0, 8);
    }
    lastSweptNano = Number(st.swept_to_cold_nano_skn || 0);

    const logEl = document.getElementById('wsSweepLog');
    const logs = (sweepEntries && sweepEntries.length) ? sweepEntries : localFallback.sweep_log;
    if (logEl) {
      logEl.innerHTML = logs.length
        ? logs.map(item => `<div class="py-0.5 border-b border-[var(--border)]/40 text-emerald-300">${item}</div>`).join('')
        : '<div class="text-[var(--muted-foreground)]">Chờ lãi ròng vượt ngưỡng 50 SKN (0% phí trao đổi chất)...</div>';
    }
  }

  async function pollHttpFallback() {
    if (window.__skynetWsActive) return;
    try {
      const res = await fetch(`${apiOrigin}/api/worker-studio`, { cache: 'no-store' });
      if (res.ok) {
        renderWorkerStudio(await res.json(), null);
        return;
      }
    } catch (_) {}
    localFallback.fountain_shards_completed += 1;
    localFallback.earned_nano_skn += 150_000_000;
    renderWorkerStudio(localFallback, localFallback.sweep_log);
  }

  function startHttpFallback() {
    window.__skynetWsActive = false;
    setStreamBadge('http');
    if (!fallbackTimer) fallbackTimer = setInterval(pollHttpFallback, 500);
    pollHttpFallback();
  }

  function connectWebSocket() {
    try {
      wsSocket = new WebSocket(wsUrl);
    } catch (_) {
      startHttpFallback();
      return;
    }
    wsSocket.onopen = () => {
      window.__skynetWsActive = true;
      setStreamBadge('ws');
      if (fallbackTimer) { clearInterval(fallbackTimer); fallbackTimer = null; }
    };
    wsSocket.onmessage = (ev) => {
      framesThisSecond += 1;
      try {
        const snap = JSON.parse(ev.data);
        if (typeof window.applyState === 'function' && snap.agents) window.applyState(snap);
        if (snap.worker_studio) renderWorkerStudio(snap.worker_studio, snap.yield_sweep_log);
      } catch (_) {}
    };
    wsSocket.onerror = () => { startHttpFallback(); };
    wsSocket.onclose = () => {
      startHttpFallback();
      setTimeout(connectWebSocket, 2500);
    };
  }

  async function triggerStudioAction(action) {
    try {
      const res = await fetch(`${apiOrigin}/api/control?action=${encodeURIComponent(action)}`, { method: 'POST' });
      if (res.ok) {
        if (!window.__skynetWsActive) await pollHttpFallback();
        return;
      }
    } catch (_) {}
    if (action === 'sim_game_launch') {
      Object.assign(localFallback, { governor_state: 'PausedGaming', active_game_detected: true, gpu_util_pct: 95, vram_used_pct: 74, preempt_latency_ms: 2.4, earning_rate_skn_per_hr: 0 });
    } else if (action === 'sim_thermal_spike') {
      Object.assign(localFallback, { governor_state: 'PausedThermal', active_game_detected: false, temp_c: 82, preempt_latency_ms: 1.9, earning_rate_skn_per_hr: 0 });
    } else if (action === 'sim_canary_fraud') {
      localFallback.slashed_count += 1;
      localFallback.pouw_score_bps = 0;
    } else if (action === 'sim_yield_sweep') {
      localFallback.swept_to_cold_nano_skn += 55 * NANO_PER_SKN;
      localFallback.sweep_log.unshift('+55.000 SKN -> Ví Lạnh Chủ Nhân (0% phí burn)');
    }
    renderWorkerStudio(localFallback, localFallback.sweep_log);
  }

  window.triggerStudioAction = triggerStudioAction;
  renderWorkerStudio(localFallback, localFallback.sweep_log);
  connectWebSocket();
})();
