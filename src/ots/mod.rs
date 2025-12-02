//! OpenTimestamps Integration Module
//!
//! This module provides historical proof of governance operations
//! by anchoring monthly registries to the Bitcoin blockchain.

pub mod anchor;
pub mod client;
pub mod verify;

pub use anchor::RegistryAnchorer;
pub use client::OtsClient;
pub use verify::verify_registry;
