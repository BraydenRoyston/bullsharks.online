use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum InjuryRiskType {
    /// High volume spike detected (e.g., >10% increase week-over-week)
    #[serde(rename = "HIGH_VOLUME_SPIKE")]
    HighVolumeSpike,

    #[serde(rename = "SSRD_30")]
    SSRD30
}

impl InjuryRiskType {
    /// Convert enum variant to string representation for JSON serialization
    pub fn as_str(&self) -> &str {
        match self {
            InjuryRiskType::HighVolumeSpike => "HIGH_VOLUME_SPIKE",
            InjuryRiskType::SSRD30 => "SSRD30"
        }
    }
}

impl std::fmt::Display for InjuryRiskType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
