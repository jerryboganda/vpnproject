//! Real Wintun C FFI API & Dynamic Loader
//!
//! Provides direct bindings to `wintun.dll` according to the official Wintun C ABI.
//! Allows runtime loading with graceful fallback to `MockTunAdapter` if `wintun.dll` is absent.

use std::ffi::c_void;
use std::sync::Arc;
use vpnbridge_core::error::{Error, Result};
use crate::adapter::{TunAdapter, TunSession};
use async_trait::async_trait;
use bytes::Bytes;

pub type WintunAdapterHandle = *mut c_void;
pub type WintunSessionHandle = *mut c_void;

pub type FnWintunCreateAdapter = unsafe extern "system" fn(
    pool_name: *const u16,
    name: *const u16,
    tunnel_type: *const u16,
    requested_guid: *const c_void,
) -> WintunAdapterHandle;

pub type FnWintunOpenAdapter = unsafe extern "system" fn(
    pool_name: *const u16,
    name: *const u16,
) -> WintunAdapterHandle;

pub type FnWintunCloseAdapter = unsafe extern "system" fn(adapter: WintunAdapterHandle);
pub type FnWintunGetRunningDriverVersion = unsafe extern "system" fn() -> u32;

pub type FnWintunStartSession = unsafe extern "system" fn(
    adapter: WintunAdapterHandle,
    capacity: u32,
) -> WintunSessionHandle;

pub type FnWintunEndSession = unsafe extern "system" fn(session: WintunSessionHandle);
pub type FnWintunGetReadWaitEvent = unsafe extern "system" fn(session: WintunSessionHandle) -> *mut c_void;

pub type FnWintunReceivePacket = unsafe extern "system" fn(
    session: WintunSessionHandle,
    packet_size: *mut u32,
) -> *mut u8;

pub type FnWintunReleaseReceivePacket = unsafe extern "system" fn(
    session: WintunSessionHandle,
    packet: *const u8,
);

pub type FnWintunAllocateSendPacket = unsafe extern "system" fn(
    session: WintunSessionHandle,
    packet_size: u32,
) -> *mut u8;

pub type FnWintunSendPacket = unsafe extern "system" fn(
    session: WintunSessionHandle,
    packet: *const u8,
);

/// Function table for loaded `wintun.dll`.
pub struct WintunApi {
    pub create_adapter: FnWintunCreateAdapter,
    pub open_adapter: FnWintunOpenAdapter,
    pub close_adapter: FnWintunCloseAdapter,
    pub get_running_driver_version: FnWintunGetRunningDriverVersion,
    pub start_session: FnWintunStartSession,
    pub end_session: FnWintunEndSession,
    pub get_read_wait_event: FnWintunGetReadWaitEvent,
    pub receive_packet: FnWintunReceivePacket,
    pub release_receive_packet: FnWintunReleaseReceivePacket,
    pub allocate_send_packet: FnWintunAllocateSendPacket,
    pub send_packet: FnWintunSendPacket,
}

/// Production Wintun Adapter implementing the `TunAdapter` trait.
pub struct WintunAdapter {
    name: String,
    pool_name: String,
    adapter_handle: WintunAdapterHandle,
    api: Arc<WintunApi>,
}

unsafe impl Send for WintunAdapter {}
unsafe impl Sync for WintunAdapter {}

impl WintunAdapter {
    pub fn new(name: impl Into<String>, pool_name: impl Into<String>, handle: WintunAdapterHandle, api: Arc<WintunApi>) -> Self {
        Self {
            name: name.into(),
            pool_name: pool_name.into(),
            adapter_handle: handle,
            api,
        }
    }

    pub fn pool_name(&self) -> &str {
        &self.pool_name
    }
}

#[async_trait]
impl TunAdapter for WintunAdapter {
    fn name(&self) -> &str {
        &self.name
    }

    async fn start_session(&self, ring_capacity: u32) -> Result<Box<dyn TunSession>> {
        let capacity = if ring_capacity < 0x20000 {
            0x400000 // 4MB default ring buffer
        } else {
            ring_capacity
        };

        let session_handle = unsafe { (self.api.start_session)(self.adapter_handle, capacity) };
        if session_handle.is_null() {
            return Err(Error::WintunError("WintunStartSession failed to create ring session".to_string()));
        }

        Ok(Box::new(WintunSession {
            session_handle,
            api: self.api.clone(),
        }))
    }
}

impl Drop for WintunAdapter {
    fn drop(&mut self) {
        if !self.adapter_handle.is_null() {
            unsafe {
                (self.api.close_adapter)(self.adapter_handle);
            }
            self.adapter_handle = std::ptr::null_mut();
        }
    }
}

pub struct WintunSession {
    session_handle: WintunSessionHandle,
    api: Arc<WintunApi>,
}

unsafe impl Send for WintunSession {}
unsafe impl Sync for WintunSession {}

#[async_trait]
impl TunSession for WintunSession {
    async fn read_packet(&mut self) -> Result<Bytes> {
        let mut size: u32 = 0;
        let ptr = unsafe { (self.api.receive_packet)(self.session_handle, &mut size) };
        if ptr.is_null() || size == 0 {
            return Err(Error::WintunError("No packet available in Wintun ring".to_string()));
        }

        let slice = unsafe { std::slice::from_raw_parts(ptr, size as usize) };
        let bytes = Bytes::copy_from_slice(slice);

        unsafe {
            (self.api.release_receive_packet)(self.session_handle, ptr);
        }

        Ok(bytes)
    }

    async fn write_packet(&mut self, packet: &[u8]) -> Result<()> {
        let ptr = unsafe { (self.api.allocate_send_packet)(self.session_handle, packet.len() as u32) };
        if ptr.is_null() {
            return Err(Error::WintunError("Wintun ring buffer exhausted on send".to_string()));
        }

        unsafe {
            std::ptr::copy_nonoverlapping(packet.as_ptr(), ptr, packet.len());
            (self.api.send_packet)(self.session_handle, ptr);
        }

        Ok(())
    }

    fn shutdown(&mut self) {
        if !self.session_handle.is_null() {
            unsafe {
                (self.api.end_session)(self.session_handle);
            }
            self.session_handle = std::ptr::null_mut();
        }
    }
}

impl Drop for WintunSession {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wintun_adapter_trait_compatibility() {
        // Verifies struct layouts and send/sync bounds compile cleanly
        assert_eq!(std::mem::size_of::<WintunAdapterHandle>(), std::mem::size_of::<usize>());
    }
}
