# 07 — DNS, IPv6, and Leak Protection

## Principle

No protocol family or resolver path may escape merely because the primary TCP path is protected.

## DNS policy

Default: **Follow Android VPN**.

1. Obtain DNS servers from the validated VPN network's `LinkProperties`.
2. Send DNS requests through Android sockets bound to that VPN network.
3. If DNS server addresses change with the VPN network generation, invalidate cached configuration.
4. Cache only with correct TTL behavior and bounded memory.

Optional modes:

- user-selected DoH resolver over a VPN-bound TLS connection,
- user-selected DoT resolver over a VPN-bound TLS connection.

Never use raw Windows DNS while Full Tunnel is protected.

## DNS tests

- A/AAAA lookups.
- NXDOMAIN.
- large responses / TCP fallback.
- DNS change after VPN server switch.
- resolver unavailable.
- malformed response handling.
- repeated VPN drop during lookups.

## IPv6 policy

### Release option 1

Full dual-stack forwarding through VPNBridge.

### Release option 2

Explicit IPv6 block while protected if dual-stack is not ready.

No release may have implicit IPv6 bypass.

## Leak harness

Continuously generate:

- HTTPS/TCP requests to an IP echo endpoint,
- UDP probes to a controlled endpoint,
- DNS queries,
- IPv6 requests,
- frequent new connections.

Then repeatedly:

- disconnect/reconnect VPN,
- switch VPN servers,
- toggle phone upstream,
- restart VPNBridge gateway,
- suspend/resume Windows.

Record every successful response and observed public source address. Any response through raw ISP egress while protected/fail-closed state is expected is a release-blocking defect.
