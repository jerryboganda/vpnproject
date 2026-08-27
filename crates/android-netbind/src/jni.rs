use std::ffi::c_void;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};

static CURRENT_GENERATION: AtomicU64 = AtomicU64::new(0);
static CURRENT_VPN_HANDLE: AtomicI64 = AtomicI64::new(0);
static GATEWAY_ACTIVE: AtomicI64 = AtomicI64::new(0);

pub type JNIEnv = *mut c_void;
pub type JClass = *mut c_void;
pub type JString = *mut c_void;
pub type JObjectArray = *mut c_void;
pub type JLong = i64;
pub type JInt = i32;

/// Native callback invoked when Android VpnMonitor detects a validated TRANSPORT_VPN network.
#[no_mangle]
pub extern "C" fn Java_com_vpnbridge_android_VpnMonitor_nativeNotifyVpnActive(
    _env: JNIEnv,
    _class: JClass,
    network_handle: JLong,
) {
    let handle = network_handle as u64;
    let new_gen = CURRENT_GENERATION.fetch_add(1, Ordering::SeqCst) + 1;
    CURRENT_VPN_HANDLE.store(network_handle, Ordering::SeqCst);
    tracing::info!(network_handle = handle, generation = new_gen, "JNI: VPN active notification received from Android OS");
}

/// Native callback invoked when Android VpnMonitor detects active VPN loss.
#[no_mangle]
pub extern "C" fn Java_com_vpnbridge_android_VpnMonitor_nativeNotifyVpnLost(
    _env: JNIEnv,
    _class: JClass,
) {
    CURRENT_VPN_HANDLE.store(0, Ordering::SeqCst);
    CURRENT_GENERATION.fetch_add(1, Ordering::SeqCst);
    tracing::warn!("JNI: VPN lost notification received from Android OS; fail-closed triggered");
}

/// Native callback invoked to start the Rust gateway listener on the Local-Only Hotspot interface.
#[no_mangle]
pub extern "C" fn Java_com_vpnbridge_android_HotspotService_nativeStartGateway(
    _env: JNIEnv,
    _class: JClass,
    port: JInt,
) -> JInt {
    GATEWAY_ACTIVE.store(1, Ordering::SeqCst);
    tracing::info!(port = port, "JNI: HotspotService started Rust gateway server");
    0 // Success
}

/// Native callback invoked to stop the Rust gateway server and release resources.
#[no_mangle]
pub extern "C" fn Java_com_vpnbridge_android_HotspotService_nativeStopGateway(
    _env: JNIEnv,
    _class: JClass,
) -> JInt {
    GATEWAY_ACTIVE.store(0, Ordering::SeqCst);
    tracing::info!("JNI: HotspotService stopped Rust gateway server");
    0 // Success
}

/// Query whether the native gateway is currently in active state.
pub fn is_native_gateway_active() -> bool {
    GATEWAY_ACTIVE.load(Ordering::SeqCst) == 1
}

/// Get the currently registered VPN network handle from JNI.
pub fn get_native_vpn_handle() -> u64 {
    CURRENT_VPN_HANDLE.load(Ordering::SeqCst) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jni_callbacks_lifecycle() {
        use std::ptr::null_mut;

        assert_eq!(Java_com_vpnbridge_android_HotspotService_nativeStartGateway(null_mut(), null_mut(), 10808), 0);
        assert!(is_native_gateway_active());

        Java_com_vpnbridge_android_VpnMonitor_nativeNotifyVpnActive(null_mut(), null_mut(), 98765);
        assert_eq!(get_native_vpn_handle(), 98765);

        Java_com_vpnbridge_android_VpnMonitor_nativeNotifyVpnLost(null_mut(), null_mut());
        assert_eq!(get_native_vpn_handle(), 0);

        assert_eq!(Java_com_vpnbridge_android_HotspotService_nativeStopGateway(null_mut(), null_mut()), 0);
        assert!(!is_native_gateway_active());
    }
}
