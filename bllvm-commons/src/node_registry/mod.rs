//! External Node Registry
//!
//! Simple registry system for nodes/miners to register for fee forwarding attribution.
//! Nodes register with their Bitcoin addresses, and the system maps transactions to nodes.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tracing::info;

pub mod api;

/// Node type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    Miner,
    Node,
    Pool,
    Exchange,
    Other,
}

impl NodeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeType::Miner => "miner",
            NodeType::Node => "node",
            NodeType::Pool => "pool",
            NodeType::Exchange => "exchange",
            NodeType::Other => "other",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "miner" => NodeType::Miner,
            "node" => NodeType::Node,
            "pool" => NodeType::Pool,
            "exchange" => NodeType::Exchange,
            _ => NodeType::Other,
        }
    }
}

/// Node registration record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeRegistration {
    pub node_id: String,
    pub node_name: String,
    pub node_type: NodeType,
    pub bitcoin_addresses: Vec<String>,
    pub registered_at: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub active: bool,
    pub metadata: Option<serde_json::Value>,
}

/// Node registry manager
pub struct NodeRegistry {
    pool: SqlitePool,
}

impl NodeRegistry {
    /// Create a new node registry
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Register a new node
    pub async fn register_node(
        &self,
        node_id: &str,
        node_name: &str,
        node_type: NodeType,
        bitcoin_addresses: Vec<String>,
        metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        // Insert or update node registration
        sqlx::query(
            r#"
            INSERT INTO node_registry
            (node_id, node_name, node_type, bitcoin_addresses, metadata, active, last_seen)
            VALUES (?, ?, ?, ?, ?, TRUE, CURRENT_TIMESTAMP)
            ON CONFLICT(node_id) DO UPDATE SET
                node_name = excluded.node_name,
                node_type = excluded.node_type,
                bitcoin_addresses = excluded.bitcoin_addresses,
                metadata = excluded.metadata,
                active = TRUE,
                last_seen = CURRENT_TIMESTAMP
            "#,
        )
        .bind(node_id)
        .bind(node_name)
        .bind(node_type.as_str())
        .bind(serde_json::to_string(&bitcoin_addresses)?)
        .bind(
            metadata
                .as_ref()
                .map(|m| serde_json::to_string(m).unwrap_or_default()),
        )
        .execute(&self.pool)
        .await?;

        // Update address mappings
        self.update_address_mappings(node_id, &bitcoin_addresses)
            .await?;

        info!(
            "Registered node: {} ({}) with {} addresses",
            node_id,
            node_name,
            bitcoin_addresses.len()
        );
        Ok(())
    }

    /// Update address mappings for a node
    async fn update_address_mappings(&self, node_id: &str, addresses: &[String]) -> Result<()> {
        // Delete old mappings
        sqlx::query("DELETE FROM address_to_node WHERE node_id = ?")
            .bind(node_id)
            .execute(&self.pool)
            .await?;

        // Insert new mappings
        for address in addresses {
            sqlx::query("INSERT OR REPLACE INTO address_to_node (address, node_id) VALUES (?, ?)")
                .bind(address)
                .bind(node_id)
                .execute(&self.pool)
                .await?;
        }

        Ok(())
    }

    /// Get node ID for a Bitcoin address
    pub async fn get_node_for_address(&self, address: &str) -> Result<Option<String>> {
        let node_id: Option<String> =
            sqlx::query_scalar("SELECT node_id FROM address_to_node WHERE address = ?")
                .bind(address)
                .fetch_optional(&self.pool)
                .await?;

        Ok(node_id)
    }

    /// Get node registration by ID
    pub async fn get_node(&self, node_id: &str) -> Result<Option<NodeRegistration>> {
        #[derive(sqlx::FromRow)]
        struct NodeRow {
            node_id: String,
            node_name: String,
            node_type: String,
            bitcoin_addresses: String,
            registered_at: DateTime<Utc>,
            last_seen: DateTime<Utc>,
            active: bool,
            metadata: Option<String>,
        }

        let row: Option<NodeRow> = sqlx::query_as::<_, NodeRow>(
            "SELECT node_id, node_name, node_type, bitcoin_addresses, registered_at, last_seen, active, metadata FROM node_registry WHERE node_id = ?"
        )
        .bind(node_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let addresses: Vec<String> = serde_json::from_str(&row.bitcoin_addresses)?;
            let metadata = row
                .metadata
                .as_ref()
                .and_then(|m| serde_json::from_str(m).ok());

            Ok(Some(NodeRegistration {
                node_id: row.node_id,
                node_name: row.node_name,
                node_type: NodeType::from_str(&row.node_type),
                bitcoin_addresses: addresses,
                registered_at: row.registered_at,
                last_seen: row.last_seen,
                active: row.active,
                metadata,
            }))
        } else {
            Ok(None)
        }
    }

    /// Update last seen timestamp for a node
    pub async fn update_last_seen(&self, node_id: &str) -> Result<()> {
        sqlx::query("UPDATE node_registry SET last_seen = CURRENT_TIMESTAMP WHERE node_id = ?")
            .bind(node_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Deactivate a node
    pub async fn deactivate_node(&self, node_id: &str) -> Result<()> {
        sqlx::query("UPDATE node_registry SET active = FALSE WHERE node_id = ?")
            .bind(node_id)
            .execute(&self.pool)
            .await?;
        info!("Deactivated node: {}", node_id);
        Ok(())
    }

    /// Get all active nodes
    pub async fn get_active_nodes(&self) -> Result<Vec<NodeRegistration>> {
        #[derive(sqlx::FromRow)]
        struct NodeRow {
            node_id: String,
            node_name: String,
            node_type: String,
            bitcoin_addresses: String,
            registered_at: DateTime<Utc>,
            last_seen: DateTime<Utc>,
            active: bool,
            metadata: Option<String>,
        }

        let rows: Vec<NodeRow> = sqlx::query_as::<_, NodeRow>(
            "SELECT node_id, node_name, node_type, bitcoin_addresses, registered_at, last_seen, active, metadata FROM node_registry WHERE active = TRUE ORDER BY node_name"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut nodes = Vec::new();
        for row in rows {
            let addresses: Vec<String> = serde_json::from_str(&row.bitcoin_addresses)?;
            let metadata = row
                .metadata
                .as_ref()
                .and_then(|m| serde_json::from_str(m).ok());

            nodes.push(NodeRegistration {
                node_id: row.node_id,
                node_name: row.node_name,
                node_type: NodeType::from_str(&row.node_type),
                bitcoin_addresses: addresses,
                registered_at: row.registered_at,
                last_seen: row.last_seen,
                active: row.active,
                metadata,
            });
        }

        Ok(nodes)
    }
}
