package com.vpnbridge.android

import android.content.Context
import android.net.ConnectivityManager
import android.net.Network
import android.net.NetworkCapabilities
import android.net.NetworkRequest
import android.util.Log

/**
 * Monitors Android network changes and identifies the active TRANSPORT_VPN network handle.
 * Invokes callbacks to update the SOCKS5 gateway with the validated VPN Network.
 */
class VpnMonitor(
    private val context: Context,
    private val onVpnValidated: (network: Network, networkHandle: Long, dnsServers: List<String>) -> Unit,
    private val onVpnLost: () -> Unit
) {
    companion object {
        private const val TAG = "VPNBridge.VpnMonitor"
    }

    private val connectivityManager =
        context.getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager

    var activeVpnNetwork: Network? = null
        private set

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

                onVpnValidated(network, handle, dnsServers)
            }
        }

        override fun onCapabilitiesChanged(network: Network, caps: NetworkCapabilities) {
            if (caps.hasTransport(NetworkCapabilities.TRANSPORT_VPN)) {
                val handle = network.networkHandle
                Log.d(TAG, "VPN capabilities updated for network handle: $handle")
                activeVpnNetwork = network
                val linkProperties = connectivityManager.getLinkProperties(network)
                val dnsServers = linkProperties?.dnsServers?.map { it.hostAddress ?: "" }?.filter { it.isNotEmpty() }
                    ?: emptyList()
                onVpnValidated(network, handle, dnsServers)
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
        // First, check if there's already an active VPN network
        try {
            val allNetworks = connectivityManager.allNetworks
            for (net in allNetworks) {
                val caps = connectivityManager.getNetworkCapabilities(net)
                if (caps != null && caps.hasTransport(NetworkCapabilities.TRANSPORT_VPN)) {
                    val handle = net.networkHandle
                    Log.i(TAG, "Initial active VPN network detected: $handle")
                    activeVpnNetwork = net
                    val linkProps = connectivityManager.getLinkProperties(net)
                    val dns = linkProps?.dnsServers?.map { it.hostAddress ?: "" }?.filter { it.isNotEmpty() } ?: emptyList()
                    onVpnValidated(net, handle, dns)
                    break
                }
            }
        } catch (e: Exception) {
            Log.w(TAG, "Could not check initial active networks", e)
        }

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
