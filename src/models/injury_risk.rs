use serde::{Deserialize, Serialize};

/// Centralized enum for injury risk types.
///
/// This enum provides type-safe representation of various injury risks
/// that can be detected through training volume analysis.
///
/// Add additional variants as needed for your specific injury detection algorithms.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum InjuryRiskType {
    /// High volume spike detected (e.g., >10% increase week-over-week)
    #[serde(rename = "HIGH_VOLUME_SPIKE")]
    HighVolumeSpike,

    /// Insufficient recovery time between high-volume weeks
    #[serde(rename = "INSUFFICIENT_RECOVERY")]
    InsufficientRecovery,

    // TODO: Add more injury risk types here
    // Examples:
    // - OVERTRAINING
    // - SUDDEN_MILEAGE_DROP
    // - CONSECUTIVE_HIGH_VOLUME_WEEKS
    // - RAPID_VOLUME_INCREASE
}

impl InjuryRiskType {
    /// Convert enum variant to string representation for JSON serialization
    pub fn as_str(&self) -> &str {
        match self {
            InjuryRiskType::HighVolumeSpike => "HIGH_VOLUME_SPIKE",
            InjuryRiskType::InsufficientRecovery => "INSUFFICIENT_RECOVERY",
        }
    }
}

impl std::fmt::Display for InjuryRiskType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
