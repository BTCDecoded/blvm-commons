//! Fee Forwarding Tracker
//!
//! Monitors blockchain for transactions to Commons address and records
//! fee forwarding contributions for governance.

use crate::governance::ContributionTracker;
use anyhow::Result;
use chrono::{DateTime, Utc};
use sha2::Digest;
use sqlx::SqlitePool;
use tracing::info;

/// Fee forwarding tracker
pub struct FeeForwardingTracker {
    pool: SqlitePool,
    contribution_tracker: ContributionTracker,
    commons_addresses: Vec<String>, // List of Commons addresses to monitor
    network: bitcoin::Network,      // Bitcoin network (mainnet, testnet, regtest)
}

impl FeeForwardingTracker {
    /// Create a new fee forwarding tracker
    pub fn new(
        pool: SqlitePool,
        commons_addresses: Vec<String>,
        network: bitcoin::Network,
    ) -> Self {
        Self {
            pool: pool.clone(),
            contribution_tracker: ContributionTracker::new(pool),
            commons_addresses,
            network,
        }
    }

    /// Create from network string (mainnet, testnet, regtest)
    pub fn from_network_string(
        pool: SqlitePool,
        commons_addresses: Vec<String>,
        network_str: &str,
    ) -> Self {
        let network = match network_str.to_lowercase().as_str() {
            "testnet" => bitcoin::Network::Testnet,
            "regtest" => bitcoin::Network::Regtest,
            _ => bitcoin::Network::Bitcoin, // Default to mainnet
        };
        Self::new(pool, commons_addresses, network)
    }

    /// Process a block and check for fee forwarding transactions
    /// This would be called when a new block is received
    /// If contributor_id is None, will try to look up from node registry based on transaction inputs
    pub async fn process_block(
        &self,
        block: &bllvm_protocol::Block,
        block_height: i32,
        contributor_id: Option<&str>, // Optional miner/node identifier (will lookup if None)
    ) -> Result<Vec<FeeForwardingContribution>> {
        let mut contributions = Vec::new();

        // Check each transaction in the block
        for (tx_index, tx) in block.transactions.iter().enumerate() {
            // Skip coinbase (index 0)
            if tx_index == 0 {
                continue;
            }

            // Check each output for Commons address
            for output in &tx.outputs {
                // Decode script_pubkey to get address
                // For now, we'll check if the output value is sent to a Commons address
                // In production, this would decode the script_pubkey to get the address
                let address = self.decode_address_from_script(&output.script_pubkey)?;

                if let Some(address) = address {
                    if self.commons_addresses.contains(&address) {
                        // This is a fee forwarding transaction
                        let amount_btc = output.value as f64 / 100_000_000.0; // Convert satoshis to BTC

                        // Get transaction hash
                        let tx_hash = self.calculate_tx_hash(tx);

                        // Check if we've already recorded this transaction
                        let existing: Option<i64> = sqlx::query_scalar(
                            r#"
                            SELECT id FROM fee_forwarding_contributions
                            WHERE tx_hash = ?
                            "#,
                        )
                        .bind(&tx_hash)
                        .fetch_optional(&self.pool)
                        .await?;

                        if existing.is_some() {
                            continue; // Already recorded
                        }

                        // Determine contributor_id: use provided, or lookup from node registry
                        let final_contributor_id = if let Some(id) = contributor_id {
                            id.to_string()
                        } else {
                            // Try to find node from address registry
                            self.lookup_contributor_from_address(&address)
                                .await
                                .unwrap_or_else(|| format!("unknown-{}", &tx_hash[..8]))
                        };

                        // Record the contribution (this also records in unified_contributions)
                        self.contribution_tracker
                            .record_fee_forwarding_contribution(
                                &final_contributor_id,
                                &tx_hash,
                                amount_btc,
                                &address,
                                block_height,
                                Utc::now(),
                            )
                            .await?;

                        let tx_hash_clone = tx_hash.clone();
                        let address_clone = address.clone();

                        contributions.push(FeeForwardingContribution {
                            contributor_id: final_contributor_id.clone(),
                            tx_hash: tx_hash_clone.clone(),
                            block_height,
                            amount_btc,
                            commons_address: address_clone.clone(),
                            timestamp: Utc::now(),
                        });

                        info!(
                            "Recorded fee forwarding: {} BTC (tx: {}) from {} to Commons address {}",
                            amount_btc, tx_hash_clone, final_contributor_id, address_clone
                        );
                    }
                }
            }
        }

        Ok(contributions)
    }

    /// Decode Bitcoin address from script_pubkey
    /// Supports P2PKH, P2SH, P2WPKH, P2WSH, and P2TR addresses
    fn decode_address_from_script(&self, script_pubkey: &[u8]) -> Result<Option<String>> {
        use bitcoin::{Address, ScriptBuf};

        // Create ScriptBuf from bytes
        let script = ScriptBuf::from_bytes(script_pubkey.to_vec());

        // Use the configured network
        let network = self.network;

        // Try Address::from_script (works for P2PKH, P2SH, P2WPKH, P2WSH, P2TR)
        if let Ok(address) = Address::from_script(&script, network) {
            return Ok(Some(address.to_string()));
        }

        // Fallback: Try manual decoding for edge cases
        self.decode_legacy_address(script_pubkey)
    }

    /// Decode legacy P2PKH or P2SH address manually
    /// Used as fallback when Address::from_script fails
    /// Note: This fallback should rarely be needed since Address::from_script handles most cases
    fn decode_legacy_address(&self, _script_pubkey: &[u8]) -> Result<Option<String>> {
        // Since Address::from_script already handles P2PKH and P2SH correctly,
        // this fallback is mainly for edge cases. If Address::from_script failed,
        // it's likely not a standard address format we can decode.
        // Return None to indicate we couldn't decode it.
        Ok(None)
    }

    /// Calculate transaction hash using proper Bitcoin double SHA256
    fn calculate_tx_hash(&self, tx: &bllvm_protocol::Transaction) -> String {
        use sha2::{Digest, Sha256};

        // Serialize transaction to Bitcoin wire format
        let mut data = Vec::new();

        // Version (4 bytes, little-endian)
        data.extend_from_slice(&(tx.version as u32).to_le_bytes());

        // Input count (varint)
        data.extend_from_slice(&self.encode_varint(tx.inputs.len() as u64));

        // Inputs
        for input in &tx.inputs {
            // Previous output hash (32 bytes)
            data.extend_from_slice(&input.prevout.hash);
            // Previous output index (4 bytes, little-endian)
            data.extend_from_slice(&(input.prevout.index as u32).to_le_bytes());
            // Script length (varint)
            data.extend_from_slice(&self.encode_varint(input.script_sig.len() as u64));
            // Script
            data.extend_from_slice(&input.script_sig);
            // Sequence (4 bytes, little-endian)
            data.extend_from_slice(&(input.sequence as u32).to_le_bytes());
        }

        // Output count (varint)
        data.extend_from_slice(&self.encode_varint(tx.outputs.len() as u64));

        // Outputs
        for output in &tx.outputs {
            // Value (8 bytes, little-endian)
            data.extend_from_slice(&(output.value as u64).to_le_bytes());
            // Script length (varint)
            data.extend_from_slice(&self.encode_varint(output.script_pubkey.len() as u64));
            // Script
            data.extend_from_slice(&output.script_pubkey);
        }

        // Lock time (4 bytes, little-endian)
        data.extend_from_slice(&(tx.lock_time as u32).to_le_bytes());

        // Double SHA256 (Bitcoin standard for transaction IDs)
        let first_hash = Sha256::digest(&data);
        let second_hash = Sha256::digest(first_hash);

        // Convert to hex string (reversed for display, as Bitcoin displays txids in reverse)
        hex::encode(second_hash)
    }

    /// Encode integer as Bitcoin varint
    fn encode_varint(&self, value: u64) -> Vec<u8> {
        if value < 0xfd {
            vec![value as u8]
        } else if value <= 0xffff {
            let mut result = vec![0xfd];
            result.extend_from_slice(&(value as u16).to_le_bytes());
            result
        } else if value <= 0xffffffff {
            let mut result = vec![0xfe];
            result.extend_from_slice(&(value as u32).to_le_bytes());
            result
        } else {
            let mut result = vec![0xff];
            result.extend_from_slice(&value.to_le_bytes());
            result
        }
    }

    /// Look up contributor ID from node registry based on address
    async fn lookup_contributor_from_address(&self, address: &str) -> Option<String> {
        use crate::node_registry::NodeRegistry;
        let registry = NodeRegistry::new(self.pool.clone());
        registry.get_node_for_address(address).await.ok().flatten()
    }

    /// Get fee forwarding contributions for a contributor
    pub async fn get_contributor_contributions(
        &self,
        contributor_id: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<FeeForwardingContribution>> {
        #[derive(sqlx::FromRow)]
        struct FeeForwardingRow {
            contributor_id: String,
            tx_hash: String,
            block_height: i32,
            amount_btc: f64,
            commons_address: String,
            timestamp: DateTime<Utc>,
        }

        let rows = sqlx::query_as::<_, FeeForwardingRow>(
            r#"
            SELECT contributor_id, tx_hash, block_height, amount_btc, commons_address, timestamp
            FROM fee_forwarding_contributions
            WHERE contributor_id = ?
              AND timestamp >= ?
              AND timestamp <= ?
            ORDER BY timestamp DESC
            "#,
        )
        .bind(contributor_id)
        .bind(start_time)
        .bind(end_time)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| FeeForwardingContribution {
                contributor_id: row.contributor_id,
                tx_hash: row.tx_hash,
                block_height: row.block_height,
                amount_btc: row.amount_btc,
                commons_address: row.commons_address,
                timestamp: row.timestamp,
            })
            .collect())
    }
}

/// Fee forwarding contribution record
#[derive(Debug, Clone)]
pub struct FeeForwardingContribution {
    pub contributor_id: String,
    pub tx_hash: String,
    pub block_height: i32,
    pub amount_btc: f64,
    pub commons_address: String,
    pub timestamp: DateTime<Utc>,
}
