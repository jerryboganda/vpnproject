package com.vpnbridge.android

import android.Manifest
import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.net.wifi.WifiManager
import android.os.Build
import android.os.Bundle
import android.util.Log
import android.widget.Button
import android.widget.TextView
import android.widget.Toast
import androidx.appcompat.app.AppCompatActivity
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat
import java.net.Inet4Address
import java.net.NetworkInterface

class MainActivity : AppCompatActivity() {

    private var isSharing = false
    private lateinit var statusText: TextView
    private lateinit var toggleButton: Button
    private lateinit var tvHotspotSsid: TextView
    private lateinit var tvHotspotPass: TextView
    private lateinit var tvLanIp: TextView
    private lateinit var btnCopyHotspotPairing: Button
    private lateinit var btnCopyLanPairing: Button

    private var detectedLanIp: String = "192.168.1.1"

    companion object {
        private const val TAG = "VPNBridge.MainActivity"
        private const val PERMISSION_REQUEST_CODE = 101
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        try {
            setContentView(R.layout.activity_main)

            statusText = findViewById(R.id.tvStatus)
            toggleButton = findViewById(R.id.btnToggle)
            tvHotspotSsid = findViewById(R.id.tvHotspotSsid)
            tvHotspotPass = findViewById(R.id.tvHotspotPass)
            tvLanIp = findViewById(R.id.tvLanIp)
            btnCopyHotspotPairing = findViewById(R.id.btnCopyHotspotPairing)
            btnCopyLanPairing = findViewById(R.id.btnCopyLanPairing)

            updateUi()
            refreshLanIp()

            HotspotService.onHotspotStarted = { ssid, pass ->
                runOnUiThread {
                    tvHotspotSsid.text = "Wi-Fi Name: $ssid"
                    tvHotspotPass.text = "Password: $pass"
                    Toast.makeText(this, "Hotspot active: $ssid", Toast.LENGTH_SHORT).show()
                }
            }

            HotspotService.onHotspotFailed = { reason ->
                runOnUiThread {
                    tvHotspotSsid.text = "Hotspot Status: $reason"
                    tvHotspotPass.text = "Ensure GPS/Location is ON in quick settings"
                }
            }

            toggleButton.setOnClickListener {
                if (!hasRequiredPermissions()) {
                    requestRequiredPermissions()
                    return@setOnClickListener
                }

                if (isSharing) {
                    stopSharing()
                } else {
                    startSharing()
                }
            }

            btnCopyHotspotPairing.setOnClickListener {
                copyToClipboard(
                    "VPNBridge Hotspot Pairing",
                    "vpnbridge://pair?host=192.168.43.1&port=10808&token=vpnbridge-secret-key"
                )
            }

            btnCopyLanPairing.setOnClickListener {
                copyToClipboard(
                    "VPNBridge LAN Pairing",
                    "vpnbridge://pair?host=$detectedLanIp&port=10808&token=vpnbridge-secret-key"
                )
            }

            requestRequiredPermissions()
        } catch (e: Throwable) {
            Log.e(TAG, "Error initializing MainActivity", e)
        }
    }

    private fun refreshLanIp() {
        try {
            val interfaces = NetworkInterface.getNetworkInterfaces()
            while (interfaces.hasMoreElements()) {
                val iface = interfaces.nextElement()
                if (iface.isLoopback || !iface.isUp) continue
                val addrs = iface.inetAddresses
                while (addrs.hasMoreElements()) {
                    val addr = addrs.nextElement()
                    if (addr is Inet4Address && !addr.isLoopbackAddress) {
                        val host = addr.hostAddress ?: ""
                        if (host.startsWith("192.168.") || host.startsWith("10.") || host.startsWith("172.")) {
                            detectedLanIp = host
                            tvLanIp.text = "Phone Wi-Fi IP: $host : 10808"
                            return
                        }
                    }
                }
            }
        } catch (e: Exception) {
            Log.w(TAG, "Could not detect local IP", e)
        }
        tvLanIp.text = "Phone Wi-Fi IP: 192.168.1.x : 10808"
    }

    private fun copyToClipboard(label: String, text: String) {
        val clipboard = getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
        val clip = ClipData.newPlainText(label, text)
        clipboard.setPrimaryClip(clip)
        Toast.makeText(this, "Copied pairing code to clipboard!", Toast.LENGTH_SHORT).show()
    }

    private fun startSharing() {
        try {
            val intent = Intent(this, HotspotService::class.java).apply {
                action = HotspotService.ACTION_START
            }
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                startForegroundService(intent)
            } else {
                startService(intent)
            }
            isSharing = true
            updateUi()
            refreshLanIp()
            Toast.makeText(this, "VPN Sharing Started", Toast.LENGTH_SHORT).show()
        } catch (e: Exception) {
            Log.e(TAG, "Failed to start sharing service", e)
            Toast.makeText(this, "Failed to start: ${e.message}", Toast.LENGTH_SHORT).show()
        }
    }

    private fun stopSharing() {
        try {
            val intent = Intent(this, HotspotService::class.java).apply {
                action = HotspotService.ACTION_STOP
            }
            startService(intent)
            isSharing = false
            updateUi()
            tvHotspotSsid.text = "Wi-Fi Name: (Hotspot Stopped)"
            tvHotspotPass.text = "Password: (Hotspot Stopped)"
            Toast.makeText(this, "VPN Sharing Stopped", Toast.LENGTH_SHORT).show()
        } catch (e: Exception) {
            Log.e(TAG, "Failed to stop sharing service", e)
        }
    }

    private fun updateUi() {
        if (isSharing) {
            statusText.text = "Status: Protected Sharing Active"
            toggleButton.text = "Stop Sharing"
        } else {
            statusText.text = "Status: Gateway Idle"
            toggleButton.text = "Share VPN Connection"
        }
    }

    private fun hasRequiredPermissions(): Boolean {
        val permissions = getPermissionsList()
        return permissions.all {
            ContextCompat.checkSelfPermission(this, it) == PackageManager.PERMISSION_GRANTED
        }
    }

    private fun getPermissionsList(): Array<String> {
        val list = mutableListOf(
            Manifest.permission.ACCESS_FINE_LOCATION,
            Manifest.permission.ACCESS_COARSE_LOCATION,
            Manifest.permission.ACCESS_NETWORK_STATE,
            Manifest.permission.CHANGE_NETWORK_STATE,
            Manifest.permission.ACCESS_WIFI_STATE,
            Manifest.permission.CHANGE_WIFI_STATE
        )
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            list.add(Manifest.permission.POST_NOTIFICATIONS)
            list.add(Manifest.permission.NEARBY_WIFI_DEVICES)
        }
        return list.toTypedArray()
    }

    private fun requestRequiredPermissions() {
        val permissions = getPermissionsList()
        val missing = permissions.filter {
            ContextCompat.checkSelfPermission(this, it) != PackageManager.PERMISSION_GRANTED
        }
        if (missing.isNotEmpty()) {
            ActivityCompat.requestPermissions(this, missing.toTypedArray(), PERMISSION_REQUEST_CODE)
        }
    }
}
