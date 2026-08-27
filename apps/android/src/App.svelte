<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  interface SharingStatus {
    state: string;
    state_display: string;
    is_forwarding: boolean;
    generation: number;
    network_handle: number;
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

  let currentTab = $state<"home" | "pairing" | "diagnostics">("home");

  let status = $state<SharingStatus>({
    state: "Stopped",
    state_display: "Stopped",
    is_forwarding: false,
    generation: 0,
    network_handle: 0,
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

  let ssid = $state("AndroidAP_VPNBridge");
  let phoneIp = $state("192.168.43.1");
  let port = $state(10808);
  let authToken = $state("vpnbridge-secret-key");
  let isLoading = $state(false);
  let copyFeedback = $state(false);

  let qrUri = $derived(
    `vpnbridge://pair?ssid=${encodeURIComponent(ssid)}&gw=${phoneIp}&port=${port}&token=${encodeURIComponent(authToken)}&ts=${Math.floor(Date.now() / 1000)}&ttl=300&fp=verified`
  );

  async function refreshStatus() {
    try {
      status = await invoke<SharingStatus>("get_status");
      metrics = await invoke<Metrics>("get_metrics");
    } catch (e) {
      console.error("Failed to query status:", e);
    }
  }

  async function toggleSharing() {
    isLoading = true;
    try {
      if (status.is_forwarding) {
        await invoke("stop_sharing");
      } else {
        await invoke("start_sharing", { authToken });
      }
      await refreshStatus();
    } catch (e) {
      alert("Error: " + e);
    } finally {
      isLoading = false;
    }
  }

  function copyPairingCode() {
    navigator.clipboard.writeText(qrUri);
    copyFeedback = true;
    setTimeout(() => {
      copyFeedback = false;
    }, 2000);
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
    <h1 style="font-size: 1.5rem; font-weight: 800; color: #f8fafc;">VPNBridge Gateway</h1>
    <p style="color: #94a3b8; font-size: 0.8125rem;">No-Root Fail-Closed VPN Hotspot</p>
  </header>

  <!-- Navigation Tabs -->
  <nav style="display: flex; gap: 0.5rem; margin-bottom: 1rem;">
    <button
      class="btn"
      style="padding: 0.5rem; font-size: 0.875rem; background: {currentTab === 'home' ? '#3b82f6' : '#1e293b'};"
      onclick={() => (currentTab = "home")}
    >
      Dashboard
    </button>
    <button
      class="btn"
      style="padding: 0.5rem; font-size: 0.875rem; background: {currentTab === 'pairing' ? '#3b82f6' : '#1e293b'};"
      onclick={() => (currentTab = "pairing")}
    >
      QR Pairing
    </button>
    <button
      class="btn"
      style="padding: 0.5rem; font-size: 0.875rem; background: {currentTab === 'diagnostics' ? '#3b82f6' : '#1e293b'};"
      onclick={() => (currentTab = "diagnostics")}
    >
      Diagnostics
    </button>
  </nav>

  {#if currentTab === "home"}
    <!-- Status Card -->
    <div class="card" style="text-align: center;">
      <div style="margin-bottom: 0.75rem;">
        {#if status.is_forwarding}
          <span class="badge badge-protected">PROTECTED FORWARDING</span>
        {:else}
          <span class="badge badge-disconnected">{status.state_display}</span>
        {/if}
      </div>

      <div style="font-size: 0.875rem; color: #94a3b8; margin-bottom: 1.5rem;">
        {#if status.network_handle !== 0}
          <span>VPN Handle: {status.network_handle} (Gen {status.generation})</span>
        {:else}
          <span>No Active VPN Network Bound</span>
        {/if}
      </div>

      <button
        class="btn {status.is_forwarding ? 'btn-danger' : 'btn-primary'}"
        onclick={toggleSharing}
        disabled={isLoading}
      >
        {isLoading ? "Processing..." : status.is_forwarding ? "Stop Sharing" : "Share VPN"}
      </button>
    </div>

    <!-- Throughput & Telemetry Card -->
    <div class="card">
      <h2 style="font-size: 1rem; font-weight: 700; margin-bottom: 1rem;">Live Telemetry</h2>
      <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 0.75rem; font-size: 0.875rem;">
        <div style="background: rgba(0,0,0,0.2); padding: 0.75rem; border-radius: 0.5rem;">
          <div style="color: #94a3b8; font-size: 0.75rem;">DOWNLOAD (RX)</div>
          <div style="font-size: 1.125rem; font-weight: 700; color: #34d399;">{formatBytes(metrics.bytes_rx)}</div>
        </div>
        <div style="background: rgba(0,0,0,0.2); padding: 0.75rem; border-radius: 0.5rem;">
          <div style="color: #94a3b8; font-size: 0.75rem;">UPLOAD (TX)</div>
          <div style="font-size: 1.125rem; font-weight: 700; color: #60a5fa;">{formatBytes(metrics.bytes_tx)}</div>
        </div>
        <div style="background: rgba(0,0,0,0.2); padding: 0.75rem; border-radius: 0.5rem;">
          <div style="color: #94a3b8; font-size: 0.75rem;">ACTIVE STREAMS</div>
          <div style="font-size: 1.125rem; font-weight: 700;">{metrics.active_tcp_streams} TCP / {metrics.active_udp_mappings} UDP</div>
        </div>
        <div style="background: rgba(0,0,0,0.2); padding: 0.75rem; border-radius: 0.5rem;">
          <div style="color: #94a3b8; font-size: 0.75rem;">VPN DROPS / RECOVERIES</div>
          <div style="font-size: 1.125rem; font-weight: 700; color: #f87171;">{metrics.vpn_drops_count}</div>
        </div>
      </div>
    </div>
  {:else if currentTab === "pairing"}
    <!-- QR Pairing Code Screen -->
    <div class="card">
      <h2 style="font-size: 1.125rem; font-weight: 700; margin-bottom: 0.5rem;">Windows QR Pairing</h2>
      <p style="color: #94a3b8; font-size: 0.8125rem; margin-bottom: 1rem;">
        Scan or paste this URI on your Windows companion app to pair securely.
      </p>

      <div style="background: rgba(0,0,0,0.3); padding: 1rem; border-radius: 0.5rem; word-break: break-all; font-family: monospace; font-size: 0.75rem; margin-bottom: 1rem; border: 1px solid #334155;">
        {qrUri}
      </div>

      <button class="btn btn-primary" onclick={copyPairingCode}>
        {copyFeedback ? "Copied to Clipboard!" : "Copy Pairing URI"}
      </button>
    </div>
  {:else if currentTab === "diagnostics"}
    <!-- Diagnostics Screen -->
    <div class="card">
      <h2 style="font-size: 1.125rem; font-weight: 700; margin-bottom: 0.75rem;">System Diagnostics</h2>
      <div style="font-size: 0.8125rem; display: flex; flex-direction: column; gap: 0.5rem;">
        <div><strong>Hotspot IP:</strong> 192.168.43.1:10808</div>
        <div><strong>Active VPN Network Handle:</strong> {status.network_handle}</div>
        <div><strong>Generation Count:</strong> {status.generation}</div>
        <div><strong>Fail-Closed Status:</strong> {status.is_forwarding ? "Armored (Active)" : "Protected (Closed)"}</div>
        <div><strong>Total TCP Connections Handled:</strong> {metrics.total_tcp_connections}</div>
        <div><strong>Total UDP Datagrams Forwarded:</strong> {metrics.total_udp_packets}</div>
        <div><strong>Gateway Uptime:</strong> {metrics.uptime_seconds}s</div>
      </div>
    </div>
  {/if}
</main>
