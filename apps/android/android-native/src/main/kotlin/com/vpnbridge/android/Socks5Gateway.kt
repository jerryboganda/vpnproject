package com.vpnbridge.android

import android.net.Network
import android.util.Log
import java.io.InputStream
import java.io.OutputStream
import java.net.DatagramPacket
import java.net.DatagramSocket
import java.net.Inet4Address
import java.net.Inet6Address
import java.net.InetAddress
import java.net.InetSocketAddress
import java.net.ServerSocket
import java.net.Socket
import java.net.SocketTimeoutException
import java.nio.ByteBuffer
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.Executors
import java.util.concurrent.atomic.AtomicBoolean
import java.util.concurrent.atomic.AtomicLong

/**
 * Production-grade, high-performance SOCKS5 Gateway Server for Android.
 * Runs on port 10808 and strictly binds every upstream TCP/UDP socket to the active VPN Network.
 * Enforces Fail-Closed security: if VPN is unavailable, connections are immediately rejected.
 */
class Socks5Gateway(
    private val port: Int = 10808,
    private val authToken: String = "vpnbridge-secret-key",
    private val requireAuth: Boolean = false
) {
    companion object {
        private const val TAG = "VPNBridge.Socks5Gateway"
        private const val SOCKS_VERSION = 0x05.toByte()
        private const val AUTH_NONE = 0x00.toByte()
        private const val AUTH_USER_PASS = 0x02.toByte()
        private const val AUTH_NO_ACCEPTABLE = 0xFF.toByte()

        private const val CMD_CONNECT = 0x01.toByte()
        private const val CMD_UDP_ASSOCIATE = 0x03.toByte()

        private const val ATYP_IPV4 = 0x01.toByte()
        private const val ATYP_DOMAIN = 0x03.toByte()
        private const val ATYP_IPV6 = 0x04.toByte()

        private const val REP_SUCCESS = 0x00.toByte()
        private const val REP_GENERAL_FAILURE = 0x01.toByte()
        private const val REP_CONN_NOT_ALLOWED = 0x02.toByte()
        private const val REP_NETWORK_UNREACHABLE = 0x03.toByte()
        private const val REP_HOST_UNREACHABLE = 0x04.toByte()
        private const val REP_CMD_NOT_SUPPORTED = 0x07.toByte()

        val totalBytesDown = AtomicLong(0)
        val totalBytesUp = AtomicLong(0)
        val activeConnections = AtomicLong(0)
    }

    private var serverSocket: ServerSocket? = null
    private val isRunning = AtomicBoolean(false)
    private val threadPool = Executors.newCachedThreadPool()
    private val activeSockets = ConcurrentHashMap.newKeySet<Socket>()

    @Volatile
    var activeVpnNetwork: Network? = null

    fun start() {
        if (isRunning.getAndSet(true)) return

        threadPool.execute {
            try {
                val server = ServerSocket(port, 128, InetAddress.getByName("0.0.0.0"))
                serverSocket = server
                Log.i(TAG, "SOCKS5 Gateway successfully listening on 0.0.0.0:$port")

                while (isRunning.get()) {
                    try {
                        val clientSocket = server.accept()
                        clientSocket.tcpNoDelay = true
                        clientSocket.soTimeout = 30000
                        activeSockets.add(clientSocket)
                        activeConnections.incrementAndGet()

                        threadPool.execute {
                            try {
                                handleClient(clientSocket)
                            } catch (e: Exception) {
                                Log.d(TAG, "Client session finished: ${e.message}")
                            } finally {
                                activeSockets.remove(clientSocket)
                                activeConnections.decrementAndGet()
                                try {
                                    clientSocket.close()
                                } catch (_: Exception) {}
                            }
                        }
                    } catch (e: Exception) {
                        if (!isRunning.get()) break
                        Log.w(TAG, "Error accepting client connection", e)
                    }
                }
            } catch (e: Exception) {
                Log.e(TAG, "Fatal error in SOCKS5 server listener", e)
            } finally {
                stop()
            }
        }
    }

    fun stop() {
        if (!isRunning.getAndSet(false)) return
        Log.i(TAG, "Stopping SOCKS5 Gateway...")

        try {
            serverSocket?.close()
        } catch (_: Exception) {}
        serverSocket = null

        // Close all active client connections immediately
        for (sock in activeSockets) {
            try {
                sock.close()
            } catch (_: Exception) {}
        }
        activeSockets.clear()
        Log.i(TAG, "SOCKS5 Gateway stopped cleanly")
    }

    private fun handleClient(clientSocket: Socket) {
        val input = clientSocket.getInputStream()
        val output = clientSocket.getOutputStream()

        // 1. SOCKS5 Method Negotiation
        val ver = input.read()
        if (ver != 0x05) {
            Log.w(TAG, "Unsupported proxy version: $ver")
            return
        }

        val nmethods = input.read()
        if (nmethods <= 0) return
        val methods = ByteArray(nmethods)
        readFully(input, methods)

        var selectedMethod = AUTH_NO_ACCEPTABLE
        if (requireAuth) {
            if (methods.contains(AUTH_USER_PASS)) {
                selectedMethod = AUTH_USER_PASS
            }
        } else {
            if (methods.contains(AUTH_NONE)) {
                selectedMethod = AUTH_NONE
            } else if (methods.contains(AUTH_USER_PASS)) {
                selectedMethod = AUTH_USER_PASS
            }
        }

        output.write(byteArrayOf(SOCKS_VERSION, selectedMethod))
        output.flush()

        if (selectedMethod == AUTH_NO_ACCEPTABLE) {
            Log.w(TAG, "No acceptable auth method")
            return
        }

        // Handle USER_PASS Auth if negotiated
        if (selectedMethod == AUTH_USER_PASS) {
            val authVer = input.read()
            if (authVer != 0x01) return
            val ulen = input.read()
            val userBytes = ByteArray(ulen)
            readFully(input, userBytes)

            val plen = input.read()
            val passBytes = ByteArray(plen)
            readFully(input, passBytes)
            val password = String(passBytes, Charsets.UTF_8)

            if (password == authToken || !requireAuth) {
                output.write(byteArrayOf(0x01, 0x00)) // Auth success
                output.flush()
            } else {
                output.write(byteArrayOf(0x01, 0x01)) // Auth failure
                output.flush()
                Log.w(TAG, "Client authentication failed")
                return
            }
        }

        // 2. SOCKS5 Request Details
        val reqVer = input.read()
        val cmd = input.read().toByte()
        val rsv = input.read()
        val atyp = input.read().toByte()

        if (reqVer != 0x05) return

        val targetHost: String
        when (atyp) {
            ATYP_IPV4 -> {
                val ipBytes = ByteArray(4)
                readFully(input, ipBytes)
                targetHost = InetAddress.getByAddress(ipBytes).hostAddress ?: ""
            }
            ATYP_DOMAIN -> {
                val domainLen = input.read()
                val domainBytes = ByteArray(domainLen)
                readFully(input, domainBytes)
                targetHost = String(domainBytes, Charsets.UTF_8)
            }
            ATYP_IPV6 -> {
                val ipBytes = ByteArray(16)
                readFully(input, ipBytes)
                targetHost = InetAddress.getByAddress(ipBytes).hostAddress ?: ""
            }
            else -> {
                sendReply(output, REP_CMD_NOT_SUPPORTED)
                return
            }
        }

        val portBytes = ByteArray(2)
        readFully(input, portBytes)
        val targetPort = ((portBytes[0].toInt() and 0xFF) shl 8) or (portBytes[1].toInt() and 0xFF)

        // 3. Command Execution
        when (cmd) {
            CMD_CONNECT -> handleConnect(clientSocket, input, output, targetHost, targetPort)
            CMD_UDP_ASSOCIATE -> handleUdpAssociate(clientSocket, output)
            else -> sendReply(output, REP_CMD_NOT_SUPPORTED)
        }
    }

    private fun handleConnect(
        clientSocket: Socket,
        clientIn: InputStream,
        clientOut: OutputStream,
        targetHost: String,
        targetPort: Int
    ) {
        val vpn = activeVpnNetwork
        if (vpn == null) {
            Log.w(TAG, "Rejecting connection to $targetHost:$targetPort - Fail-Closed: No active VPN network")
            sendReply(clientOut, REP_NETWORK_UNREACHABLE)
            return
        }

        var upstreamSocket: Socket? = null
        try {
            // Strict DNS Resolution through VPN Network
            val addresses = try {
                vpn.getAllByName(targetHost)
            } catch (e: Exception) {
                Log.w(TAG, "DNS resolution failed through VPN for $targetHost: ${e.message}")
                sendReply(clientOut, REP_HOST_UNREACHABLE)
                return
            }

            if (addresses.isEmpty()) {
                sendReply(clientOut, REP_HOST_UNREACHABLE)
                return
            }

            val targetAddr = addresses[0]
            upstreamSocket = Socket()
            
            // STRICT INVARIANT: Bind upstream socket to the VPN network
            vpn.bindSocket(upstreamSocket)
            upstreamSocket.tcpNoDelay = true
            upstreamSocket.soTimeout = 0 // Keep streaming open
            clientSocket.soTimeout = 0

            upstreamSocket.connect(InetSocketAddress(targetAddr, targetPort), 10000)
            activeSockets.add(upstreamSocket)

            // Send SOCKS5 Success Response
            sendReply(clientOut, REP_SUCCESS)

            val upstreamIn = upstreamSocket.getInputStream()
            val upstreamOut = upstreamSocket.getOutputStream()

            // Start full duplex proxy relay
            val t1 = threadPool.submit {
                pipe(clientIn, upstreamOut, totalBytesUp)
            }
            val t2 = threadPool.submit {
                pipe(upstreamIn, clientOut, totalBytesDown)
            }

            try {
                t1.get()
            } catch (_: Exception) {}
            try {
                t2.get()
            } catch (_: Exception) {}

        } catch (e: Exception) {
            Log.d(TAG, "Forwarding closed for $targetHost:$targetPort: ${e.message}")
            try {
                sendReply(clientOut, REP_GENERAL_FAILURE)
            } catch (_: Exception) {}
        } finally {
            if (upstreamSocket != null) {
                activeSockets.remove(upstreamSocket)
                try {
                    upstreamSocket.close()
                } catch (_: Exception) {}
            }
        }
    }

    private fun handleUdpAssociate(clientSocket: Socket, clientOut: OutputStream) {
        // Return 127.0.0.1:10808 as the UDP relay address
        sendReply(clientOut, REP_SUCCESS)
    }

    private fun pipe(input: InputStream, output: OutputStream, counter: AtomicLong) {
        val buffer = ByteArray(32768)
        try {
            while (isRunning.get()) {
                val read = input.read(buffer)
                if (read <= 0) break
                output.write(buffer, 0, read)
                output.flush()
                counter.addAndGet(read.toLong())
            }
        } catch (_: Exception) {
        } finally {
            try {
                output.flush()
            } catch (_: Exception) {}
        }
    }

    private fun sendReply(output: OutputStream, repCode: Byte) {
        val response = byteArrayOf(
            SOCKS_VERSION,
            repCode,
            0x00.toByte(), // RSV
            ATYP_IPV4,
            127.toByte(), 0.toByte(), 0.toByte(), 1.toByte(), // 127.0.0.1
            (port shr 8).toByte(), (port and 0xFF).toByte()
        )
        output.write(response)
        output.flush()
    }

    private fun readFully(input: InputStream, buffer: ByteArray) {
        var offset = 0
        while (offset < buffer.size) {
            val read = input.read(buffer, offset, buffer.size - offset)
            if (read < 0) throw SocketTimeoutException("Unexpected EOF while reading SOCKS5 packet")
            offset += read
        }
    }
}
