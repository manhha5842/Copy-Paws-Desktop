// mDNS Service Discovery module for automatic LAN discovery

use anyhow::Result;
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

const SERVICE_TYPE: &str = "_copypaws._tcp.local.";
const SERVICE_NAME: &str = "CopyPaws Desktop";

pub struct MdnsService {
    daemon: Option<ServiceDaemon>,
    server_id: String,
    port: u16,
    discovered_services: Arc<RwLock<HashMap<String, DiscoveredService>>>,
}

#[derive(Debug, Clone)]
pub struct DiscoveredService {
    pub instance_name: String,
    pub hostname: String,
    pub addresses: Vec<IpAddr>,
    pub port: u16,
    pub properties: HashMap<String, String>,
}

impl MdnsService {
    /// Create a new mDNS service
    pub fn new(server_id: String, port: u16) -> Self {
        Self {
            daemon: None,
            server_id,
            port,
            discovered_services: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start advertising the service on the network
    pub fn start_advertising(&mut self) -> Result<()> {
        let daemon = ServiceDaemon::new()?;

        // Get local hostname
        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "clipboard-hub".to_string());

        // Create service info
        let service_name = format!("{} - {}", SERVICE_NAME, &self.server_id[..8]);
        
        let mut properties = HashMap::new();
        properties.insert("server_id".to_string(), self.server_id.clone());
        properties.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
        properties.insert("platform".to_string(), std::env::consts::OS.to_string());

        let service_info = ServiceInfo::new(
            SERVICE_TYPE,
            &service_name,
            &hostname,
            "",
            self.port,
            properties.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect::<Vec<_>>()[..].as_ref(),
        )?;

        daemon.register(service_info)?;
        self.daemon = Some(daemon);

        println!("mDNS: Advertising service '{}' on port {}", service_name, self.port);
        Ok(())
    }

    /// Stop advertising the service
    pub fn stop_advertising(&mut self) -> Result<()> {
        if let Some(daemon) = self.daemon.take() {
            daemon.shutdown()?;
        }
        println!("mDNS: Stopped advertising");
        Ok(())
    }

    /// Start browsing for other Clipboard Hub services
    pub async fn start_browsing(&self) -> Result<mpsc::UnboundedReceiver<DiscoveredService>> {
        let daemon = ServiceDaemon::new()?;
        let (tx, rx) = mpsc::unbounded_channel();
        let discovered_services = self.discovered_services.clone();
        let server_id = self.server_id.clone();

        let receiver = daemon.browse(SERVICE_TYPE)?;

        tokio::spawn(async move {
            loop {
                match receiver.recv() {
                    Ok(event) => {
                        match event {
                            ServiceEvent::ServiceResolved(info) => {
                                // Skip own service
                                let props: HashMap<String, String> = info.get_properties()
                                    .iter()
                                    .map(|p| (p.key().to_string(), p.val_str().to_string()))
                                    .collect();
                                
                                if let Some(id) = props.get("server_id") {
                                    if id == &server_id {
                                        continue; // Skip self
                                    }
                                }

                                let service = DiscoveredService {
                                    instance_name: info.get_fullname().to_string(),
                                    hostname: info.get_hostname().to_string(),
                                    addresses: info.get_addresses().iter().cloned().collect(),
                                    port: info.get_port(),
                                    properties: props,
                                };

                                // Store discovered service
                                {
                                    let mut services = discovered_services.write().await;
                                    services.insert(info.get_fullname().to_string(), service.clone());
                                }

                                let _ = tx.send(service);
                                println!("mDNS: Discovered service: {}", info.get_fullname());
                            }
                            ServiceEvent::ServiceRemoved(_type, name) => {
                                let mut services = discovered_services.write().await;
                                services.remove(&name);
                                println!("mDNS: Service removed: {}", name);
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        eprintln!("mDNS browse error: {}", e);
                        break;
                    }
                }
            }
        });

        Ok(rx)
    }

    /// Get list of currently discovered services
    pub async fn get_discovered_services(&self) -> Vec<DiscoveredService> {
        let services = self.discovered_services.read().await;
        services.values().cloned().collect()
    }

    /// Get service info by instance name
    pub async fn get_service(&self, instance_name: &str) -> Option<DiscoveredService> {
        let services = self.discovered_services.read().await;
        services.get(instance_name).cloned()
    }
}

impl Drop for MdnsService {
    fn drop(&mut self) {
        let _ = self.stop_advertising();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mdns_service_creation() {
        let service = MdnsService::new("test-server-id".to_string(), 8765);
        assert_eq!(service.port, 8765);
        assert_eq!(service.server_id, "test-server-id");
    }
}
