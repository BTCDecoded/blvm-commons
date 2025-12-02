//! YAML Configuration Loader for Governance Defaults
//!
//! Extracts configuration values from YAML files and maps them to flat config keys
//! for registration in the ConfigRegistry. This makes YAML the source of truth for
//! all governance configuration.

use crate::error::GovernanceError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Action tier configuration from YAML
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ActionTiersYaml {
    pub tiers: HashMap<String, TierYaml>,
    #[serde(default)]
    pub emergency_tiers: Option<HashMap<String, EmergencyTierYaml>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TierYaml {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub signatures: SignatureYaml,
    pub review_period_days: i64,
    #[serde(default)]
    pub economic_veto: bool,
    #[serde(default)]
    pub emergency_override: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SignatureYaml {
    pub required: u32,
    pub total: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EmergencyTierYaml {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub activation_threshold: String, // "N-of-M" format
    pub review_period_days: u32,
    pub max_duration_days: u32,
    #[serde(default)]
    pub extensions_allowed: u32,
}

/// Repository layer configuration from YAML
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RepositoryLayersYaml {
    pub layers: HashMap<String, LayerYaml>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LayerYaml {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub signatures: SignatureYaml,
    pub review_period_days: i64,
    #[serde(default)]
    pub consensus_review_period_days: Option<i64>,
}

/// Emergency tier configuration from YAML
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EmergencyTiersYaml {
    pub tiers: HashMap<String, EmergencyTierDetailYaml>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EmergencyTierDetailYaml {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub requirements: EmergencyRequirementsYaml,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EmergencyRequirementsYaml {
    pub review_period_days: u32,
    pub signature_threshold: String, // "N-of-M" format
    pub max_duration_days: u32,
    pub activation_threshold: String, // "N-of-M" format
    #[serde(default)]
    pub max_extensions: Option<u32>,
}

/// Economic node configuration from YAML
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EconomicNodesYaml {
    pub veto_mechanism: VetoMechanismYaml,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VetoMechanismYaml {
    pub veto_thresholds: VetoThresholdsYaml,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VetoThresholdsYaml {
    pub tier_3: Option<VetoThresholdYaml>,
    pub tier_4: Option<VetoThresholdYaml>,
    pub tier_5: Option<VetoThresholdYaml>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VetoThresholdYaml {
    pub mining_veto: String, // e.g., "30%+"
    pub economic_veto: String, // e.g., "40%+"
    pub review_period_days: u32,
}

/// YAML configuration loader
pub struct YamlConfigLoader {
    config_path: PathBuf,
}

impl YamlConfigLoader {
    /// Create a new YAML config loader
    pub fn new(config_path: PathBuf) -> Self {
        Self { config_path }
    }

    /// Load action tiers from YAML
    pub fn load_action_tiers(&self) -> Result<ActionTiersYaml, GovernanceError> {
        let path = self.config_path.join("action-tiers.yml");
        self.load_yaml(&path)
    }

    /// Load repository layers from YAML
    pub fn load_repository_layers(&self) -> Result<RepositoryLayersYaml, GovernanceError> {
        let path = self.config_path.join("repository-layers.yml");
        self.load_yaml(&path)
    }

    /// Load emergency tiers from YAML
    pub fn load_emergency_tiers(&self) -> Result<EmergencyTiersYaml, GovernanceError> {
        let path = self.config_path.join("emergency-tiers.yml");
        self.load_yaml(&path)
    }

    /// Load economic nodes from YAML
    pub fn load_economic_nodes(&self) -> Result<EconomicNodesYaml, GovernanceError> {
        let path = self.config_path.join("economic-nodes.yml");
        self.load_yaml(&path)
    }

    /// Load a YAML file and deserialize it
    fn load_yaml<T: for<'de> Deserialize<'de>>(&self, path: &Path) -> Result<T, GovernanceError> {
        if !path.exists() {
            return Err(GovernanceError::ConfigError(format!(
                "Configuration file not found: {:?}",
                path
            )));
        }

        let contents = std::fs::read_to_string(path).map_err(|e| {
            GovernanceError::ConfigError(format!("Failed to read {:?}: {}", path, e))
        })?;

        serde_yaml::from_str(&contents)
            .map_err(|e| GovernanceError::ConfigError(format!("Failed to parse {:?}: {}", path, e)))
    }

    /// Extract all config values from YAML files and return as flat key-value pairs
    pub fn extract_all_config_values(&self) -> Result<HashMap<String, serde_json::Value>, GovernanceError> {
        let mut config_values = HashMap::new();

        // Extract from action tiers
        if let Ok(action_tiers) = self.load_action_tiers() {
            self.extract_action_tier_values(&action_tiers, &mut config_values)?;
        } else {
            warn!("Failed to load action-tiers.yml, skipping");
        }

        // Extract from repository layers
        if let Ok(repo_layers) = self.load_repository_layers() {
            self.extract_repository_layer_values(&repo_layers, &mut config_values)?;
        } else {
            warn!("Failed to load repository-layers.yml, skipping");
        }

        // Extract from emergency tiers
        if let Ok(emergency_tiers) = self.load_emergency_tiers() {
            self.extract_emergency_tier_values(&emergency_tiers, &mut config_values)?;
        } else {
            warn!("Failed to load emergency-tiers.yml, skipping");
        }

        // Extract from economic nodes (veto thresholds)
        if let Ok(economic_nodes) = self.load_economic_nodes() {
            self.extract_veto_threshold_values(&economic_nodes, &mut config_values)?;
        } else {
            warn!("Failed to load economic-nodes.yml, skipping");
        }

        info!("Extracted {} config values from YAML files", config_values.len());
        Ok(config_values)
    }

    /// Extract action tier values from YAML
    fn extract_action_tier_values(
        &self,
        action_tiers: &ActionTiersYaml,
        config_values: &mut HashMap<String, serde_json::Value>,
    ) -> Result<(), GovernanceError> {
        for (tier_key, tier) in &action_tiers.tiers {
            // Map tier keys: tier_1_routine -> tier_1, tier_2_features -> tier_2, etc.
            let tier_num = self.extract_tier_number(tier_key)?;

            // Signature requirements
            let sig_key_req = format!("tier_{}_signatures_required", tier_num);
            let sig_key_total = format!("tier_{}_signatures_total", tier_num);
            config_values.insert(sig_key_req, serde_json::json!(tier.signatures.required));
            config_values.insert(sig_key_total, serde_json::json!(tier.signatures.total));

            // Review period
            let review_key = format!("tier_{}_review_period_days", tier_num);
            config_values.insert(review_key, serde_json::json!(tier.review_period_days));
        }

        Ok(())
    }

    /// Extract repository layer values from YAML
    fn extract_repository_layer_values(
        &self,
        repo_layers: &RepositoryLayersYaml,
        config_values: &mut HashMap<String, serde_json::Value>,
    ) -> Result<(), GovernanceError> {
        for (layer_key, layer) in &repo_layers.layers {
            // Map layer keys: layer_1_constitutional -> layer_1_2, layer_3_implementation -> layer_3, etc.
            let layer_num = self.extract_layer_number(layer_key)?;

            // Signature requirements
            let sig_key_req = format!("layer_{}_signatures_required", layer_num);
            let sig_key_total = format!("layer_{}_signatures_total", layer_num);
            config_values.insert(sig_key_req, serde_json::json!(layer.signatures.required));
            config_values.insert(sig_key_total, serde_json::json!(layer.signatures.total));

            // Review period
            let review_key = format!("layer_{}_review_period_days", layer_num);
            config_values.insert(review_key, serde_json::json!(layer.review_period_days));
        }

        Ok(())
    }

    /// Extract emergency tier values from YAML
    fn extract_emergency_tier_values(
        &self,
        emergency_tiers: &EmergencyTiersYaml,
        config_values: &mut HashMap<String, serde_json::Value>,
    ) -> Result<(), GovernanceError> {
        for (tier_key, tier) in &emergency_tiers.tiers {
            // Map tier keys: tier_1_critical -> 1, tier_2_urgent -> 2, tier_3_elevated -> 3
            let tier_num = self.extract_emergency_tier_number(tier_key)?;

            // Review period
            let review_key = format!("emergency_tier_{}_review_period_days", tier_num);
            config_values.insert(review_key, serde_json::json!(tier.requirements.review_period_days));

            // Max duration
            let duration_key = format!("emergency_tier_{}_max_duration_days", tier_num);
            config_values.insert(duration_key, serde_json::json!(tier.requirements.max_duration_days));

            // Signature threshold (parse "N-of-M" format)
            let (n, m) = self.parse_threshold_pair(&tier.requirements.signature_threshold)?;
            let sig_n_key = format!("emergency_tier_{}_signature_threshold_n", tier_num);
            let sig_m_key = format!("emergency_tier_{}_signature_threshold_m", tier_num);
            config_values.insert(sig_n_key, serde_json::json!(n));
            config_values.insert(sig_m_key, serde_json::json!(m));

            // Activation threshold
            let (act_n, act_m) = self.parse_threshold_pair(&tier.requirements.activation_threshold)?;
            let act_n_key = format!("emergency_tier_{}_activation_threshold_n", tier_num);
            let act_m_key = format!("emergency_tier_{}_activation_threshold_m", tier_num);
            config_values.insert(act_n_key, serde_json::json!(act_n));
            config_values.insert(act_m_key, serde_json::json!(act_m));

            // Max extensions
            if let Some(max_ext) = tier.requirements.max_extensions {
                let ext_key = format!("emergency_tier_{}_max_extensions", tier_num);
                config_values.insert(ext_key, serde_json::json!(max_ext));
            }
        }

        Ok(())
    }

    /// Extract veto threshold values from YAML
    fn extract_veto_threshold_values(
        &self,
        economic_nodes: &EconomicNodesYaml,
        config_values: &mut HashMap<String, serde_json::Value>,
    ) -> Result<(), GovernanceError> {
        if let Some(ref thresholds) = economic_nodes.veto_mechanism.veto_thresholds {
            // Tier 3 veto thresholds
            if let Some(ref tier_3) = thresholds.tier_3 {
                let mining = self.parse_percentage(&tier_3.mining_veto)?;
                let economic = self.parse_percentage(&tier_3.economic_veto)?;
                config_values.insert("veto_tier_3_mining_percent".to_string(), serde_json::json!(mining));
                config_values.insert("veto_tier_3_economic_percent".to_string(), serde_json::json!(economic));
                config_values.insert("veto_tier_3_review_period_days".to_string(), serde_json::json!(tier_3.review_period_days));
            }

            // Tier 4 veto thresholds
            if let Some(ref tier_4) = thresholds.tier_4 {
                let mining = self.parse_percentage(&tier_4.mining_veto)?;
                let economic = self.parse_percentage(&tier_4.economic_veto)?;
                config_values.insert("veto_tier_4_mining_percent".to_string(), serde_json::json!(mining));
                config_values.insert("veto_tier_4_economic_percent".to_string(), serde_json::json!(economic));
                config_values.insert("veto_tier_4_review_period_days".to_string(), serde_json::json!(tier_4.review_period_days));
            }

            // Tier 5 veto thresholds
            if let Some(ref tier_5) = thresholds.tier_5 {
                let mining = self.parse_percentage(&tier_5.mining_veto)?;
                let economic = self.parse_percentage(&tier_5.economic_veto)?;
                config_values.insert("veto_tier_5_mining_percent".to_string(), serde_json::json!(mining));
                config_values.insert("veto_tier_5_economic_percent".to_string(), serde_json::json!(economic));
                config_values.insert("veto_tier_5_review_period_days".to_string(), serde_json::json!(tier_5.review_period_days));
            }
        }

        Ok(())
    }

    /// Extract tier number from tier key (e.g., "tier_1_routine" -> 1)
    fn extract_tier_number(&self, tier_key: &str) -> Result<u32, GovernanceError> {
        if tier_key.starts_with("tier_") {
            let parts: Vec<&str> = tier_key.split('_').collect();
            if parts.len() >= 2 {
                if let Ok(num) = parts[1].parse::<u32>() {
                    return Ok(num);
                }
            }
        }
        Err(GovernanceError::ConfigError(format!(
            "Invalid tier key format: {}",
            tier_key
        )))
    }

    /// Extract layer number from layer key (e.g., "layer_1_constitutional" -> "1_2", "layer_3_implementation" -> "3")
    fn extract_layer_number(&self, layer_key: &str) -> Result<String, GovernanceError> {
        if layer_key.starts_with("layer_") {
            let parts: Vec<&str> = layer_key.split('_').collect();
            if parts.len() >= 2 {
                let num = parts[1];
                // Map layer_1 and layer_2 to "1_2" (constitutional layers)
                if num == "1" || num == "2" {
                    return Ok("1_2".to_string());
                }
                // Other layers use their number directly
                return Ok(num.to_string());
            }
        }
        Err(GovernanceError::ConfigError(format!(
            "Invalid layer key format: {}",
            layer_key
        )))
    }

    /// Extract emergency tier number from tier key (e.g., "tier_1_critical" -> 1)
    fn extract_emergency_tier_number(&self, tier_key: &str) -> Result<u32, GovernanceError> {
        if tier_key.starts_with("tier_") {
            let parts: Vec<&str> = tier_key.split('_').collect();
            if parts.len() >= 2 {
                if let Ok(num) = parts[1].parse::<u32>() {
                    return Ok(num);
                }
            }
        }
        Err(GovernanceError::ConfigError(format!(
            "Invalid emergency tier key format: {}",
            tier_key
        )))
    }

    /// Parse threshold pair from "N-of-M" format
    fn parse_threshold_pair(&self, threshold: &str) -> Result<(u32, u32), GovernanceError> {
        // Handle formats like "4-of-7", "4 of 7", "4/7"
        let parts: Vec<&str> = threshold
            .split(|c: char| c == '-' || c == ' ' || c == '/')
            .collect();

        if parts.len() >= 2 {
            let n = parts[0].trim().parse::<u32>().map_err(|_| {
                GovernanceError::ConfigError(format!("Invalid threshold N value: {}", threshold))
            })?;
            let m = parts[parts.len() - 1].trim().parse::<u32>().map_err(|_| {
                GovernanceError::ConfigError(format!("Invalid threshold M value: {}", threshold))
            })?;
            return Ok((n, m));
        }

        Err(GovernanceError::ConfigError(format!(
            "Invalid threshold format: {} (expected 'N-of-M')",
            threshold
        )))
    }

    /// Parse percentage from string like "30%+" -> 30.0
    fn parse_percentage(&self, percent_str: &str) -> Result<f64, GovernanceError> {
        let cleaned = percent_str.trim().trim_end_matches('%').trim_end_matches('+');
        cleaned.parse::<f64>().map_err(|_| {
            GovernanceError::ConfigError(format!("Invalid percentage format: {}", percent_str))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_tier_number() {
        let loader = YamlConfigLoader::new(PathBuf::from("/tmp"));
        assert_eq!(loader.extract_tier_number("tier_1_routine").unwrap(), 1);
        assert_eq!(loader.extract_tier_number("tier_5_governance").unwrap(), 5);
    }

    #[test]
    fn test_parse_threshold_pair() {
        let loader = YamlConfigLoader::new(PathBuf::from("/tmp"));
        assert_eq!(loader.parse_threshold_pair("4-of-7").unwrap(), (4, 7));
        assert_eq!(loader.parse_threshold_pair("5 of 7").unwrap(), (5, 7));
        assert_eq!(loader.parse_threshold_pair("6/7").unwrap(), (6, 7));
    }

    #[test]
    fn test_parse_percentage() {
        let loader = YamlConfigLoader::new(PathBuf::from("/tmp"));
        assert_eq!(loader.parse_percentage("30%+").unwrap(), 30.0);
        assert_eq!(loader.parse_percentage("40%").unwrap(), 40.0);
    }
}

