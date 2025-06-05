use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct CareerInfo {
    pub age: u32,
    pub current_rank: String,
    pub past_experience: String,
    pub division: String,
}

#[derive(Serialize)]
pub enum StartingTier {
    RankedPlay,
    Tier3,
    Tier2,
    Tier1,
}

impl StartingTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            StartingTier::RankedPlay => "Ranked Play",
            StartingTier::Tier3 => "Tier 3 (College / Premier)",
            StartingTier::Tier2 => "Tier 2 (College / Challengers)",
            StartingTier::Tier1 => "Tier 1 (VCT)",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct Team {
    pub team_name: String,
    pub region: String,
    pub tier: Option<i16>,
    pub ranking: Option<i32>,
    pub budget: Option<i64>,
    pub expenses: Option<i64>,
}
