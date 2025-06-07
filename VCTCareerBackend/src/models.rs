use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// Use String for API simplicity
pub type SimulationId = String;

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

// Simulation API Request/Response types
#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct CreateSimulationRequest {
    pub players: Vec<SimulationPlayer>,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct SimulationPlayer {
    pub id: u32,
    pub name: String,
    pub agent: String,
    pub team: String, // "Attackers" or "Defenders"
    pub aim_skill: f32,
    pub hs_skill: f32,
    pub movement_skill: f32,
    pub util_skill: f32,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct CreateSimulationResponse {
    pub simulation_id: SimulationId,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct AdvanceSimulationRequest {
    pub ticks: Option<u32>,
    pub mode: Option<String>, // "tick", "round", "match"
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct SimulationControlRequest {
    pub action: String, // "pause", "resume", "set_speed"
    pub speed: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct EventFilterRequest {
    pub event_types: Option<Vec<String>>,
    pub player_ids: Option<Vec<u32>>,
    pub round_numbers: Option<Vec<u8>>,
    pub start_timestamp: Option<u64>,
    pub end_timestamp: Option<u64>,
}
