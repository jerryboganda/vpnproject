package com.vpnbridge.android

import android.content.Context
import android.net.ConnectivityManager
import android.net.Network
import android.net.NetworkCapabilities
import android.net.NetworkRequest
import android.os.Build
import android.util.Log

/**
 * Monitors Android network changes and identifies the active TRANSPORT_VPN network handle.
 * Invokes native callbacks to trigger generation advancement or fail-closed invalidation.
 */
class VpnMonitor(
    private val context: Context,
    private val onVpnValidated: (networkHandle: Long, dnsServers: List<String>) -> Unit,
    private val onVpnLost: () -> Unit
) {
    companion object {
        private const val TAG = "VPNBridge.VpnMonitor"
    }

    private val connectivityManager =
        context.getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager

    private var activeVpnNetwork: Network? = null

    private val networkCallback = object : ConnectivityManager.NetworkCallback() {
        override fun onAvailable(network: Network) {
            val caps = connectivityManager.getNetworkCapabilities(network) ?: return
            if (caps.hasTransport(NetworkCapabilities.TRANSPORT_VPN)) {
                val handle = network.networkHandle
                Log.i(TAG, "Validated TRANSPORT_VPN network handle: $handle")
                activeVpnNetwork = network

                val linkProperties = connectivityManager.getLinkProperties(network)
                val dnsServers = linkProperties?.dnsServers?.map { it.hostAddress ?: "" }?.filter { it.isNotEmpty() }
                    ?: emptyList()

                onVpnValidated(handle, dnsServers)
            }
        }

        override fun onCapabilitiesChanged(network: Network, caps: NetworkCapabilities) {
            if (caps.hasTransport(NetworkCapabilities.TRANSPORT_VPN)) {
                val handle = network.networkHandle
                Log.d(TAG, "VPN capabilities updated for network handle: $handle")
                val linkProperties = connectivityManager.getLinkProperties(network)
                val dnsServers = linkProperties?.dnsServers?.map { it.hostAddress ?: "" }?.filter { it.isNotEmpty() }
                    ?: emptyList()
                onVpnValidated(handle, dnsServers)
            } else if (activeVpnNetwork == network) {
                Log.w(TAG, "Network lost TRANSPORT_VPN capability; failing closed")
                activeVpnNetwork = null
                onVpnLost()
            }
        }

        override fun onLost(network: Network) {
            if (activeVpnNetwork == network) {
                Log.w(TAG, "Active VPN network lost: ${network.networkHandle}; failing closed immediately")
                activeVpnNetwork = null
                onVpnLost()
            }
        }
    }

    fun startMonitoring() {
        val request = NetworkRequest.Builder()
            .addTransportType(NetworkCapabilities.TRANSPORT_VPN)
            .removeCapability(NetworkCapabilities.NET_CAPABILITY_NOT_VPN)
            .build()

        connectivityManager.registerNetworkCallback(request, networkCallback)
        Log.i(TAG, "Registered VPN network callback")
    }

    fun stopMonitoring() {
        try {
            connectivityManager.unregisterNetworkCallback(networkCallback)
            activeVpnNetwork = null
            Log.i(TAG, "Unregistered VPN network callback")
        } catch (e: Exception) {
            Log.e(TAG, "Error unregistering network callback", e)
        }
    }
}
