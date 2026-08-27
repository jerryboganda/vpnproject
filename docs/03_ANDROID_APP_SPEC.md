# 03 — Android Application Specification

## Screens

### Home

Show:

- VPN status: Protected / Not protected / Revalidating.
- Hotspot status.
- Windows connection status.
- Protected sharing status.
- current throughput.
- primary **Share VPN** / **Stop** action.

### Pairing

- Local hotspot SSID.
- QR payload containing only local connection metadata and a short-lived pairing token; never expose long-term private keys.
- connected/pending device list.

### Diagnostics

- Android API/device/OEM.
- upstream type.
- VPN transport detected.
- VPN DNS servers (redacted/optional display).
- local hotspot address.
- last failure code.
- export sanitized diagnostic bundle.

### Settings

- auto-reconnect.
- Proxy Mode / Full Tunnel preference hint.
- DNS policy.
- IPv6 policy.
- maximum sessions.
- diagnostics level.

## Native service

Use a foreground service appropriate for continuous interaction with an external device over a network connection. Verify current Android requirements before implementation.

The service owns:

- hotspot reservation,
- VPN callbacks,
- validated VPN network handle,
- Rust gateway lifecycle,
- notification,
- session state.

The Activity/UI may die without terminating an active sharing session unless the user explicitly stops it.

## Permissions

At minimum evaluate and request only what current APIs require, including:

- INTERNET
- ACCESS_NETWORK_STATE
- CHANGE_WIFI_STATE / CHANGE_NETWORK_STATE as justified
- NEARBY_WIFI_DEVICES for Local-Only Hotspot on Android 13+
- FOREGROUND_SERVICE
- FOREGROUND_SERVICE_CONNECTED_DEVICE for target SDKs requiring it
- POST_NOTIFICATIONS where applicable to UX/OS behavior

Do not add privileged tethering permissions.

## Android 15 behavior

Avoid `dataSync` foreground-service type for an indefinitely running gateway. Android 15 limits dataSync background foreground-service time. Use the service type that truthfully represents connected-device/network interaction and satisfy its prerequisites.

## Lifecycle

- Start only from clear user action.
- Keep persistent notification while sharing.
- Handle task removal without corrupting service state.
- Handle OS process recreation.
- On service destruction, close gateway and hotspot reservation safely.
- On hotspot loss, notify Windows and fail closed.
