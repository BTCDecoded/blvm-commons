//! Build orchestration module
//!
//! Handles cross-repository build coordination, dependency management,
//! build monitoring, and artifact collection for releases.

pub mod dependency;
pub mod monitor;
pub mod orchestrator;
pub mod artifacts;

#[cfg(test)]
mod tests;

pub use dependency::DependencyGraph;
pub use monitor::BuildMonitor;
pub use orchestrator::BuildOrchestrator;
pub use artifacts::ArtifactCollector;

