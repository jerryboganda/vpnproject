package com.vpnbridge.android

import android.Manifest
import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Build
import android.os.Bundle
import android.util.Log
import android.widget.Button
import android.widget.TextView
import android.widget.Toast
import androidx.appcompat.app.AppCompatActivity
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat

class MainActivity : AppCompatActivity() {

    private var isSharing = false
    private lateinit var statusText: TextView
    private lateinit var toggleButton: Button
    private lateinit var btnCopyPairing: Button

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
            btnCopyPairing = findViewById(R.id.btnCopyPairing)

            updateUi()

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

            btnCopyPairing.setOnClickListener {
                val clipboard = getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
                val clip = ClipData.newPlainText(
                    "VPNBridge Pairing",
                    "vpnbridge://pair?host=192.168.43.1&port=10808&token=vpnbridge-secret-key"
                )
                clipboard.setPrimaryClip(clip)
                Toast.makeText(this, "Pairing URI copied to clipboard!", Toast.LENGTH_SHORT).show()
            }

            requestRequiredPermissions()
        } catch (e: Throwable) {
            Log.e(TAG, "Error initializing MainActivity", e)
            Toast.makeText(this, "Initialization error: ${e.localizedMessage}", Toast.LENGTH_LONG).show()
        }
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
