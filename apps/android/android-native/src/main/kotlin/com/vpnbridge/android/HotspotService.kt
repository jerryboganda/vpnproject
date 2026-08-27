package com.vpnbridge.android

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.net.wifi.WifiManager
import android.os.Build
import android.os.IBinder
import android.util.Log

/**
 * Foreground service managing the Local-Only Hotspot reservation, SOCKS5 Gateway server,
 * and VPN network monitoring.
 */
class HotspotService : Service() {
    companion object {
        private const val TAG = "VPNBridge.HotspotService"
        private const val NOTIFICATION_ID = 1001
        private const val CHANNEL_ID = "vpnbridge_service_channel"
        const val ACTION_START = "com.vpnbridge.START"
        const val ACTION_STOP = "com.vpnbridge.STOP"

        var onHotspotStarted: ((ssid: String, pass: String) -> Unit)? = null
        var onHotspotFailed: ((reason: String) -> Unit)? = null
        var currentSsid: String? = null
        var currentPass: String? = null
    }

    private var hotspotReservation: WifiManager.LocalOnlyHotspotReservation? = null
    private var vpnMonitor: VpnMonitor? = null
    private var socks5Gateway: Socks5Gateway? = null

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onCreate() {
        super.onCreate()
        createNotificationChannel()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        when (intent?.action) {
            ACTION_START -> startGatewayService()
            ACTION_STOP -> stopGatewayService()
        }
        return START_STICKY
    }

    private fun startGatewayService() {
        val notification = buildForegroundNotification("VPNBridge is actively sharing protected VPN")

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            startForeground(
                NOTIFICATION_ID,
                notification,
                ServiceInfo.FOREGROUND_SERVICE_TYPE_CONNECTED_DEVICE
            )
        } else {
            startForeground(NOTIFICATION_ID, notification)
        }

        // 1. Initialize SOCKS5 Gateway Server on port 10808
        val gateway = Socks5Gateway(port = 10808, authToken = "vpnbridge-secret-key", requireAuth = false)
        socks5Gateway = gateway
        gateway.start()

        // 2. Initialize VPN monitor and bridge active VPN network to SOCKS5 gateway
        vpnMonitor = VpnMonitor(
            context = applicationContext,
            onVpnValidated = { network, handle, dnsServers ->
                Log.i(TAG, "VPN Validated ($handle); binding gateway upstream to VPN network")
                socks5Gateway?.activeVpnNetwork = network
            },
            onVpnLost = {
                Log.w(TAG, "VPN Lost; triggering SOCKS5 gateway fail-closed")
                socks5Gateway?.activeVpnNetwork = null
            }
        )
        vpnMonitor?.startMonitoring()

        // 3. Start Local-Only Hotspot reservation
        val wifiManager = applicationContext.getSystemService(Context.WIFI_SERVICE) as WifiManager
        try {
            wifiManager.startLocalOnlyHotspot(object : WifiManager.LocalOnlyHotspotCallback() {
                override fun onStarted(reservation: WifiManager.LocalOnlyHotspotReservation) {
                    super.onStarted(reservation)
                    hotspotReservation = reservation
                    
                    var ssid = "AndroidAP_VPNBridge"
                    var pass = "None (Open or Check Hotspot Settings)"

                    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
                        try {
                            val config = reservation.softApConfiguration
                            ssid = config.ssid ?: ssid
                            pass = config.passphrase ?: "Open"
                        } catch (e: Throwable) {
                            Log.w(TAG, "Could not extract SoftApConfiguration: ${e.message}")
                        }
                    } else {
                        @Suppress("DEPRECATION")
                        val config = reservation.wifiConfiguration
                        ssid = config?.SSID ?: ssid
                        pass = config?.preSharedKey ?: "Open"
                    }

                    currentSsid = ssid
                    currentPass = pass
                    Log.i(TAG, "Local-Only Hotspot started: SSID=$ssid, Password=$pass")
                    onHotspotStarted?.invoke(ssid, pass)
                }

                override fun onStopped() {
                    super.onStopped()
                    Log.w(TAG, "Local-Only Hotspot stopped")
                    hotspotReservation = null
                    currentSsid = null
                    currentPass = null
                }

                override fun onFailed(reason: Int) {
                    super.onFailed(reason)
                    val reasonStr = when (reason) {
                        1 -> "ERROR_NO_CHANNEL (Check if Location is enabled)"
                        2 -> "ERROR_GENERIC (Hotspot already active or restricted)"
                        3 -> "ERROR_INCOMPATIBLE_MODE"
                        4 -> "ERROR_TETHERING_DISALLOWED"
                        else -> "Reason code $reason"
                    }
                    Log.e(TAG, "Local-Only Hotspot failed: $reasonStr")
                    onHotspotFailed?.invoke(reasonStr)
                }
            }, null)
        } catch (e: SecurityException) {
            Log.e(TAG, "Missing permissions to start Local-Only Hotspot", e)
            onHotspotFailed?.invoke("Permission error: ${e.localizedMessage}")
        } catch (e: Throwable) {
            Log.e(TAG, "Unexpected error starting hotspot", e)
            onHotspotFailed?.invoke("Error: ${e.localizedMessage}")
        }
    }

    private fun stopGatewayService() {
        vpnMonitor?.stopMonitoring()
        vpnMonitor = null

        socks5Gateway?.stop()
        socks5Gateway = null

        try {
            hotspotReservation?.close()
        } catch (e: Exception) {
            Log.w(TAG, "Error closing hotspot reservation", e)
        }
        hotspotReservation = null
        currentSsid = null
        currentPass = null

        stopForeground(STOP_FOREGROUND_REMOVE)
        stopSelf()
        Log.i(TAG, "Gateway service stopped cleanly")
    }

    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                CHANNEL_ID,
                "VPNBridge Sharing Gateway",
                NotificationManager.IMPORTANCE_LOW
            ).apply {
                description = "Shows active status when VPNBridge is sharing your VPN connection"
            }
            val manager = getSystemService(NotificationManager::class.java)
            manager.createNotificationChannel(channel)
        }
    }

    private fun buildForegroundNotification(contentText: String): Notification {
        val builder = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            Notification.Builder(this, CHANNEL_ID)
        } else {
            Notification.Builder(this)
        }

        return builder
            .setContentTitle("VPNBridge Sharing Active")
            .setContentText(contentText)
            .setSmallIcon(android.R.drawable.stat_notify_sync)
            .setOngoing(true)
            .build()
    }

    override fun onDestroy() {
        stopGatewayService()
        super.onDestroy()
    }
}
