//! Message bus for agent-to-agent communication.

use crate::collaboration::error::{CollaborationError, Result};
use crate::storage::database::Database;
use crate::storage::error::StorageError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex as StdMutex};
use tokio::sync::mpsc;
use tracing::{debug, error, warn};
use uuid::Uuid;

/// Types of messages that can be sent between agents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    /// Request for a task to be performed.
    TaskRequest,
    /// Response to a task request.
    TaskResponse,
    /// Status update from an agent.
    StatusUpdate,
    /// Resource lock request.
    ResourceLock,
    /// Resource lock release.
    ResourceRelease,
}

impl MessageType {
    /// Converts a string to a MessageType.
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "TaskRequest" => Ok(MessageType::TaskRequest),
            "TaskResponse" => Ok(MessageType::TaskResponse),
            "StatusUpdate" => Ok(MessageType::StatusUpdate),
            "ResourceLock" => Ok(MessageType::ResourceLock),
            "ResourceRelease" => Ok(MessageType::ResourceRelease),
            _ => Err(CollaborationError::InvalidMessageType {
                message_type: s.to_string(),
            }),
        }
    }

    /// Converts a MessageType to a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            MessageType::TaskRequest => "TaskRequest",
            MessageType::TaskResponse => "TaskResponse",
            MessageType::StatusUpdate => "StatusUpdate",
            MessageType::ResourceLock => "ResourceLock",
            MessageType::ResourceRelease => "ResourceRelease",
        }
    }
}

/// A message sent between agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    /// Unique message identifier.
    pub id: String,
    /// ID of the agent that sent the message.
    pub sender_id: String,
    /// ID of the recipient agent (None for broadcast messages).
    pub recipient_id: Option<String>,
    /// Type of message.
    pub message_type: MessageType,
    /// Message payload as JSON.
    pub payload_json: String,
    /// Unix epoch timestamp when message was sent.
    pub timestamp: i64,
    /// Whether the message has been delivered.
    pub delivered: bool,
}

/// Repository trait for message persistence.
pub trait MessageRepository: Send + Sync {
    /// Stores a message in the database.
    fn store_message(&self, message: &AgentMessage) -> Result<()>;

    /// Retrieves messages for an agent.
    fn get_messages(
        &self,
        agent_id: &str,
        undelivered_only: bool,
    ) -> Result<Vec<AgentMessage>>;

    /// Marks a message as delivered.
    fn mark_delivered(&self, message_id: &str) -> Result<()>;
}

/// Database-backed message repository.
pub struct DatabaseMessageRepository {
    db: Arc<StdMutex<Database>>,
}

impl DatabaseMessageRepository {
    /// Creates a new database message repository.
    pub fn new(db: Arc<StdMutex<Database>>) -> Self {
        Self { db }
    }
}

impl MessageRepository for DatabaseMessageRepository {
    fn store_message(&self, message: &AgentMessage) -> Result<()> {
        let message = message.clone();
        let mut db = self.db.lock().map_err(|e| {
            CollaborationError::DatabaseError(StorageError::InvalidData(format!(
                "Database lock error: {}",
                e
            )))
        })?;
        let conn = db.conn_mut();

        conn.execute(
            "INSERT INTO agent_messages (id, sender_id, recipient_id, message_type, payload_json, timestamp, delivered) VALUES (?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                message.id,
                message.sender_id,
                message.recipient_id,
                message.message_type.as_str(),
                message.payload_json,
                message.timestamp,
                if message.delivered { 1 } else { 0 }
            ],
        )
        .map_err(|e| CollaborationError::DatabaseError(StorageError::Connection(e)))?;

        Ok(())
    }

    fn get_messages(
        &self,
        agent_id: &str,
        undelivered_only: bool,
    ) -> Result<Vec<AgentMessage>> {
        let agent_id = agent_id.to_string();
        let db = self.db.lock().map_err(|e| {
            CollaborationError::DatabaseError(StorageError::InvalidData(format!(
                "Database lock error: {}",
                e
            )))
        })?;
        let conn = db.conn();

        let query = if undelivered_only {
            "SELECT id, sender_id, recipient_id, message_type, payload_json, timestamp, delivered FROM agent_messages WHERE (recipient_id = ? OR recipient_id IS NULL) AND delivered = 0 ORDER BY timestamp ASC"
        } else {
            "SELECT id, sender_id, recipient_id, message_type, payload_json, timestamp, delivered FROM agent_messages WHERE recipient_id = ? OR recipient_id IS NULL ORDER BY timestamp ASC"
        };

        let mut stmt = conn.prepare(query).map_err(|e| {
            CollaborationError::DatabaseError(StorageError::Connection(e))
        })?;

        let messages = stmt
            .query_map([agent_id], |row| {
                let message_type_str: String = row.get(3)?;
                let message_type = MessageType::from_str(&message_type_str)
                    .map_err(|_| rusqlite::Error::InvalidColumnType(3, "message_type".to_string(), rusqlite::types::Type::Text))?;

                Ok(AgentMessage {
                    id: row.get(0)?,
                    sender_id: row.get(1)?,
                    recipient_id: row.get(2)?,
                    message_type,
                    payload_json: row.get(4)?,
                    timestamp: row.get(5)?,
                    delivered: row.get::<_, i64>(6)? != 0,
                })
            })
            .map_err(|e| CollaborationError::DatabaseError(StorageError::Connection(e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| CollaborationError::DatabaseError(StorageError::Connection(e)))?;

        Ok(messages)
    }

    fn mark_delivered(&self, message_id: &str) -> Result<()> {
        let message_id = message_id.to_string();
        let mut db = self.db.lock().map_err(|e| {
            CollaborationError::DatabaseError(StorageError::InvalidData(format!(
                "Database lock error: {}",
                e
            )))
        })?;
        let conn = db.conn_mut();

        conn.execute(
            "UPDATE agent_messages SET delivered = 1 WHERE id = ?",
            [message_id],
        )
        .map_err(|e| CollaborationError::DatabaseError(StorageError::Connection(e)))?;

        Ok(())
    }
}

/// Message bus for agent-to-agent communication.
pub struct MessageBus {
    /// Database repository for message persistence.
    repository: Arc<dyn MessageRepository>,
    /// Channels for each registered agent (agent_id -> sender).
    channels: Arc<StdMutex<HashMap<String, mpsc::UnboundedSender<AgentMessage>>>>,
}

impl MessageBus {
    /// Creates a new message bus.
    pub fn new(db: Arc<StdMutex<Database>>) -> Self {
        let repository: Arc<dyn MessageRepository> = Arc::new(DatabaseMessageRepository::new(db));
        Self {
            repository,
            channels: Arc::new(StdMutex::new(HashMap::new())),
        }
    }

    /// Registers an agent with the message bus.
    ///
    /// # Arguments
    /// * `agent_id` - ID of the agent to register
    ///
    /// # Returns
    /// Returns the receiver channel for the agent to listen for messages.
    pub async fn register_agent(
        &self,
        agent_id: String,
    ) -> mpsc::UnboundedReceiver<AgentMessage> {
        let (tx, rx) = mpsc::unbounded_channel();
        let mut channels = self.channels.lock().unwrap();
        channels.insert(agent_id.clone(), tx);
        debug!(agent_id = %agent_id, "Registered agent with message bus");
        rx
    }

    /// Unregisters an agent from the message bus.
    ///
    /// # Arguments
    /// * `agent_id` - ID of the agent to unregister
    pub async fn unregister_agent(&self, agent_id: &str) {
        let mut channels = self.channels.lock().unwrap();
        channels.remove(agent_id);
        debug!(agent_id = %agent_id, "Unregistered agent from message bus");
    }

    /// Sends a message to a specific agent.
    ///
    /// # Arguments
    /// * `sender_id` - ID of the sending agent
    /// * `recipient_id` - ID of the recipient agent
    /// * `message_type` - Type of message
    /// * `payload` - Message payload as JSON value
    ///
    /// # Returns
    /// Returns the message ID if successful.
    pub async fn send_message(
        &self,
        sender_id: &str,
        recipient_id: &str,
        message_type: MessageType,
        payload: serde_json::Value,
    ) -> Result<String> {
        let message_id = Uuid::new_v4().to_string();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let payload_json = serde_json::to_string(&payload).map_err(|e| {
            CollaborationError::DatabaseError(StorageError::Serialization(e))
        })?;

        let message = AgentMessage {
            id: message_id.clone(),
            sender_id: sender_id.to_string(),
            recipient_id: Some(recipient_id.to_string()),
            message_type,
            payload_json,
            timestamp,
            delivered: false,
        };

        // Store in database first
        self.repository.store_message(&message)?;

        // Try to deliver via channel
        let channels = self.channels.lock().unwrap();
        if let Some(sender) = channels.get(recipient_id) {
            if sender.send(message.clone()).is_err() {
                warn!(recipient_id = %recipient_id, "Failed to send message via channel (agent may have disconnected)");
            } else {
                // Mark as delivered
                self.repository.mark_delivered(&message_id)?;
                debug!(
                    sender_id = %sender_id,
                    recipient_id = %recipient_id,
                    message_type = ?message_type,
                    "Message sent and delivered"
                );
            }
        } else {
            debug!(
                recipient_id = %recipient_id,
                "Recipient not registered, message stored for later delivery"
            );
        }

        Ok(message_id)
    }

    /// Broadcasts a message to all registered agents.
    ///
    /// # Arguments
    /// * `sender_id` - ID of the sending agent
    /// * `message_type` - Type of message
    /// * `payload` - Message payload as JSON value
    ///
    /// # Returns
    /// Returns the message ID if successful.
    pub async fn broadcast_message(
        &self,
        sender_id: &str,
        message_type: MessageType,
        payload: serde_json::Value,
    ) -> Result<String> {
        let message_id = Uuid::new_v4().to_string();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let payload_json = serde_json::to_string(&payload).map_err(|e| {
            CollaborationError::DatabaseError(StorageError::Serialization(e))
        })?;

        let message = AgentMessage {
            id: message_id.clone(),
            sender_id: sender_id.to_string(),
            recipient_id: None, // None indicates broadcast
            message_type,
            payload_json,
            timestamp,
            delivered: false,
        };

        // Store in database first
        self.repository.store_message(&message)?;

        // Broadcast to all registered agents
        let channels = self.channels.lock().unwrap();
        let mut delivered_count = 0;
        for (agent_id, sender) in channels.iter() {
            if *agent_id != sender_id {
                // Don't send to sender
                if sender.send(message.clone()).is_ok() {
                    delivered_count += 1;
                }
            }
        }

        if delivered_count > 0 {
            // Mark as delivered (broadcast messages are considered delivered if at least one recipient got it)
            self.repository.mark_delivered(&message_id)?;
            debug!(
                sender_id = %sender_id,
                recipients = delivered_count,
                message_type = ?message_type,
                "Broadcast message sent"
            );
        }

        Ok(message_id)
    }

    /// Retrieves messages for an agent.
    ///
    /// # Arguments
    /// * `agent_id` - ID of the agent
    /// * `undelivered_only` - If true, only return undelivered messages
    ///
    /// # Returns
    /// Returns a vector of messages.
    pub async fn get_messages(
        &self,
        agent_id: &str,
        undelivered_only: bool,
    ) -> Result<Vec<AgentMessage>> {
        self.repository.get_messages(agent_id, undelivered_only)
    }

    /// Marks a message as delivered.
    ///
    /// # Arguments
    /// * `message_id` - ID of the message to mark as delivered
    pub async fn mark_delivered(&self, message_id: &str) -> Result<()> {
        self.repository.mark_delivered(message_id)
    }
}

