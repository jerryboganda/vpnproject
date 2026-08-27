# 09 — UI/UX Specification

## Design goal

The networking is sophisticated; the user flow must not be.

## Android primary screen

Status hierarchy:

1. **VPN protection**
2. **Private hotspot**
3. **Laptop connection**
4. **Sharing state**

Primary button states:

- `Share VPN`
- `Preparing...`
- `Waiting for VPN`
- `Protected — Sharing`
- `Reconnecting...`
- `Stop Sharing`

Do not show a green/protected state unless the forwarding gate is actually safe.

## Windows primary screen

Show:

- paired phone,
- phone VPN protection state,
- connection mode: Proxy / Full Tunnel,
- kill switch: on/off,
- DNS: protected/blocking,
- IPv6: tunneled/blocked,
- throughput,
- Connect/Disconnect.

## Error UX

Every error must include:

- short human description,
- whether Internet is blocked or still safe,
- one recommended action,
- expandable technical code.

Examples:

- `VPNBridge is excluded from your Android VPN. Include VPNBridge in the VPN and try again.`
- `The phone VPN disconnected. Internet forwarding is blocked until protection returns.`
- `This phone cannot keep Wi-Fi upstream active while running the private hotspot. Use mobile data or a supported device.`

## Accessibility

- keyboard navigation on Windows,
- appropriate contrast,
- screen-reader labels,
- no status conveyed by color alone,
- minimum touch targets on Android,
- reduced-motion respect.

## UI performance

Update throughput counters at a human-friendly interval (for example 2–4 times/second), not per packet. Keep packet processing completely outside the WebView/UI thread.
