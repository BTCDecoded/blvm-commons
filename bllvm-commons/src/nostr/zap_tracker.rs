//! Zap Tracker Service
//!
//! Tracks Lightning zaps (NIP-57) for governance contributions.
//! Subscribes to zap receipt events from Nostr relays and records them in the database.

use crate::governance::ContributionTracker;
use crate::nostr::{NostrClient, ZapEvent};
use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use std::sync::Arc;
use tracing::{info, warn};

/// Zap tracker service that monitors and records zap contributions
pub struct ZapTracker {
    pool: SqlitePool,
    nostr_client: Arc<NostrClient>,
    bot_pubkeys: Vec<String>,  // All bot pubkeys to track
}

impl ZapTracker {
    /// Create a new zap tracker
    pub fn new(pool: SqlitePool, nostr_client: Arc<NostrClient>, bot_pubkeys: Vec<String>) -> Self {
        Self {
            pool,
            nostr_client,
            bot_pubkeys,
        }
    }
    
    /// Start tracking zaps for all bot pubkeys
    pub async fn start_tracking(&self) -> Result<()> {
        // Subscribe to zaps for each bot pubkey
        for pubkey in &self.bot_pubkeys {
            let mut zap_rx = Arc::as_ref(&self.nostr_client).subscribe_to_zaps(pubkey).await?;
            
            // Spawn task to process zaps for this pubkey
            let pool = self.pool.clone();
            let pubkey_clone = pubkey.clone();
            tokio::spawn(async move {
                while let Some(zap) = zap_rx.recv().await {
                    if let Err(e) = Self::process_zap(&pool, &pubkey_clone, zap).await {
                        warn!("Failed to process zap: {}", e);
                    }
                }
            });
        }
        
        info!("Started tracking zaps for {} bot pubkeys", self.bot_pubkeys.len());
        Ok(())
    }
    
    /// Process a zap event and record it in the database
    async fn process_zap(
        pool: &SqlitePool,
        recipient_pubkey: &str,
        zap: ZapEvent,
    ) -> Result<()> {
        // Convert millisatoshis to BTC
        let amount_btc = zap.amount_msat as f64 / 100_000_000_000.0;
        
        // Convert timestamp to DateTime
        let timestamp = DateTime::from_timestamp(zap.timestamp, 0)
            .unwrap_or_else(Utc::now);
        
        // Determine if this is a proposal zap (has zapped_event_id)
        let is_proposal_zap = zap.zapped_event_id.is_some();
        
        // Record zap in database
        let invoice_hash = zap.invoice.as_ref().and_then(|i| Self::extract_payment_hash(i));
        let governance_event_id = zap.zapped_event_id.clone();
        sqlx::query(
            r#"
            INSERT INTO zap_contributions
            (recipient_pubkey, sender_pubkey, amount_msat, amount_btc, timestamp, invoice_hash, message, zapped_event_id, is_proposal_zap, governance_event_id)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(recipient_pubkey)
        .bind(zap.sender_pubkey.as_deref())
        .bind(zap.amount_msat as i64)
        .bind(amount_btc)
        .bind(timestamp)
        .bind(invoice_hash.as_deref())
        .bind(zap.message.as_deref())
        .bind(zap.zapped_event_id.as_deref())
        .bind(is_proposal_zap)
        .bind(governance_event_id.as_deref())
        .execute(pool)
        .await?;
        
        info!(
            "Recorded zap: {} msat ({:.8} BTC) to {} from {}",
            zap.amount_msat,
            amount_btc,
            recipient_pubkey,
            zap.sender_pubkey.as_deref().unwrap_or("unknown")
        );
        
        // Also record in unified contributions if we have sender pubkey
        if let Some(ref sender_pubkey) = zap.sender_pubkey {
            let tracker = ContributionTracker::new(pool.clone());
            if let Err(e) = tracker
                .record_zap_contribution(
                    sender_pubkey,
                    amount_btc,
                    timestamp,
                    is_proposal_zap,
                )
                .await
            {
                warn!("Failed to record zap in unified contributions: {}", e);
            }
        }
        
        Ok(())
    }
    
    /// Extract payment hash from invoice (for verification)
    /// This is a placeholder - in production, use a bolt11 parsing library
    fn extract_payment_hash(_invoice: &str) -> Option<String> {
        // TODO: Parse bolt11 invoice to extract payment hash
        // For now, return None
        None
    }
    
    /// Get total zaps for a pubkey in time period
    pub async fn get_total_zaps(
        &self,
        pubkey: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<f64> {
        let result: Option<f64> = sqlx::query_scalar(
            r#"
            SELECT SUM(amount_btc) as total
            FROM zap_contributions
            WHERE recipient_pubkey = ? 
              AND timestamp >= ? 
              AND timestamp <= ?
            "#,
        )
        .bind(pubkey)
        .bind(start_time)
        .bind(end_time)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(result.unwrap_or(0.0))
    }
    
    /// Get zaps by sender (for contributor qualification)
    pub async fn get_zaps_by_sender(
        &self,
        sender_pubkey: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<ZapContribution>> {
        #[derive(sqlx::FromRow)]
        struct ZapRow {
            id: i32,
            recipient_pubkey: String,
            sender_pubkey: Option<String>,
            amount_msat: i64,
            amount_btc: f64,
            timestamp: DateTime<Utc>,
            invoice_hash: Option<String>,
            message: Option<String>,
            zapped_event_id: Option<String>,
            is_proposal_zap: bool,
            governance_event_id: Option<String>,
        }
        
        let rows = sqlx::query_as::<_, ZapRow>(
            r#"
            SELECT id, recipient_pubkey, sender_pubkey, amount_msat, amount_btc, timestamp, invoice_hash, message, zapped_event_id, is_proposal_zap, governance_event_id
            FROM zap_contributions
            WHERE sender_pubkey = ?
              AND timestamp >= ?
              AND timestamp <= ?
            ORDER BY timestamp DESC
            "#,
        )
        .bind(sender_pubkey)
        .bind(start_time)
        .bind(end_time)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(rows.into_iter().map(|row| ZapContribution {
            id: row.id,
            recipient_pubkey: row.recipient_pubkey,
            sender_pubkey: row.sender_pubkey,
            amount_msat: row.amount_msat as u64,
            amount_btc: row.amount_btc,
            timestamp: row.timestamp,
            invoice_hash: row.invoice_hash,
            message: row.message,
            zapped_event_id: row.zapped_event_id,
            is_proposal_zap: row.is_proposal_zap,
            governance_event_id: row.governance_event_id,
        }).collect())
    }
    
    /// Get proposal zaps (zaps to governance events)
    pub async fn get_proposal_zaps(
        &self,
        governance_event_id: &str,
    ) -> Result<Vec<ZapContribution>> {
        #[derive(sqlx::FromRow)]
        struct ZapRow {
            id: i32,
            recipient_pubkey: String,
            sender_pubkey: Option<String>,
            amount_msat: i64,
            amount_btc: f64,
            timestamp: DateTime<Utc>,
            invoice_hash: Option<String>,
            message: Option<String>,
            zapped_event_id: Option<String>,
            is_proposal_zap: bool,
            governance_event_id: Option<String>,
        }
        
        let rows = sqlx::query_as::<_, ZapRow>(
            r#"
            SELECT id, recipient_pubkey, sender_pubkey, amount_msat, amount_btc, timestamp, invoice_hash, message, zapped_event_id, is_proposal_zap, governance_event_id
            FROM zap_contributions
            WHERE governance_event_id = ?
            ORDER BY timestamp DESC
            "#,
        )
        .bind(governance_event_id)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(rows.into_iter().map(|row| ZapContribution {
            id: row.id,
            recipient_pubkey: row.recipient_pubkey,
            sender_pubkey: row.sender_pubkey,
            amount_msat: row.amount_msat as u64,
            amount_btc: row.amount_btc,
            timestamp: row.timestamp,
            invoice_hash: row.invoice_hash,
            message: row.message,
            zapped_event_id: row.zapped_event_id,
            is_proposal_zap: row.is_proposal_zap,
            governance_event_id: row.governance_event_id,
        }).collect())
    }
}

/// Zap contribution record from database
#[derive(Debug, Clone)]
pub struct ZapContribution {
    pub id: i32,
    pub recipient_pubkey: String,
    pub sender_pubkey: Option<String>,
    pub amount_msat: u64,
    pub amount_btc: f64,
    pub timestamp: DateTime<Utc>,
    pub invoice_hash: Option<String>,
    pub message: Option<String>,
    pub zapped_event_id: Option<String>,
    pub is_proposal_zap: bool,
    pub governance_event_id: Option<String>,
}

