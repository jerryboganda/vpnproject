# 12 — Device and VPN Compatibility Matrix

Maintain a living matrix. Do not claim universal compatibility from emulator-only testing.

## Android dimensions

- Android 15 stock/near-stock device.
- Samsung One UI Android 15.
- Xiaomi/HyperOS Android 15 where available.
- OnePlus/OxygenOS Android 15 where available.
- Pixel Android 15 reference device.

For each record:

- device model,
- build number,
- API level,
- Local-Only Hotspot works,
- hotspot local address/subnet behavior,
- cellular upstream works,
- Wi-Fi upstream concurrency works,
- screen-off stability,
- OEM battery restrictions,
- notes.

## VPN dimensions

Test several implementations/protocols, for example:

- WireGuard-based client,
- OpenVPN-based client,
- ProtonVPN,
- Mullvad,
- another popular commercial VPN if available.

Record:

- VPNBridge active network reports `TRANSPORT_VPN`,
- VPN LAN/local-sharing setting needed,
- split-tunnel behavior,
- TCP,
- UDP,
- DNS,
- IPv6,
- server-switch recovery.

## Required v1 baseline

At least one stock/near-stock Android 15 device plus two materially different VPN implementations must pass the full acceptance matrix before claiming a broadly usable beta.
