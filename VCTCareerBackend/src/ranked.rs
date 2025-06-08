use std::str::FromStr;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
// #[serde(rename_all = "camelCase")]
pub struct MatchInput {
    pub rank: String,
    pub is_win: bool,
    pub rounds_won: i32,
    pub rounds_lost: i32,
    pub acs_percentile: f32,
    pub hidden_mmr: i32,
    pub five_stack_penalty: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RrEstimateResponse {
    pub rr_change: i32,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct MapPoolRequest {
    #[serde(default, deserialize_with = "crate::ranked::deserialize_maps_param")]
    pub maps: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RandomMapResponse {
    pub selected_map: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RankTier {
    Iron,
    Bronze,
    Silver,
    Gold,
    Platinum,
    Diamond,
    Ascendant,
    Immortal,
    Radiant,
}
#[derive(Debug)]
pub enum RankTierParseError {
    InvalidRankName(String),
}

impl FromStr for RankTier {
    type Err = RankTierParseError;

    fn from_str(s: &str) -> Result<RankTier, RankTierParseError> {
        match s.to_lowercase().as_str() {
            "iron" => Ok(RankTier::Iron),
            "bronze" => Ok(RankTier::Bronze),
            "silver" => Ok(RankTier::Silver),
            "gold" => Ok(RankTier::Gold),
            "platinum" => Ok(RankTier::Platinum),
            "diamond" => Ok(RankTier::Diamond),
            "ascendant" => Ok(RankTier::Ascendant),
            "immortal" => Ok(RankTier::Immortal),
            "radiant" => Ok(RankTier::Radiant),
            _ => Err(RankTierParseError::InvalidRankName(s.to_string())),
        }
    }
}

pub fn get_rank_mmr_range(rank: RankTier) -> (i32, i32) {
    match rank {
        RankTier::Iron => (0, 299),
        RankTier::Bronze => (300, 599),
        RankTier::Silver => (600, 899),
        RankTier::Gold => (900, 1199),
        RankTier::Platinum => (1200, 1499),
        RankTier::Diamond => (1500, 1799),
        RankTier::Ascendant => (1800, 2099),
        RankTier::Immortal => (2100, 2399),
        RankTier::Radiant => (2400, 2700),
    }
}

pub fn calculate_mmr_modifier(hidden_mmr: i32, rank: RankTier) -> f32 {
    let (min, max) = get_rank_mmr_range(rank);
    let midpoint = (min + max) as f32 / 2.0;
    let diff = hidden_mmr as f32 - midpoint;
    (1.0 + diff * 0.001).clamp(0.8, 1.2)
}

pub fn apply_penalties(base: i32, mmr_modifier: f32, five_stack: bool) -> i32 {
    let mut adjusted = (base as f32 * mmr_modifier).round() as i32;
    if five_stack {
        adjusted = ((adjusted as f32) * 0.75).round() as i32;
    }
    adjusted.clamp(-30, 42)
}

pub fn estimate_rr_change(input: &MatchInput, rank: RankTier) -> i32 {
    let round_diff = (input.rounds_won - input.rounds_lost).clamp(-13, 13);
    let mmr_modifier = calculate_mmr_modifier(input.hidden_mmr, rank);

    let base_rr = if matches!(rank, RankTier::Immortal | RankTier::Radiant) {
        if input.is_win {
            10 + round_diff
        } else {
            -10 + round_diff
        }
    } else {
        let mut rr = if input.is_win {
            20 + round_diff
        } else {
            -15 + round_diff
        };

        if input.is_win
            && matches!(
                rank,
                RankTier::Iron
                    | RankTier::Bronze
                    | RankTier::Silver
                    | RankTier::Gold
                    | RankTier::Platinum
            )
        {
            if input.acs_percentile > 0.95 {
                rr += 5;
            } else if input.acs_percentile > 0.85 {
                rr += 3;
            } else if input.acs_percentile > 0.70 {
                rr += 1;
            }
        }

        if !input.is_win {
            if input.acs_percentile > 0.85 {
                rr += 2;
            } else if input.acs_percentile < 0.15 {
                rr -= 3;
            }
        }

        rr
    };

    apply_penalties(base_rr, mmr_modifier, input.five_stack_penalty)
}

pub fn deserialize_maps_param<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{Error, SeqAccess, Visitor};
    use std::fmt;

    struct MapsVisitor;
    impl<'de> Visitor<'de> for MapsVisitor {
        type Value = Vec<String>;
        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str(
                "a sequence, repeated query params, or a comma-separated string for maps",
            )
        }
        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut maps = Vec::new();
            while let Some(value) = seq.next_element()? {
                maps.push(value);
            }
            Ok(maps)
        }
        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            // Split on commas and trim whitespace
            Ok(value
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect())
        }
    }
    deserializer.deserialize_any(MapsVisitor)
}
