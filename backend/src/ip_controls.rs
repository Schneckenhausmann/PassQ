//! IP-based controls module for geolocation alerts and IP whitelisting

use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::collections::HashSet;
use ipnetwork::IpNetwork;
use log;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeolocationInfo {
    pub country: String,
    pub region: String,
    pub city: String,
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: String,
    pub isp: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct IpApiResponse {
    status: String,
    country: Option<String>,
    #[serde(rename = "regionName")]
    region_name: Option<String>,
    city: Option<String>,
    lat: Option<f64>,
    lon: Option<f64>,
    timezone: Option<String>,
    isp: Option<String>,
}

#[derive(Debug, Clone)]
pub struct IpWhitelist {
    allowed_ips: HashSet<IpAddr>,
    allowed_networks: Vec<IpNetwork>,
}

impl IpWhitelist {
    pub fn new() -> Self {
        Self {
            allowed_ips: HashSet::new(),
            allowed_networks: Vec::new(),
        }
    }

    pub fn add_ip(&mut self, ip: IpAddr) {
        self.allowed_ips.insert(ip);
    }

    pub fn add_network(&mut self, network: IpNetwork) {
        self.allowed_networks.push(network);
    }

    pub fn is_allowed(&self, ip: &IpAddr) -> bool {
        // Check if IP is directly whitelisted
        if self.allowed_ips.contains(ip) {
            return true;
        }

        // Check if IP is in any whitelisted network
        for network in &self.allowed_networks {
            if network.contains(*ip) {
                return true;
            }
        }

        false
    }

    pub fn from_env() -> Self {
        let mut whitelist = Self::new();
        
        // Load from environment variable if set
        if let Ok(whitelist_str) = std::env::var("IP_WHITELIST") {
            for entry in whitelist_str.split(',') {
                let entry = entry.trim();
                if entry.is_empty() {
                    continue;
                }

                // Try to parse as IP address first
                if let Ok(ip) = entry.parse::<IpAddr>() {
                    whitelist.add_ip(ip);
                    log::info!("Added IP to whitelist: {}", ip);
                } else if let Ok(network) = entry.parse::<IpNetwork>() {
                    whitelist.add_network(network);
                    log::info!("Added network to whitelist: {}", network);
                } else {
                    log::warn!("Invalid IP or network in whitelist: {}", entry);
                }
            }
        }

        whitelist
    }
}

/// Get geolocation information for an IP address using ip-api.com
#[allow(dead_code)]
pub async fn get_geolocation(ip: &IpAddr) -> Result<GeolocationInfo, String> {
    // Skip geolocation for local/private IPs
    if is_private_ip(ip) {
        return Ok(GeolocationInfo {
            country: "Local".to_string(),
            region: "Private Network".to_string(),
            city: "Local".to_string(),
            latitude: 0.0,
            longitude: 0.0,
            timezone: "UTC".to_string(),
            isp: "Local Network".to_string(),
        });
    }

    let client = reqwest::Client::new();
    let url = format!("http://ip-api.com/json/{}", ip);
    
    match client.get(&url).send().await {
        Ok(response) => {
            match response.json::<IpApiResponse>().await {
                Ok(data) => {
                    if data.status == "success" {
                        Ok(GeolocationInfo {
                            country: data.country.unwrap_or_else(|| "Unknown".to_string()),
                            region: data.region_name.unwrap_or_else(|| "Unknown".to_string()),
                            city: data.city.unwrap_or_else(|| "Unknown".to_string()),
                            latitude: data.lat.unwrap_or(0.0),
                            longitude: data.lon.unwrap_or(0.0),
                            timezone: data.timezone.unwrap_or_else(|| "UTC".to_string()),
                            isp: data.isp.unwrap_or_else(|| "Unknown".to_string()),
                        })
                    } else {
                        Err("Geolocation API returned failure status".to_string())
                    }
                }
                Err(e) => {
                    log::error!("Failed to parse geolocation response: {}", e);
                    Err(format!("Failed to parse geolocation data: {}", e))
                }
            }
        }
        Err(e) => {
            log::error!("Failed to fetch geolocation for IP {}: {}", ip, e);
            Err(format!("Failed to fetch geolocation: {}", e))
        }
    }
}

/// Check if an IP address is private/local
#[allow(dead_code)]
fn is_private_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(ipv4) => {
            ipv4.is_private() || ipv4.is_loopback() || ipv4.is_link_local()
        }
        IpAddr::V6(ipv6) => {
            ipv6.is_loopback() || ipv6.segments()[0] == 0xfe80
        }
    }
}

/// Extract client IP from request headers
pub fn extract_client_ip(req: &actix_web::HttpRequest) -> Option<IpAddr> {
    // Check X-Forwarded-For header first (for reverse proxies)
    if let Some(forwarded_for) = req.headers().get("X-Forwarded-For") {
        if let Ok(forwarded_str) = forwarded_for.to_str() {
            // Take the first IP in the chain
            if let Some(first_ip) = forwarded_str.split(',').next() {
                if let Ok(ip) = first_ip.trim().parse::<IpAddr>() {
                    return Some(ip);
                }
            }
        }
    }

    // Check X-Real-IP header
    if let Some(real_ip) = req.headers().get("X-Real-IP") {
        if let Ok(ip_str) = real_ip.to_str() {
            if let Ok(ip) = ip_str.parse::<IpAddr>() {
                return Some(ip);
            }
        }
    }

    // Fall back to connection info
    if let Some(peer_addr) = req.peer_addr() {
        return Some(peer_addr.ip());
    }

    None
}

/// Check if login is from a suspicious location
#[allow(dead_code)]
pub fn is_suspicious_location(current: &GeolocationInfo, previous: &GeolocationInfo) -> bool {
    // Consider it suspicious if country changed
    if current.country != previous.country {
        return true;
    }

    // Calculate approximate distance (simple Euclidean distance)
    let lat_diff = current.latitude - previous.latitude;
    let lon_diff = current.longitude - previous.longitude;
    let distance = (lat_diff * lat_diff + lon_diff * lon_diff).sqrt();
    
    // Consider suspicious if distance is more than ~500km (roughly 5 degrees)
    distance > 5.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_ip_whitelist() {
        let mut whitelist = IpWhitelist::new();
        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        
        assert!(!whitelist.is_allowed(&ip));
        
        whitelist.add_ip(ip);
        assert!(whitelist.is_allowed(&ip));
    }

    #[test]
    fn test_network_whitelist() {
        let mut whitelist = IpWhitelist::new();
        let network = "192.168.0.0/16".parse::<IpNetwork>().unwrap();
        let ip_in_network = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100));
        let ip_outside_network = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
        
        whitelist.add_network(network);
        
        assert!(whitelist.is_allowed(&ip_in_network));
        assert!(!whitelist.is_allowed(&ip_outside_network));
    }

    #[test]
    fn test_private_ip_detection() {
        assert!(is_private_ip(&IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))));
        assert!(is_private_ip(&IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))));
        assert!(is_private_ip(&IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))));
        assert!(!is_private_ip(&IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))));
    }
}