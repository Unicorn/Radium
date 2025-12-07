//! Resource lock manager for workspace coordination.

use crate::collaboration::error::{CollaborationError, Result};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tokio::time::timeout;
use tracing::{debug, warn};

/// Type of lock (read or write).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockType {
    /// Read lock (shared, multiple agents can hold).
    Read,
    /// Write lock (exclusive, only one agent can hold).
    Write,
}

/// Information about a resource lock.
#[derive(Debug, Clone)]
struct LockInfo {
    /// ID of the agent holding the lock.
    holder_agent_id: String,
    /// Type of lock.
    lock_type: LockType,
    /// Timestamp when lock was acquired (Unix epoch seconds).
    acquired_timestamp: u64,
}

/// Handle for a resource lock that automatically releases on drop.
#[derive(Debug)]
pub struct LockHandle {
    /// Path to the locked resource.
    resource_path: String,
    /// ID of the agent holding the lock.
    agent_id: String,
    /// Reference to the lock manager for cleanup.
    lock_manager: Arc<ResourceLockManager>,
}

impl Drop for LockHandle {
    fn drop(&mut self) {
        // Release the lock when handle is dropped
        let lock_manager = Arc::clone(&self.lock_manager);
        let resource_path = self.resource_path.clone();
        let agent_id = self.agent_id.clone();
        tokio::spawn(async move {
            if let Err(e) = lock_manager.release_lock_internal(&resource_path, &agent_id).await {
                warn!(
                    resource_path = %resource_path,
                    agent_id = %agent_id,
                    error = %e,
                    "Failed to release lock on drop"
                );
            }
        });
    }
}

/// Resource lock manager for coordinating workspace access.
#[derive(Debug)]
pub struct ResourceLockManager {
    /// Map of resource paths to lock information.
    locks: Arc<RwLock<HashMap<String, LockInfo>>>,
    /// Default timeout for lock acquisition (seconds).
    default_timeout_secs: u64,
}

impl ResourceLockManager {
    /// Creates a new resource lock manager.
    pub fn new() -> Self {
        Self {
            locks: Arc::new(RwLock::new(HashMap::new())),
            default_timeout_secs: 30,
        }
    }

    /// Creates a new resource lock manager with custom timeout.
    ///
    /// # Arguments
    /// * `timeout_secs` - Default timeout in seconds
    pub fn with_timeout(timeout_secs: u64) -> Self {
        Self {
            locks: Arc::new(RwLock::new(HashMap::new())),
            default_timeout_secs: timeout_secs,
        }
    }

    /// Requests a read lock on a resource.
    ///
    /// # Arguments
    /// * `agent_id` - ID of the agent requesting the lock
    /// * `resource_path` - Path to the resource
    /// * `timeout_secs` - Optional timeout (uses default if None)
    ///
    /// # Returns
    /// Returns a `LockHandle` if the lock is acquired, or an error if timeout occurs.
    pub async fn request_read_lock(
        &self,
        agent_id: &str,
        resource_path: &str,
        timeout_secs: Option<u64>,
    ) -> Result<LockHandle> {
        let timeout_duration = Duration::from_secs(timeout_secs.unwrap_or(self.default_timeout_secs));
        let start_time = SystemTime::now();

        loop {
            // Check if we've exceeded timeout
            if start_time.elapsed().unwrap() >= timeout_duration {
                let locks = self.locks.read().await;
                let holder = locks.get(resource_path).map(|info| info.holder_agent_id.clone());
                return Err(CollaborationError::LockTimeout {
                    resource_path: resource_path.to_string(),
                    holder_agent_id: holder,
                    timeout_secs: timeout_duration.as_secs(),
                });
            }

            // Try to acquire read lock
            let acquired = {
                let mut locks = self.locks.write().await;

                // Check if resource is locked
                if let Some(lock_info) = locks.get(resource_path) {
                    // If it's a write lock, we can't acquire read lock
                    if lock_info.lock_type == LockType::Write {
                        // Release write lock and try again
                        drop(locks);
                        tokio::time::sleep(Duration::from_millis(100)).await;
                        continue;
                    }
                    // If it's a read lock, we can share it (multiple read locks allowed)
                }

                // Acquire read lock
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                locks.insert(
                    resource_path.to_string(),
                    LockInfo {
                        holder_agent_id: agent_id.to_string(),
                        lock_type: LockType::Read,
                        acquired_timestamp: timestamp,
                    },
                );

                true
            };

            if acquired {
                debug!(
                    agent_id = %agent_id,
                    resource_path = %resource_path,
                    "Read lock acquired"
                );
                return Ok(LockHandle {
                    resource_path: resource_path.to_string(),
                    agent_id: agent_id.to_string(),
                    lock_manager: Arc::new(ResourceLockManager {
                        locks: Arc::clone(&self.locks),
                        default_timeout_secs: self.default_timeout_secs,
                    }),
                });
            }

            // Wait a bit before retrying
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// Requests a write lock on a resource.
    ///
    /// # Arguments
    /// * `agent_id` - ID of the agent requesting the lock
    /// * `resource_path` - Path to the resource
    /// * `timeout_secs` - Optional timeout (uses default if None)
    ///
    /// # Returns
    /// Returns a `LockHandle` if the lock is acquired, or an error if timeout occurs.
    pub async fn request_write_lock(
        &self,
        agent_id: &str,
        resource_path: &str,
        timeout_secs: Option<u64>,
    ) -> Result<LockHandle> {
        let timeout_duration = Duration::from_secs(timeout_secs.unwrap_or(self.default_timeout_secs));
        let start_time = SystemTime::now();

        loop {
            // Check if we've exceeded timeout
            if start_time.elapsed().unwrap() >= timeout_duration {
                let locks = self.locks.read().await;
                let holder = locks.get(resource_path).map(|info| info.holder_agent_id.clone());
                return Err(CollaborationError::LockTimeout {
                    resource_path: resource_path.to_string(),
                    holder_agent_id: holder,
                    timeout_secs: timeout_duration.as_secs(),
                });
            }

            // Try to acquire write lock
            let acquired = {
                let mut locks = self.locks.write().await;

                // Check if resource is already locked
                if locks.contains_key(resource_path) {
                    // Resource is locked, can't acquire write lock
                    drop(locks);
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    continue;
                }

                // Acquire write lock
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                locks.insert(
                    resource_path.to_string(),
                    LockInfo {
                        holder_agent_id: agent_id.to_string(),
                        lock_type: LockType::Write,
                        acquired_timestamp: timestamp,
                    },
                );

                true
            };

            if acquired {
                debug!(
                    agent_id = %agent_id,
                    resource_path = %resource_path,
                    "Write lock acquired"
                );
                return Ok(LockHandle {
                    resource_path: resource_path.to_string(),
                    agent_id: agent_id.to_string(),
                    lock_manager: Arc::new(ResourceLockManager {
                        locks: Arc::clone(&self.locks),
                        default_timeout_secs: self.default_timeout_secs,
                    }),
                });
            }

            // Wait a bit before retrying
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// Releases a lock on a resource.
    ///
    /// # Arguments
    /// * `handle` - The lock handle to release
    pub async fn release_lock(&self, handle: LockHandle) -> Result<()> {
        self.release_lock_internal(&handle.resource_path, &handle.agent_id).await
    }

    /// Internal method to release a lock.
    async fn release_lock_internal(&self, resource_path: &str, agent_id: &str) -> Result<()> {
        let mut locks = self.locks.write().await;

        if let Some(lock_info) = locks.get(resource_path) {
            if lock_info.holder_agent_id == agent_id {
                locks.remove(resource_path);
                debug!(
                    agent_id = %agent_id,
                    resource_path = %resource_path,
                    "Lock released"
                );
                Ok(())
            } else {
                Err(CollaborationError::LockTimeout {
                    resource_path: resource_path.to_string(),
                    holder_agent_id: Some(lock_info.holder_agent_id.clone()),
                    timeout_secs: 0,
                })
            }
        } else {
            // Lock doesn't exist, that's okay
            Ok(())
        }
    }

    /// Gets information about a lock on a resource.
    ///
    /// # Arguments
    /// * `resource_path` - Path to the resource
    ///
    /// # Returns
    /// Returns lock information if the resource is locked, None otherwise.
    pub async fn get_lock_info(&self, resource_path: &str) -> Option<(String, LockType, u64)> {
        let locks = self.locks.read().await;
        locks.get(resource_path).map(|info| {
            (
                info.holder_agent_id.clone(),
                info.lock_type,
                info.acquired_timestamp,
            )
        })
    }

    /// Starts a background task to clean up expired locks.
    ///
    /// # Arguments
    /// * `max_age_secs` - Maximum age of locks before they're considered expired
    pub fn start_cleanup_task(self: Arc<Self>, max_age_secs: u64) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60)); // Check every minute

            loop {
                interval.tick().await;

                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                let mut locks = self.locks.write().await;
                let expired_paths: Vec<String> = locks
                    .iter()
                    .filter(|(_, info)| now - info.acquired_timestamp > max_age_secs)
                    .map(|(path, _)| path.clone())
                    .collect();

                for path in expired_paths {
                    locks.remove(&path);
                    warn!(resource_path = %path, "Removed expired lock");
                }
            }
        });
    }
}

impl Default for ResourceLockManager {
    fn default() -> Self {
        Self::new()
    }
}

