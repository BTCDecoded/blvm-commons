//! Nostr Publisher CLI
//!
//! Command-line tool for publishing Nostr events from GitHub Actions
//! Usage: nostr-publisher --bot <bot_id> --event-type <type> --content <content>

use anyhow::Result;
use clap::Parser;
use nostr_sdk::prelude::*;
use std::env;

#[derive(Parser)]
#[command(name = "nostr-publisher")]
#[command(about = "Publish Nostr events from GitHub Actions")]
struct Args {
    /// Bot identity (gov, dev, research, network)
    #[arg(long)]
    bot: String,

    /// Event type (merge, release, milestone, etc.)
    #[arg(long)]
    event_type: String,

    /// Event content (JSON or markdown text)
    #[arg(long)]
    content: String,

    /// Nostr event kind (1=text note, 30023=long-form, etc.)
    #[arg(long, default_value = "1")]
    kind: String,

    /// Comma-separated list of relay URLs
    #[arg(long)]
    relays: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    // Get nsec from environment variable
    let nsec_var = format!("NOSTR_NSEC_{}", args.bot.to_uppercase());
    let nsec = env::var(&nsec_var).map_err(|_| {
        anyhow::anyhow!(
            "Environment variable {} not set. Make sure the GitHub secret is configured.",
            nsec_var
        )
    })?;

    // Parse relays
    let relays: Vec<String> = args
        .relays
        .unwrap_or_else(|| {
            env::var("NOSTR_RELAYS")
                .unwrap_or_else(|_| "wss://relay.damus.io,wss://nos.lol".to_string())
        })
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    // Create Nostr keys
    let keys = Keys::from_sk_str(&nsec).map_err(|e| anyhow::anyhow!("Invalid nsec key: {}", e))?;

    // Create client
    let client = Client::new(&keys);

    // Connect to relays
    for relay_url in &relays {
        match client.add_relay(relay_url.clone()).await {
            Ok(_) => {
                tracing::info!("Connected to relay: {}", relay_url);
            }
            Err(e) => {
                tracing::warn!("Failed to connect to relay {}: {}", relay_url, e);
            }
        }
    }

    client.connect().await;

    // Parse event kind
    let kind: u64 = args
        .kind
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid event kind: {}", args.kind))?;

    // Create event tags
    let mut tags = vec![
        Tag::Generic(TagKind::Custom("t".into()), vec![args.event_type.clone()]),
        Tag::Generic(TagKind::Custom("bot".into()), vec![args.bot.clone()]),
    ];

    // Add zap tag if lightning address is available
    let lightning_var = format!("NOSTR_LIGHTNING_{}", args.bot.to_uppercase());
    if let Ok(lightning) = env::var(&lightning_var) {
        tags.push(Tag::Generic(TagKind::Custom("zap".into()), vec![lightning]));
    }

    // Create event
    let event = EventBuilder::new(Kind::from(kind), args.content, tags)
        .to_event(&keys)
        .map_err(|e| anyhow::anyhow!("Failed to create Nostr event: {}", e))?;

    // Publish event
    let mut published = 0;
    let relays_list = client.relays().await;

    for (relay_url, relay) in &relays_list {
        match relay
            .send_event(event.clone(), RelaySendOptions::new())
            .await
        {
            Ok(_) => {
                tracing::info!("Published to relay: {}", relay_url);
                published += 1;
            }
            Err(e) => {
                tracing::warn!("Failed to publish to relay {}: {}", relay_url, e);
            }
        }
    }

    if published == 0 {
        return Err(anyhow::anyhow!("Failed to publish to any relay"));
    }

    tracing::info!(
        "✅ Successfully published to {}/{} relays",
        published,
        relays_list.len()
    );
    tracing::info!("Event ID: {}", event.id);
    tracing::info!("Bot: @BTCCommons_{}", args.bot);

    Ok(())
}
