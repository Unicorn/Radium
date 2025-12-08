//! Network-level policy interception for blocking operations at the network layer.

use super::types::{PolicyError, PolicyResult};
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Network pattern for matching domains, IPs, or IP ranges.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkPattern {
    /// Domain pattern (e.g., "*.malicious.com").
    Domain(String),
    /// Single IP address.
    Ip(IpAddr),
    /// IP range in CIDR notation (e.g., "10.0.0.0/8").
    IpRange(String),
}

/// Trait for platform-specific network interception implementations.
pub trait NetworkInterceptor: Send + Sync {
    /// Checks if a domain should be blocked.
    ///
    /// # Arguments
    /// * `domain` - Domain name to check
    ///
    /// # Returns
    /// `true` if the domain should be blocked, `false` otherwise
    fn should_block_domain(&self, domain: &str) -> bool;

    /// Checks if an IP address should be blocked.
    ///
    /// # Arguments
    /// * `ip` - IP address to check
    ///
    /// # Returns
    /// `true` if the IP should be blocked, `false` otherwise
    fn should_block_ip(&self, ip: IpAddr) -> bool;

    /// Blocks a domain (prevents DNS resolution).
    ///
    /// # Arguments
    /// * `domain` - Domain to block
    fn block_domain(&mut self, domain: &str) -> PolicyResult<()>;

    /// Blocks an IP address or range.
    ///
    /// # Arguments
    /// * `pattern` - Network pattern to block
    fn block_pattern(&mut self, pattern: &NetworkPattern) -> PolicyResult<()>;
}

/// Default network interceptor (no-op implementation).
///
/// This is a placeholder that doesn't actually block anything.
/// Platform-specific implementations should be used in production.
pub struct DefaultNetworkInterceptor {
    /// Blocked domains.
    blocked_domains: Vec<String>,
    /// Blocked IP patterns.
    blocked_patterns: Vec<NetworkPattern>,
}

impl DefaultNetworkInterceptor {
    /// Creates a new default network interceptor.
    pub fn new() -> Self {
        Self {
            blocked_domains: Vec::new(),
            blocked_patterns: Vec::new(),
        }
    }
}

impl Default for DefaultNetworkInterceptor {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkInterceptor for DefaultNetworkInterceptor {
    fn should_block_domain(&self, domain: &str) -> bool {
        // Check exact matches
        if self.blocked_domains.contains(&domain.to_string()) {
            return true;
        }

        // Check wildcard patterns
        for pattern in &self.blocked_domains {
            if Self::matches_domain_pattern(domain, pattern) {
                return true;
            }
        }

        false
    }

    fn should_block_ip(&self, ip: IpAddr) -> bool {
        for pattern in &self.blocked_patterns {
            match pattern {
                NetworkPattern::Ip(blocked_ip) => {
                    if ip == *blocked_ip {
                        return true;
                    }
                }
                NetworkPattern::IpRange(range) => {
                    if Self::ip_in_range(ip, range) {
                        return true;
                    }
                }
                NetworkPattern::Domain(_) => {} // Domain patterns don't apply to IPs
            }
        }
        false
    }

    fn block_domain(&mut self, domain: &str) -> PolicyResult<()> {
        if !self.blocked_domains.contains(&domain.to_string()) {
            self.blocked_domains.push(domain.to_string());
        }
        Ok(())
    }

    fn block_pattern(&mut self, pattern: &NetworkPattern) -> PolicyResult<()> {
        match pattern {
            NetworkPattern::Domain(domain) => self.block_domain(domain),
            NetworkPattern::Ip(_) | NetworkPattern::IpRange(_) => {
                if !self.blocked_patterns.contains(pattern) {
                    self.blocked_patterns.push(pattern.clone());
                }
                Ok(())
            }
        }
    }
}

impl DefaultNetworkInterceptor {
    /// Checks if a domain matches a pattern (supports wildcards).
    fn matches_domain_pattern(domain: &str, pattern: &str) -> bool {
        if pattern == domain {
            return true;
        }

        // Handle wildcard patterns like "*.example.com"
        if pattern.starts_with("*.") {
            let suffix = &pattern[2..];
            return domain.ends_with(suffix) || domain == &suffix[..suffix.len().saturating_sub(1)];
        }

        false
    }

    /// Checks if an IP address is within a CIDR range.
    fn ip_in_range(ip: IpAddr, cidr: &str) -> bool {
        // Simple CIDR matching (basic implementation)
        // For production, use a proper CIDR library like ipnet
        if let Some((network_str, prefix_len_str)) = cidr.split_once('/') {
            if let (Ok(network_ip), Ok(prefix_len)) = (
                network_str.parse::<IpAddr>(),
                prefix_len_str.parse::<u8>(),
            ) {
                return Self::ip_in_cidr(ip, network_ip, prefix_len);
            }
        }
        false
    }

    /// Checks if an IP is within a CIDR network.
    fn ip_in_cidr(ip: IpAddr, network: IpAddr, prefix_len: u8) -> bool {
        match (ip, network) {
            (IpAddr::V4(ip), IpAddr::V4(net)) => {
                let ip_u32: u32 = ip.into();
                let net_u32: u32 = net.into();
                let mask = if prefix_len == 0 {
                    0
                } else {
                    !((1u32 << (32 - prefix_len)) - 1)
                };
                (ip_u32 & mask) == (net_u32 & mask)
            }
            (IpAddr::V6(_), IpAddr::V6(_)) => {
                // IPv6 CIDR matching would require more complex logic
                // For now, just return false
                false
            }
            _ => false,
        }
    }
}

/// Network interceptor manager that coordinates policy-based blocking.
pub struct NetworkInterceptorManager {
    /// The network interceptor implementation.
    interceptor: Arc<RwLock<dyn NetworkInterceptor>>,
}

impl NetworkInterceptorManager {
    /// Creates a new network interceptor manager.
    ///
    /// # Arguments
    /// * `interceptor` - Network interceptor implementation
    pub fn new(interceptor: Arc<RwLock<dyn NetworkInterceptor>>) -> Self {
        Self { interceptor }
    }

    /// Creates a default network interceptor manager (no-op).
    pub fn default() -> Self {
        Self::new(Arc::new(RwLock::new(DefaultNetworkInterceptor::new())))
    }

    /// Checks if a domain should be blocked.
    pub async fn should_block_domain(&self, domain: &str) -> bool {
        let interceptor = self.interceptor.read().await;
        interceptor.should_block_domain(domain)
    }

    /// Checks if an IP address should be blocked.
    pub async fn should_block_ip(&self, ip: IpAddr) -> bool {
        let interceptor = self.interceptor.read().await;
        interceptor.should_block_ip(ip)
    }

    /// Blocks a network pattern based on policy rules.
    pub async fn block_pattern(&self, pattern: &NetworkPattern) -> PolicyResult<()> {
        let mut interceptor = self.interceptor.write().await;
        interceptor.block_pattern(pattern)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_domain_pattern_matching() {
        let mut interceptor = DefaultNetworkInterceptor::new();
        interceptor.block_domain("*.malicious.com").unwrap();
        interceptor.block_domain("evil.com").unwrap();

        assert!(interceptor.should_block_domain("evil.malicious.com"));
        assert!(interceptor.should_block_domain("evil.com"));
        assert!(!interceptor.should_block_domain("safe.com"));
    }

    #[test]
    fn test_ip_range_blocking() {
        let mut interceptor = DefaultNetworkInterceptor::new();
        interceptor
            .block_pattern(&NetworkPattern::IpRange("10.0.0.0/8".to_string()))
            .unwrap();

        let blocked_ip = IpAddr::V4(Ipv4Addr::new(10, 1, 2, 3));
        let allowed_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));

        assert!(interceptor.should_block_ip(blocked_ip));
        assert!(!interceptor.should_block_ip(allowed_ip));
    }

    #[test]
    fn test_single_ip_blocking() {
        let mut interceptor = DefaultNetworkInterceptor::new();
        let blocked_ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        interceptor.block_pattern(&NetworkPattern::Ip(blocked_ip)).unwrap();

        assert!(interceptor.should_block_ip(blocked_ip));
        assert!(!interceptor.should_block_ip(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2))));
    }

    #[tokio::test]
    async fn test_network_interceptor_manager() {
        let manager = NetworkInterceptorManager::default();
        assert!(!manager.should_block_domain("example.com").await);
    }
}

