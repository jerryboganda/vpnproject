# 04 — Android Networking Specification

This is the most safety-critical Android document.

## 1. Local-Only Hotspot

Use the public `WifiManager.startLocalOnlyHotspot(...)` API or its current supported equivalent.

Properties:

- local communication only,
- no direct Internet connectivity for hotspot clients,
- OS-provided SSID/security configuration,
- reservation object controls lifetime.

Never replace this with privileged system tethering in the primary design.

## 2. VPN discovery

VPNBridge must determine the network that is the default route for its own UID.

Preferred flow:

1. `ConnectivityManager.getActiveNetwork()` for initial state.
2. `getNetworkCapabilities(network)`.
3. require `hasTransport(NetworkCapabilities.TRANSPORT_VPN)`.
4. inspect capability changes and `LinkProperties` via callbacks.
5. retain the validated `Network` and its network handle only while current.

If the active network for VPNBridge is Wi-Fi/cellular rather than VPN, assume VPNBridge is excluded/split-tunneled or VPN is absent and disable forwarding.

## 3. Upstream socket binding — mandatory

Before an Internet-facing socket connects or a datagram is sent, bind it to the validated VPN `Network`.

Possible implementation paths:

- Kotlin/Java `Network.bindSocket(Socket/DatagramSocket/FileDescriptor)`, or
- NDK `android_setsocknetwork(networkHandle, fd)` using the handle obtained from Android.

Rust abstraction example conceptually:

```text
trait ProtectedSocketBinder {
    fn bind_tcp_fd_to_current_vpn(fd) -> Result<VpnGeneration>;
    fn bind_udp_fd_to_current_vpn(fd) -> Result<VpnGeneration>;
}
```

The binder must verify that the network generation is still current before allowing the flow to become active.

## 4. Never use bypass APIs

In external-VPN mode do not call `VpnService.protect()` for gateway upstream sockets. Its purpose is to bypass VPN routing, which is the opposite of this product's requirement.

Do not bind upstream sockets to the physical Wi-Fi/cellular network.

## 5. Local listeners

The SOCKS/control listener must bind only to the hotspot-local IP/interface.

Do not call `ConnectivityManager.bindProcessToNetwork(hotspotNetwork)` because that risks sending subsequently created upstream sockets through the local/underlying network.

If a specific local interface binding is required, scope it to the listener socket or local address only.

## 6. VPN generation model

Represent each validated VPN network as a monotonically increasing generation:

```text
VpnBinding {
  generation,
  network_handle,
  capabilities_hash,
  dns_snapshot,
  validated_at
}
```

Every active upstream flow records the generation it was created under.

On network loss/replacement:

1. increment generation / invalidate old binding,
2. set global forwarding gate false,
3. cancel all flows from old generation,
4. clear UDP mappings,
5. revalidate new VPN,
6. resume only after validation.

## 7. DNS

Read VPN `LinkProperties` DNS servers. Route DNS to those servers using VPN-bound sockets. For domain-based SOCKS requests, either resolve on the VPN network or pass domain names through a resolver that is itself VPN-bound.

## 8. Concurrent Wi-Fi caveat

Cellular upstream + Local-Only Hotspot is the primary baseline. Wi-Fi upstream + hotspot requires hardware/OEM STA+AP concurrency. Detect actual behavior and include it in the device matrix; never claim universal support without evidence.

## 9. Network race tests

Automate/perform:

- VPN disconnect during TCP transfer.
- VPN disconnect during rapid new TCP connects.
- VPN disconnect during UDP flood.
- VPN server change causing network replacement.
- upstream cellular/Wi-Fi transition while VPN reconnects.
- app split-tunnel exclusion.
- hotspot restart while VPN remains active.

The required result is fail-closed forwarding.
