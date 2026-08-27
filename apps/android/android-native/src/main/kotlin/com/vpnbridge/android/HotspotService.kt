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
 * Foreground service managing the Local-Only Hotspot reservation and VPN network monitoring.
 * Uses FOREGROUND_SERVICE_CONNECTED_DEVICE type on modern Android targets (Android 14/15).
 */
class HotspotService : Service() {
    companion object {
        private const val TAG = "VPNBridge.HotspotService"
        private const val NOTIFICATION_ID = 1001
        private const val CHANNEL_ID = "vpnbridge_service_channel"
        const val ACTION_START = "com.vpnbridge.START"
        const val ACTION_STOP = "com.vpnbridge.STOP"
    }

    private var hotspotReservation: WifiManager.LocalOnlyHotspotReservation? = null
    private var vpnMonitor: VpnMonitor? = null

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

        // Initialize VPN monitor
        vpnMonitor = VpnMonitor(
            context = applicationContext,
            onVpnValidated = { handle, dnsServers ->
                Log.i(TAG, "VPN Validated with handle $handle; forwarding to Rust core")
                // Call native JNI / Tauri update_vpn_state(true, handle, dnsServers)
            },
            onVpnLost = {
                Log.w(TAG, "VPN Lost; triggering Rust core fail-closed")
                // Call native JNI / Tauri update_vpn_state(false, 0, emptyList)
            }
        )
        vpnMonitor?.startMonitoring()

        // Start Local-Only Hotspot reservation
        val wifiManager = applicationContext.getSystemService(Context.WIFI_SERVICE) as WifiManager
        try {
            wifiManager.startLocalOnlyHotspot(object : WifiManager.LocalOnlyHotspotCallback() {
                override fun onStarted(reservation: WifiManager.LocalOnlyHotspotReservation) {
                    super.onStarted(reservation)
                    hotspotReservation = reservation
                    val config = reservation.wifiConfiguration
                    Log.i(TAG, "Local-Only Hotspot started: SSID=${config?.SSID}")
                }

                override fun onStopped() {
                    super.onStopped()
                    Log.w(TAG, "Local-Only Hotspot stopped")
                    hotspotReservation = null
                }

                override fun onFailed(reason: Int) {
                    super.onFailed(reason)
                    Log.e(TAG, "Local-Only Hotspot failed with reason: $reason")
                }
            }, null)
        } catch (e: SecurityException) {
            Log.e(TAG, "Missing permissions to start Local-Only Hotspot", e)
        }
    }

    private fun stopGatewayService() {
        vpnMonitor?.stopMonitoring()
        vpnMonitor = null

        hotspotReservation?.close()
        hotspotReservation = null

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
