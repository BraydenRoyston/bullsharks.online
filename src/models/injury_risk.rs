use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum InjuryRiskType {
    /// High volume spike detected (e.g., >10% increase week-over-week)
    #[serde(rename = "HIGH_VOLUME_SPIKE")]
    HighVolumeSpike,

    #[serde(rename = "SSRD30_NO_RISK")]
    SSRD30NoRisk,

    #[serde(rename = "SSRD30_SMALL_RISK")]
    SSRD30SmallRisk,


    #[serde(rename = "SSRD30_MODERATE_RISK")]
    SSRD30ModerateRisk,

    
    #[serde(rename = "SSRD30_LARGE_RISK")]
    SSRD30LargeRisk,
}

impl InjuryRiskType {
    /// Convert enum variant to string representation for JSON serialization
    pub fn as_str(&self) -> &str {
        match self {
            InjuryRiskType::HighVolumeSpike => "HIGH_VOLUME_SPIKE",
            InjuryRiskType::SSRD30NoRisk => "SSRD30_NO_RISK",
            InjuryRiskType::SSRD30SmallRisk => "SSRD30_SMALL_RISK",
            InjuryRiskType::SSRD30ModerateRisk => "SSRD30_MODERATE_RISK",
            InjuryRiskType::SSRD30LargeRisk => "SSRD30_LARGE_RISK",
        }
    }
}

impl std::fmt::Display for InjuryRiskType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
