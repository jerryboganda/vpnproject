<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  interface WindowsStatus {
    is_connected: boolean;
    mode: string;
    is_kill_switch_armed: boolean;
    status_text: string;
  }

  interface Metrics {
    uptime_seconds: number;
    bytes_rx: number;
    bytes_tx: number;
    active_tcp_streams: number;
    total_tcp_connections: number;
    active_udp_mappings: number;
    total_udp_packets: number;
    vpn_drops_count: number;
  }

  let currentTab = $state<"dashboard" | "pair" | "settings">("dashboard");

  let status = $state<WindowsStatus>({
    is_connected: false,
    mode: "full_tunnel",
    is_kill_switch_armed: false,
    status_text: "Disconnected",
  });

  let metrics = $state<Metrics>({
    uptime_seconds: 0,
    bytes_rx: 0,
    bytes_tx: 0,
    active_tcp_streams: 0,
    total_tcp_connections: 0,
    active_udp_mappings: 0,
    total_udp_packets: 0,
    vpn_drops_count: 0,
  });

  let phoneIp = $state("192.168.43.1");
  let port = $state(10808);
  let authToken = $state("vpnbridge-secret-key");
  let selectedMode = $state("full_tunnel");
  let pairingUriInput = $state("");
  let isLoading = $state(false);
  let pairingFeedback = $state("");

  async function refreshStatus() {
    try {
      status = await invoke<WindowsStatus>("get_status");
      metrics = await invoke<Metrics>("get_metrics");
    } catch (e) {
      console.error("Failed to query status:", e);
    }
  }

  async function toggleConnection() {
    isLoading = true;
    try {
      if (status.is_connected) {
        await invoke("disconnect_tunnel");
      } else {
        await invoke("connect_tunnel", {
          phoneIp,
          port,
          authToken,
          mode: selectedMode,
        });
      }
      await refreshStatus();
    } catch (e) {
      alert("Error: " + e);
    } finally {
      isLoading = false;
    }
  }

  function parsePairingUri() {
    if (!pairingUriInput.startsWith("vpnbridge://pair?")) {
      pairingFeedback = "Invalid pairing URI format";
      return;
    }

    try {
      const url = new URL(pairingUriInput.replace("vpnbridge://pair?", "http://localhost/?"));
      const gw = url.searchParams.get("gw");
      const p = url.searchParams.get("port");
      const token = url.searchParams.get("token");

      if (gw) phoneIp = gw;
      if (p) port = parseInt(p, 10);
      if (token) authToken = token;

      pairingFeedback = "Pairing parameters imported successfully!";
      currentTab = "dashboard";
    } catch (e) {
      pairingFeedback = "Failed to parse URI: " + e;
    }
  }

  function formatBytes(bytes: number): string {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
  }

  onMount(() => {
    refreshStatus();
    const interval = setInterval(refreshStatus, 1000);
    return () => clearInterval(interval);
  });
</script>

<main class="container">
  <header style="margin-bottom: 1rem; text-align: center;">
    <h1 style="font-size: 1.75rem; font-weight: 800; color: #f8fafc;">VPNBridge Companion</h1>
    <p style="color: #94a3b8; font-size: 0.8125rem;">Windows Full Tunnel & Kill Switch Client</p>
  </header>

  <!-- Navigation Tabs -->
  <nav style="display: flex; gap: 0.5rem; margin-bottom: 1.25rem;">
    <button
      class="btn"
      style="padding: 0.5rem; font-size: 0.875rem; background: {currentTab === 'dashboard' ? '#0284c7' : '#1e293b'}; color: #fff;"
      onclick={() => (currentTab = "dashboard")}
    >
      Dashboard
    </button>
    <button
      class="btn"
      style="padding: 0.5rem; font-size: 0.875rem; background: {currentTab === 'pair' ? '#0284c7' : '#1e293b'}; color: #fff;"
      onclick={() => (currentTab = "pair")}
    >
      Pair from Phone
    </button>
    <button
      class="btn"
      style="padding: 0.5rem; font-size: 0.875rem; background: {currentTab === 'settings' ? '#0284c7' : '#1e293b'}; color: #fff;"
      onclick={() => (currentTab = "settings")}
    >
      Settings & WFP
    </button>
  </nav>

  {#if currentTab === "dashboard"}
    <!-- Connection Status Card -->
    <div class="card" style="text-align: center;">
      <div style="margin-bottom: 0.75rem;">
        {#if status.is_connected}
          <span class="badge badge-protected">PROTECTED & TUNNELED</span>
        {:else}
          <span class="badge badge-disconnected">DISCONNECTED</span>
        {/if}
      </div>

      <div style="font-size: 0.875rem; color: #94a3b8; margin-bottom: 1.25rem;">
        {status.status_text}
        {#if status.is_kill_switch_armed}
          <span style="color: #34d399; margin-left: 0.5rem;">[WFP Kill Switch Active]</span>
        {/if}
      </div>

      <button
        class="btn {status.is_connected ? 'btn-danger' : 'btn-primary'}"
        onclick={toggleConnection}
        disabled={isLoading}
      >
        {isLoading ? "Processing..." : status.is_connected ? "Disconnect" : "Connect Protected Tunnel"}
      </button>
    </div>

    <!-- Telemetry Card -->
    <div class="card">
      <h2 style="font-size: 1rem; font-weight: 700; margin-bottom: 1rem;">Tunnel Telemetry</h2>
      <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 0.75rem; font-size: 0.875rem;">
        <div style="background: rgba(0,0,0,0.2); padding: 0.75rem; border-radius: 0.5rem;">
          <div style="color: #94a3b8; font-size: 0.75rem;">DOWNLOAD</div>
          <div style="font-size: 1.125rem; font-weight: 700; color: #34d399;">{formatBytes(metrics.bytes_rx)}</div>
        </div>
        <div style="background: rgba(0,0,0,0.2); padding: 0.75rem; border-radius: 0.5rem;">
          <div style="color: #94a3b8; font-size: 0.75rem;">UPLOAD</div>
          <div style="font-size: 1.125rem; font-weight: 700; color: #38bdf8;">{formatBytes(metrics.bytes_tx)}</div>
        </div>
        <div style="background: rgba(0,0,0,0.2); padding: 0.75rem; border-radius: 0.5rem;">
          <div style="color: #94a3b8; font-size: 0.75rem;">ACTIVE STREAMS</div>
          <div style="font-size: 1.125rem; font-weight: 700;">{metrics.active_tcp_streams} TCP</div>
        </div>
        <div style="background: rgba(0,0,0,0.2); padding: 0.75rem; border-radius: 0.5rem;">
          <div style="color: #94a3b8; font-size: 0.75rem;">KILL SWITCH</div>
          <div style="font-size: 1.125rem; font-weight: 700; color: {status.is_kill_switch_armed ? '#34d399' : '#94a3b8'};">
            {status.is_kill_switch_armed ? "ARMED" : "OFF"}
          </div>
        </div>
      </div>
    </div>
  {:else if currentTab === "pair"}
    <!-- Pair Screen -->
    <div class="card">
      <h2 style="font-size: 1.125rem; font-weight: 700; margin-bottom: 0.5rem;">Import QR Pairing URI</h2>
      <p style="color: #94a3b8; font-size: 0.8125rem; margin-bottom: 1rem;">
        Paste the URI generated by the phone app to configure gateway endpoints automatically.
      </p>

      <input
        type="text"
        bind:value={pairingUriInput}
        placeholder="vpnbridge://pair?ssid=...&gw=192.168.43.1&port=10808&token=..."
      />

      {#if pairingFeedback}
        <div style="font-size: 0.8125rem; color: #34d399; margin-bottom: 0.75rem;">
          {pairingFeedback}
        </div>
      {/if}

      <button class="btn btn-primary" onclick={parsePairingUri}>
        Import Pairing Config
      </button>
    </div>
  {:else if currentTab === "settings"}
    <!-- Settings Screen -->
    <div class="card">
      <h2 style="font-size: 1.125rem; font-weight: 700; margin-bottom: 0.75rem;">Connection Settings</h2>

      <label style="font-size: 0.75rem; color: #94a3b8; display: block; margin-bottom: 0.25rem;">Tunnel Mode</label>
      <select bind:value={selectedMode} disabled={status.is_connected}>
        <option value="full_tunnel">Full Tunnel (Wintun + WFP Kill Switch)</option>
        <option value="proxy">Proxy Mode (SOCKS5 Direct)</option>
      </select>

      <label style="font-size: 0.75rem; color: #94a3b8; display: block; margin-bottom: 0.25rem;">Phone Gateway IP</label>
      <input type="text" bind:value={phoneIp} disabled={status.is_connected} />

      <label style="font-size: 0.75rem; color: #94a3b8; display: block; margin-bottom: 0.25rem;">SOCKS5 Port</label>
      <input type="number" bind:value={port} disabled={status.is_connected} />

      <label style="font-size: 0.75rem; color: #94a3b8; display: block; margin-bottom: 0.25rem;">Authentication Token</label>
      <input type="password" bind:value={authToken} disabled={status.is_connected} />
    </div>
  {/if}
</main>
