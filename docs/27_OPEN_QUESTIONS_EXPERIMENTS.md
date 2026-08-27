# 27 — Open Questions and Automated Experiments

Resolve these empirically early instead of guessing.

## E01 — VPN Network socket binding from Rust

Prove that a Rust TCP socket file descriptor can be bound to the Android VPN `Network` before connect using the selected Kotlin/NDK bridge.

Evidence:

- network capabilities,
- bind return code,
- protected public IP,
- failure after VPN network loss.

## E02 — Local listener + VPN-bound egress simultaneously

Prove the process can accept a TCP client on Local-Only Hotspot while separate upstream sockets are explicitly VPN-bound without process-wide binding conflicts.

## E03 — VPN provider LAN blocking

Test representative VPN apps with their default LAN/local-network settings. Determine whether hotspot-local inbound traffic reaches VPNBridge and what settings are required.

## E04 — Wi-Fi STA + Local-Only Hotspot concurrency

Test target phones while Android upstream is Wi-Fi. Record whether the upstream remains active and VPN stable.

## E05 — DNS server reachability

Verify DNS servers reported by VPN `LinkProperties` can be reached from VPN-bound UDP/TCP sockets and identify VPNs with unusual resolver behavior.

## E06 — `tun2proxy` embedding

Evaluate current crate/API versus sidecar integration on Windows. Benchmark startup, throughput, UDP, DNS, IPv6, and recovery.

## E07 — WFP kill switch

Build an isolated test that permits only phone-local gateway traffic on physical hotspot while blocking raw external egress, then safely rolls back.

## E08 — Android screen-off/OEM behavior

Run 30-minute and multi-hour tests with screen off and UI closed.

Each experiment ends with a decision entry, reproducible test, and captured evidence.
