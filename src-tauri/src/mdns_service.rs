//! mDNS/DNS-SD service registration for local network discovery.
//!
//! This module handles advertising the Sermon Helper service on the local network
//! using mDNS/DNS-SD, allowing mobile apps to discover and connect to the desktop app.

use mdns_sd::{ServiceDaemon, ServiceInfo};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Service type for Sermon Helper (RFC 6763 compliant)
pub const SERVICE_TYPE: &str = "_sermon-helper._tcp.local.";

/// mDNS service registration handle
pub struct MdnsService {
    daemon: ServiceDaemon,
    service_fullname: String,
}

impl MdnsService {
    /// Register a new mDNS service
    pub fn register(
        instance_name: &str,
        port: u16,
        properties: HashMap<String, String>,
    ) -> Result<Self, String> {
        // Create the mDNS daemon
        let daemon = ServiceDaemon::new()
            .map_err(|e| format!("Failed to create mDNS daemon: {}", e))?;

        // Get local IP addresses for registration
        let host_ipv4 = local_ip_address::local_ip()
            .map_err(|e| format!("Failed to get local IP: {}", e))?;

        let hostname = hostname::get()
            .map_err(|e| format!("Failed to get hostname: {}", e))?
            .to_string_lossy()
            .to_string();

        // Build the service info
        let service_info = ServiceInfo::new(
            SERVICE_TYPE,
            instance_name,
            &format!("{}.local.", hostname),
            host_ipv4,
            port,
            properties,
        )
        .map_err(|e| format!("Failed to create service info: {}", e))?;

        let service_fullname = service_info.get_fullname().to_string();

        // Register the service
        daemon
            .register(service_info)
            .map_err(|e| format!("Failed to register mDNS service: {}", e))?;

        log::info!(
            "mDNS service registered: {} on port {} ({})",
            instance_name,
            port,
            host_ipv4
        );

        Ok(Self {
            daemon,
            service_fullname,
        })
    }

    /// Unregister the service
    pub fn unregister(&self) -> Result<(), String> {
        self.daemon
            .unregister(&self.service_fullname)
            .map_err(|e| format!("Failed to unregister mDNS service: {}", e))?;

        log::info!("mDNS service unregistered: {}", self.service_fullname);
        Ok(())
    }

    /// Get the full service name
    pub fn fullname(&self) -> &str {
        &self.service_fullname
    }
}

impl Drop for MdnsService {
    fn drop(&mut self) {
        if let Err(e) = self.unregister() {
            log::warn!("Failed to unregister mDNS service on drop: {}", e);
        }
        // Shutdown the daemon
        if let Err(e) = self.daemon.shutdown() {
            log::warn!("Failed to shutdown mDNS daemon: {}", e);
        }
    }
}

/// Shared mDNS service state
pub type SharedMdnsService = Arc<Mutex<Option<MdnsService>>>;

/// Create a new shared mDNS service state
pub fn create_shared_mdns_service() -> SharedMdnsService {
    Arc::new(Mutex::new(None))
}
