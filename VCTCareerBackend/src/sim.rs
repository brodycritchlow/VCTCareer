use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub enum Agent {
    Jett,
    Raze,
    Phoenix,
    Breach,
    Sova,
    Sage,
    Omen,
    Brimstone,
    Viper,
    Cypher,
    Killjoy,
    Skye,
    Yoru,
    Astra,
    Kayo,
    Chamber,
    Neon,
    Fade,
    Harbor,
    Gekko,
    Deadlock,
    Iso,
    Clove,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub enum Weapon {
    Classic,
    Shorty,
    Frenzy,
    Ghost,
    Sheriff,
    Stinger,
    Spectre,
    Bucky,
    Judge,
    Bulldog,
    Guardian,
    Phantom,
    Vandal,
    Marshal,
    Operator,
    Ares,
    Odin,
    Knife,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub enum Team {
    Attackers,
    Defenders,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BodyPart {
    Head,
    Body,
    Legs,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArmorType {
    None,
    Light, // 25 armor, costs 400
    Heavy, // 50 armor, costs 1000
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Penetration {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone)]
pub struct WeaponStats {
    pub price: u32,
    pub damage_head: (u32, u32, u32), // no armor, light armor, heavy armor
    pub damage_body: (u32, u32, u32),
    pub damage_legs: (u32, u32, u32),
    pub fire_rate: f32, // rounds per second
    pub penetration: Penetration,
    pub magazine_size: u32,
    pub reload_time_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerLoadout {
    pub primary_weapon: Option<Weapon>,
    pub secondary_weapon: Weapon, // Always have Classic minimum
    pub armor: ArmorType,
    pub abilities_purchased: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerSkills {
    pub aim: f32,
    pub hs: f32,
    pub movement: f32,
    pub util: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Player {
    pub id: u32,
    pub name: String,
    pub agent: Agent,
    pub team: Team,
    pub current_health: u32,
    pub current_armor: u32,
    pub is_alive: bool,
    pub current_credits: u32,
    pub ultimate_points: u32,
    pub current_loadout: PlayerLoadout,

    skills: PlayerSkills,
}

impl Player {
    pub fn new(id: u32, name: String, agent: Agent, team: Team, skills: PlayerSkills) -> Self {
        Player {
            id,
            name,
            agent,
            team,
            current_health: 100,
            current_armor: 0,
            is_alive: true,
            current_credits: 0,
            ultimate_points: 0,
            current_loadout: PlayerLoadout {
                primary_weapon: None,
                secondary_weapon: Weapon::Classic,
                armor: ArmorType::None,
                abilities_purchased: Vec::new(),
            },
            skills,
        }
    }

    pub fn reset_for_round(&mut self) {
        self.current_health = 100;
        // Set armor based on purchased armor type
        self.current_armor = match self.current_loadout.armor {
            ArmorType::None => 0,
            ArmorType::Light => 25,
            ArmorType::Heavy => 50,
        };
        self.is_alive = true;
    }

    pub fn take_damage(&mut self, amount: u32) {
        if !self.is_alive {
            return;
        }

        let total_health = self.current_health + self.current_armor;
        if amount >= total_health {
            self.current_health = 0;
            self.current_armor = 0;
            self.is_alive = false;
        } else if amount > self.current_armor {
            let remaining_damage = amount - self.current_armor;
            self.current_armor = 0;
            self.current_health = self.current_health.saturating_sub(remaining_damage);
        } else {
            self.current_armor = self.current_armor.saturating_sub(amount);
        }
    }

    pub fn survived_round(&self) -> bool {
        self.is_alive
    }
}

pub type Timestamp = u64;

// Helper functions for UUID serialization in schemas
fn serialize_uuid<S>(uuid: &Uuid, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&uuid.to_string())
}

fn deserialize_uuid<'de, D>(deserializer: D) -> Result<Uuid, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Uuid::parse_str(&s).map_err(serde::de::Error::custom)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum SimulationMode {
    Paused,
    Playing,
    FastForward,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum SimulationPhase {
    NotStarted,
    BuyPhase {
        round_number: u8,
    },
    RoundActive {
        round_number: u8,
        spike_planted: bool,
    },
    RoundEnd {
        round_number: u8,
        winner: Team,
    },
    MatchEnd {
        winner: Team,
        final_score: (u8, u8),
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SimulationState {
    #[serde(
        serialize_with = "serialize_uuid",
        deserialize_with = "deserialize_uuid"
    )]
    #[schema(value_type = String)]
    pub id: Uuid,
    pub mode: SimulationMode,
    pub phase: SimulationPhase,
    pub playback_speed: f32,
    pub current_timestamp: Timestamp,
    pub current_round: u8,
    pub attacker_score: u8,
    pub defender_score: u8,
    pub overtime_active: bool,
    pub tick_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EventFilter {
    pub event_types: Option<Vec<String>>,
    pub player_ids: Option<Vec<u32>>,
    pub round_numbers: Option<Vec<u8>>,
    pub start_timestamp: Option<Timestamp>,
    pub end_timestamp: Option<Timestamp>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PlayerStats {
    pub player_id: u32,
    pub kills: u32,
    pub deaths: u32,
    pub assists: u32,
    pub damage_dealt: u32,
    pub headshot_percentage: f32,
    pub credits: u32,
    pub ultimate_points: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum GameEvent {
    MatchStart {
        timestamp: Timestamp,
    },
    MatchEnd {
        timestamp: Timestamp,
        winning_team: Team,
        score_attackers: u8,
        score_defenders: u8,
    },
    BuyPhaseStart {
        timestamp: Timestamp,
        round_number: u8,
    },
    BuyPhaseEnd {
        timestamp: Timestamp,
        round_number: u8,
    },
    RoundStart {
        timestamp: Timestamp,
        round_number: u8,
        attacker_credits_start: u32,
        defender_credits_start: u32,
    },
    RoundEnd {
        timestamp: Timestamp,
        round_number: u8,
        winning_team: Team,
        reason: RoundEndReason,
    },
    Kill {
        timestamp: Timestamp,
        killer_id: u32,
        victim_id: u32,
        weapon: Weapon,
        is_headshot: bool,
    },
    Damage {
        timestamp: Timestamp,
        attacker_id: u32,
        victim_id: u32,
        amount: u32,
        weapon: Weapon,
        is_headshot: bool,
    },
    SpikePlant {
        timestamp: Timestamp,
        planter_id: u32,
    },
    SpikeDefuse {
        timestamp: Timestamp,
        defuser_id: u32,
        successful: bool,
    },
    AbilityUsed {
        timestamp: Timestamp,
        player_id: u32,
        ability_name: String,
    },
    SideSwap {
        timestamp: Timestamp,
        round_number: u8,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum RoundEndReason {
    AllAttackersEliminated,
    AllDefendersEliminated,
    SpikeDetonated,
    SpikeDefused,
    TimeExpired,
}

impl GameEvent {
    pub fn timestamp(&self) -> Timestamp {
        match self {
            GameEvent::MatchStart { timestamp } => *timestamp,
            GameEvent::MatchEnd { timestamp, .. } => *timestamp,
            GameEvent::BuyPhaseStart { timestamp, .. } => *timestamp,
            GameEvent::BuyPhaseEnd { timestamp, .. } => *timestamp,
            GameEvent::RoundStart { timestamp, .. } => *timestamp,
            GameEvent::RoundEnd { timestamp, .. } => *timestamp,
            GameEvent::Kill { timestamp, .. } => *timestamp,
            GameEvent::Damage { timestamp, .. } => *timestamp,
            GameEvent::SpikePlant { timestamp, .. } => *timestamp,
            GameEvent::SpikeDefuse { timestamp, .. } => *timestamp,
            GameEvent::AbilityUsed { timestamp, .. } => *timestamp,
            GameEvent::SideSwap { timestamp, .. } => *timestamp,
        }
    }
}

pub struct ValorantSimulation {
    pub state: SimulationState,
    pub players: HashMap<u32, Player>,
    pub events: Vec<GameEvent>,
    pub loss_streaks: HashMap<Team, u8>,
    pub weapon_stats: HashMap<Weapon, WeaponStats>,

    // New fields for modular control
    pub checkpoints: HashMap<u64, SimulationCheckpoint>,
    pub round_timer_ms: i32,
    pub spike_timer_ms: i32,
    pub spike_planted: bool,
    pub spike_defused: bool,
    pub round_start_timestamp: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationCheckpoint {
    pub state: SimulationState,
    pub players: HashMap<u32, Player>,
    pub events: Vec<GameEvent>,
    pub loss_streaks: HashMap<Team, u8>,
}

#[allow(clippy::new_without_default)]
impl ValorantSimulation {
    pub fn new() -> Self {
        let mut weapon_stats = HashMap::new();

        // Initialize weapon stats based on Valorant data
        weapon_stats.insert(
            Weapon::Classic,
            WeaponStats {
                price: 0,
                damage_head: (78, 66, 26),
                damage_body: (26, 22, 22),
                damage_legs: (22, 18, 18),
                fire_rate: 6.75,
                penetration: Penetration::Low,
                magazine_size: 12,
                reload_time_ms: 2250,
            },
        );

        weapon_stats.insert(
            Weapon::Ghost,
            WeaponStats {
                price: 500,
                damage_head: (105, 87, 30),
                damage_body: (30, 25, 25),
                damage_legs: (25, 21, 21),
                fire_rate: 6.75,
                penetration: Penetration::Medium,
                magazine_size: 15,
                reload_time_ms: 2500,
            },
        );

        weapon_stats.insert(
            Weapon::Sheriff,
            WeaponStats {
                price: 800,
                damage_head: (159, 145, 55),
                damage_body: (55, 50, 46),
                damage_legs: (46, 42, 42),
                fire_rate: 4.0,
                penetration: Penetration::High,
                magazine_size: 6,
                reload_time_ms: 3000,
            },
        );

        weapon_stats.insert(
            Weapon::Spectre,
            WeaponStats {
                price: 1600,
                damage_head: (78, 66, 26),
                damage_body: (26, 22, 22),
                damage_legs: (22, 18, 18),
                fire_rate: 13.33,
                penetration: Penetration::Medium,
                magazine_size: 30,
                reload_time_ms: 2250,
            },
        );

        weapon_stats.insert(
            Weapon::Phantom,
            WeaponStats {
                price: 2900,
                damage_head: (156, 140, 124), // Close range
                damage_body: (39, 35, 31),
                damage_legs: (33, 29, 26),
                fire_rate: 11.0,
                penetration: Penetration::Medium,
                magazine_size: 30,
                reload_time_ms: 2500,
            },
        );

        weapon_stats.insert(
            Weapon::Vandal,
            WeaponStats {
                price: 2900,
                damage_head: (160, 160, 160), // Always 160 regardless of armor
                damage_body: (40, 40, 40),
                damage_legs: (34, 34, 34),
                fire_rate: 9.75,
                penetration: Penetration::Medium,
                magazine_size: 25,
                reload_time_ms: 2500,
            },
        );

        weapon_stats.insert(
            Weapon::Operator,
            WeaponStats {
                price: 4700,
                damage_head: (255, 255, 255),
                damage_body: (150, 150, 150),
                damage_legs: (120, 120, 120),
                fire_rate: 0.75,
                penetration: Penetration::High,
                magazine_size: 5,
                reload_time_ms: 3700,
            },
        );

        let simulation_id = Uuid::new_v4();

        ValorantSimulation {
            state: SimulationState {
                id: simulation_id,
                mode: SimulationMode::Playing,
                phase: SimulationPhase::NotStarted,
                playback_speed: 1.0,
                current_timestamp: 0,
                current_round: 0,
                attacker_score: 0,
                defender_score: 0,
                overtime_active: false,
                tick_count: 0,
            },
            players: HashMap::new(),
            events: Vec::new(),
            loss_streaks: HashMap::new(),
            weapon_stats,
            checkpoints: HashMap::new(),
            round_timer_ms: 100_000,
            spike_timer_ms: 45_000,
            spike_planted: false,
            spike_defused: false,
            round_start_timestamp: 0,
        }
    }

    pub fn add_player(&mut self, player: Player) {
        self.players.insert(player.id, player);
    }

    pub fn advance_time(&mut self, delta_ms: u64) {
        let adjusted_delta = (delta_ms as f32 * self.state.playback_speed) as u64;
        self.state.current_timestamp += adjusted_delta;
        self.state.tick_count += 1;
    }

    // New modular control methods
    pub fn start_simulation(&mut self) {
        if matches!(self.state.phase, SimulationPhase::NotStarted) {
            self.state.mode = SimulationMode::Playing;
            self.state.phase = SimulationPhase::BuyPhase { round_number: 1 };
            self.state.current_round = 1;

            // Initialize players with starting credits
            for player in self.players.values_mut() {
                player.current_credits = 800;
            }

            // Initialize loss streaks
            self.loss_streaks.insert(Team::Attackers, 0);
            self.loss_streaks.insert(Team::Defenders, 0);

            self.record_event(GameEvent::MatchStart {
                timestamp: self.state.current_timestamp,
            });
        }
    }

    pub fn pause_simulation(&mut self) {
        self.state.mode = SimulationMode::Paused;
    }

    pub fn resume_simulation(&mut self) {
        if matches!(self.state.mode, SimulationMode::Paused) {
            self.state.mode = SimulationMode::Playing;
        }
    }

    pub fn set_playback_speed(&mut self, speed: f32) {
        self.state.playback_speed = speed.clamp(0.1, 5.0);
        if speed > 1.0 {
            self.state.mode = SimulationMode::FastForward;
        } else {
            self.state.mode = SimulationMode::Playing;
        }
    }

    pub fn create_checkpoint(&mut self) {
        let checkpoint = SimulationCheckpoint {
            state: self.state.clone(),
            players: self.players.clone(),
            events: self.events.clone(),
            loss_streaks: self.loss_streaks.clone(),
        };
        self.checkpoints.insert(self.state.tick_count, checkpoint);
    }

    pub fn restore_checkpoint(&mut self, tick: u64) -> Result<(), String> {
        if let Some(checkpoint) = self.checkpoints.get(&tick).cloned() {
            self.state = checkpoint.state;
            self.players = checkpoint.players;
            self.events = checkpoint.events;
            self.loss_streaks = checkpoint.loss_streaks;
            Ok(())
        } else {
            Err(format!("Checkpoint not found for tick {}", tick))
        }
    }

    pub fn get_current_state(&self) -> &SimulationState {
        &self.state
    }

    pub fn get_filtered_events(&self, filter: &EventFilter) -> Vec<&GameEvent> {
        self.events
            .iter()
            .filter(|event| {
                if let Some(ref event_types) = filter.event_types {
                    let event_name = match event {
                        GameEvent::MatchStart { .. } => "MatchStart",
                        GameEvent::MatchEnd { .. } => "MatchEnd",
                        GameEvent::BuyPhaseStart { .. } => "BuyPhaseStart",
                        GameEvent::BuyPhaseEnd { .. } => "BuyPhaseEnd",
                        GameEvent::RoundStart { .. } => "RoundStart",
                        GameEvent::RoundEnd { .. } => "RoundEnd",
                        GameEvent::Kill { .. } => "Kill",
                        GameEvent::Damage { .. } => "Damage",
                        GameEvent::SpikePlant { .. } => "SpikePlant",
                        GameEvent::SpikeDefuse { .. } => "SpikeDefuse",
                        GameEvent::AbilityUsed { .. } => "AbilityUsed",
                        GameEvent::SideSwap { .. } => "SideSwap",
                    };
                    if !event_types.contains(&event_name.to_string()) {
                        return false;
                    }
                }

                if let Some(ref start_time) = filter.start_timestamp {
                    let event_time = match event {
                        GameEvent::MatchStart { timestamp } => *timestamp,
                        GameEvent::MatchEnd { timestamp, .. } => *timestamp,
                        GameEvent::BuyPhaseStart { timestamp, .. } => *timestamp,
                        GameEvent::BuyPhaseEnd { timestamp, .. } => *timestamp,
                        GameEvent::RoundStart { timestamp, .. } => *timestamp,
                        GameEvent::RoundEnd { timestamp, .. } => *timestamp,
                        GameEvent::Kill { timestamp, .. } => *timestamp,
                        GameEvent::Damage { timestamp, .. } => *timestamp,
                        GameEvent::SpikePlant { timestamp, .. } => *timestamp,
                        GameEvent::SpikeDefuse { timestamp, .. } => *timestamp,
                        GameEvent::AbilityUsed { timestamp, .. } => *timestamp,
                        GameEvent::SideSwap { timestamp, .. } => *timestamp,
                    };
                    if event_time < *start_time {
                        return false;
                    }
                }

                if let Some(ref end_time) = filter.end_timestamp {
                    let event_time = match event {
                        GameEvent::MatchStart { timestamp } => *timestamp,
                        GameEvent::MatchEnd { timestamp, .. } => *timestamp,
                        GameEvent::BuyPhaseStart { timestamp, .. } => *timestamp,
                        GameEvent::BuyPhaseEnd { timestamp, .. } => *timestamp,
                        GameEvent::RoundStart { timestamp, .. } => *timestamp,
                        GameEvent::RoundEnd { timestamp, .. } => *timestamp,
                        GameEvent::Kill { timestamp, .. } => *timestamp,
                        GameEvent::Damage { timestamp, .. } => *timestamp,
                        GameEvent::SpikePlant { timestamp, .. } => *timestamp,
                        GameEvent::SpikeDefuse { timestamp, .. } => *timestamp,
                        GameEvent::AbilityUsed { timestamp, .. } => *timestamp,
                        GameEvent::SideSwap { timestamp, .. } => *timestamp,
                    };
                    if event_time > *end_time {
                        return false;
                    }
                }

                true
            })
            .collect()
    }

    pub fn get_player_stats(&self) -> Vec<PlayerStats> {
        self.players
            .values()
            .map(|player| {
                let kills = self.events.iter().filter(|e| {
                matches!(e, GameEvent::Kill { killer_id, .. } if *killer_id == player.id)
            }).count() as u32;

                let deaths = self.events.iter().filter(|e| {
                matches!(e, GameEvent::Kill { victim_id, .. } if *victim_id == player.id)
            }).count() as u32;

                let headshot_kills = self
                    .events
                    .iter()
                    .filter(|e| {
                        matches!(e, GameEvent::Kill { killer_id, is_headshot, .. }
                    if *killer_id == player.id && *is_headshot)
                    })
                    .count() as u32;

                let damage_dealt = self
                    .events
                    .iter()
                    .filter_map(|e| match e {
                        GameEvent::Damage {
                            attacker_id,
                            amount,
                            ..
                        } if *attacker_id == player.id => Some(*amount),
                        _ => None,
                    })
                    .sum::<u32>();

                PlayerStats {
                    player_id: player.id,
                    kills,
                    deaths,
                    assists: 0, // TODO: Implement assist tracking
                    damage_dealt,
                    headshot_percentage: if kills > 0 {
                        (headshot_kills as f32 / kills as f32) * 100.0
                    } else {
                        0.0
                    },
                    credits: player.current_credits,
                    ultimate_points: player.ultimate_points,
                }
            })
            .collect()
    }

    fn record_event(&mut self, event: GameEvent) {
        self.events.push(event);
    }

    pub fn advance_tick(&mut self) -> Result<(), String> {
        if matches!(self.state.mode, SimulationMode::Paused) {
            return Ok(());
        }

        match &self.state.phase {
            SimulationPhase::NotStarted => {
                self.start_simulation();
            }
            SimulationPhase::BuyPhase { round_number } => {
                self.advance_buy_phase(*round_number)?;
            }
            SimulationPhase::RoundActive {
                round_number,
                spike_planted,
            } => {
                self.advance_round_active(*round_number, *spike_planted)?;
            }
            SimulationPhase::RoundEnd {
                round_number,
                winner,
            } => {
                self.advance_round_end(*round_number, winner.clone())?;
            }
            SimulationPhase::MatchEnd { .. } => {
                return Ok(()); // Match is over, no more ticks
            }
        }

        self.advance_time(500); // Each tick is 500ms
        Ok(())
    }

    fn advance_buy_phase(&mut self, round_number: u8) -> Result<(), String> {
        // Check if we need to start the buy phase (first time entering this phase)
        let should_start_buy_phase = self.events.is_empty()
            || !matches!(self.events.last(), Some(GameEvent::BuyPhaseStart { .. }))
            || matches!(self.events.last(), Some(GameEvent::RoundEnd { .. }));

        if should_start_buy_phase {
            self.record_event(GameEvent::BuyPhaseStart {
                timestamp: self.state.current_timestamp,
                round_number,
            });

            // Set the buy phase start timestamp
            self.round_start_timestamp = self.state.current_timestamp;

            // Reset players for round
            for player in self.players.values_mut() {
                player.reset_for_round();
            }

            // Handle side swaps
            if round_number == 13 {
                for player in self.players.values_mut() {
                    player.team = match player.team {
                        Team::Attackers => Team::Defenders,
                        Team::Defenders => Team::Attackers,
                    };
                    player.current_credits = 800;
                    player.current_loadout = PlayerLoadout {
                        primary_weapon: None,
                        secondary_weapon: Weapon::Classic,
                        armor: ArmorType::None,
                        abilities_purchased: Vec::new(),
                    };
                }
                self.loss_streaks.insert(Team::Attackers, 0);
                self.loss_streaks.insert(Team::Defenders, 0);

                self.record_event(GameEvent::SideSwap {
                    timestamp: self.state.current_timestamp,
                    round_number,
                });
            }
        }

        // Calculate elapsed time since buy phase started
        let elapsed_time = self.state.current_timestamp - self.round_start_timestamp;

        // Simulate buying logic at the start of buy phase (after 1 second to allow setup)
        if (1000..=1500).contains(&elapsed_time) {
            self.simulate_player_purchases();
        }

        // End buy phase after 30 seconds
        if elapsed_time >= 30_000 {
            self.record_event(GameEvent::BuyPhaseEnd {
                timestamp: self.state.current_timestamp,
                round_number,
            });

            self.state.phase = SimulationPhase::RoundActive {
                round_number,
                spike_planted: false,
            };
            self.spike_planted = false;
            self.spike_defused = false;
            self.round_start_timestamp = self.state.current_timestamp;
            self.round_timer_ms = 100_000;
            self.spike_timer_ms = 45_000;

            self.record_event(GameEvent::RoundStart {
                timestamp: self.state.current_timestamp,
                round_number,
                attacker_credits_start: self
                    .players
                    .values()
                    .find(|p| p.team == Team::Attackers)
                    .map_or(0, |p| p.current_credits),
                defender_credits_start: self
                    .players
                    .values()
                    .find(|p| p.team == Team::Defenders)
                    .map_or(0, |p| p.current_credits),
            });
        }

        Ok(())
    }

    fn advance_round_active(
        &mut self,
        round_number: u8,
        _spike_planted: bool,
    ) -> Result<(), String> {
        self.round_timer_ms = self.round_timer_ms.saturating_sub(500);

        let alive_attackers: Vec<u32> = self
            .get_alive_players_on_team(&Team::Attackers)
            .into_iter()
            .map(|p| p.id)
            .collect();
        let alive_defenders: Vec<u32> = self
            .get_alive_players_on_team(&Team::Defenders)
            .into_iter()
            .map(|p| p.id)
            .collect();

        // Check win conditions
        if alive_attackers.is_empty() {
            self.end_round(
                round_number,
                Team::Defenders,
                RoundEndReason::AllAttackersEliminated,
            );
            return Ok(());
        }
        if alive_defenders.is_empty() {
            if self.spike_planted && !self.spike_defused {
                self.end_round(
                    round_number,
                    Team::Attackers,
                    RoundEndReason::SpikeDetonated,
                );
            } else {
                self.end_round(
                    round_number,
                    Team::Attackers,
                    RoundEndReason::AllDefendersEliminated,
                );
            }
            return Ok(());
        }

        // Spike mechanics
        if !self.spike_planted {
            // 15% chance per tick after 30 seconds to plant spike
            if self.state.current_timestamp - self.round_start_timestamp > 30_000 {
                let mut rng = rand::rng();
                if rng.random::<f32>() < 0.15 {
                    let planter_id = alive_attackers[rng.random_range(0..alive_attackers.len())];
                    self.record_event(GameEvent::SpikePlant {
                        timestamp: self.state.current_timestamp,
                        planter_id,
                    });
                    self.award_spike_plant_bonus(planter_id);
                    self.spike_planted = true;
                    self.state.phase = SimulationPhase::RoundActive {
                        round_number,
                        spike_planted: true,
                    };
                }
            }
        } else {
            self.spike_timer_ms = self.spike_timer_ms.saturating_sub(500);
            if self.spike_timer_ms <= 0 {
                self.end_round(
                    round_number,
                    Team::Attackers,
                    RoundEndReason::SpikeDetonated,
                );
                return Ok(());
            }

            // 5% chance per tick for defuse attempt
            let mut rng = rand::rng();
            if !alive_defenders.is_empty() && rng.random::<f32>() < 0.05 {
                let defuser_id = alive_defenders[rng.random_range(0..alive_defenders.len())];
                self.record_event(GameEvent::SpikeDefuse {
                    timestamp: self.state.current_timestamp,
                    defuser_id,
                    successful: true,
                });
                if let Some(defuser) = self.players.get_mut(&defuser_id) {
                    defuser.ultimate_points += 1;
                }
                self.spike_defused = true;
                self.end_round(round_number, Team::Defenders, RoundEndReason::SpikeDefused);
                return Ok(());
            }
        }

        // Combat simulation
        if !alive_attackers.is_empty() && !alive_defenders.is_empty() {
            self.simulate_combat(&alive_attackers, &alive_defenders);
        }

        // Time expiration
        if !self.spike_planted && self.round_timer_ms <= 0 {
            self.end_round(round_number, Team::Defenders, RoundEndReason::TimeExpired);
        }

        Ok(())
    }

    fn advance_round_end(&mut self, round_number: u8, _winner: Team) -> Result<(), String> {
        // Calculate elapsed time since the round ended
        let round_end_timestamp = match self
            .events
            .iter()
            .rev()
            .find(|e| matches!(e, GameEvent::RoundEnd { .. }))
        {
            Some(GameEvent::RoundEnd { timestamp, .. }) => *timestamp,
            _ => {
                // Fallback: use current timestamp if no RoundEnd event found
                log::warn!(
                    "No RoundEnd event found for round {}, using current timestamp",
                    round_number
                );
                self.state.current_timestamp
            }
        };

        let elapsed_since_round_end = self.state.current_timestamp - round_end_timestamp;

        // Wait 2 seconds before starting next round
        if elapsed_since_round_end >= 2000 {
            if self.check_match_end_conditions() {
                return Ok(());
            }

            // Start next round
            let next_round = round_number + 1;
            self.state.current_round = next_round;
            self.state.phase = SimulationPhase::BuyPhase {
                round_number: next_round,
            };
        }
        Ok(())
    }

    fn end_round(&mut self, round_number: u8, winner: Team, reason: RoundEndReason) {
        // Award round-end credits
        self.calculate_round_rewards(&winner, &reason, self.spike_planted);

        // Update scores
        if winner == Team::Attackers {
            self.state.attacker_score += 1;
        } else {
            self.state.defender_score += 1;
        }

        self.record_event(GameEvent::RoundEnd {
            timestamp: self.state.current_timestamp,
            round_number,
            winning_team: winner.clone(),
            reason,
        });

        self.state.phase = SimulationPhase::RoundEnd {
            round_number,
            winner,
        };
    }

    fn check_match_end_conditions(&mut self) -> bool {
        const WIN_SCORE_REGULAR: u8 = 13;
        const WIN_MARGIN_OVERTIME: u8 = 2;

        let current_diff =
            (self.state.attacker_score as i16 - self.state.defender_score as i16).abs();

        if self.state.overtime_active {
            if current_diff >= WIN_MARGIN_OVERTIME as i16 {
                let winning_team = if self.state.attacker_score > self.state.defender_score {
                    Team::Attackers
                } else {
                    Team::Defenders
                };
                self.record_event(GameEvent::MatchEnd {
                    timestamp: self.state.current_timestamp,
                    winning_team: winning_team.clone(),
                    score_attackers: self.state.attacker_score,
                    score_defenders: self.state.defender_score,
                });
                self.state.phase = SimulationPhase::MatchEnd {
                    winner: winning_team,
                    final_score: (self.state.attacker_score, self.state.defender_score),
                };
                return true;
            }
        } else if self.state.attacker_score >= WIN_SCORE_REGULAR
            || self.state.defender_score >= WIN_SCORE_REGULAR
        {
            if current_diff >= 2 {
                let winning_team = if self.state.attacker_score > self.state.defender_score {
                    Team::Attackers
                } else {
                    Team::Defenders
                };
                self.record_event(GameEvent::MatchEnd {
                    timestamp: self.state.current_timestamp,
                    winning_team: winning_team.clone(),
                    score_attackers: self.state.attacker_score,
                    score_defenders: self.state.defender_score,
                });
                self.state.phase = SimulationPhase::MatchEnd {
                    winner: winning_team,
                    final_score: (self.state.attacker_score, self.state.defender_score),
                };
                return true;
            } else if self.state.attacker_score == 12 && self.state.defender_score == 12 {
                self.state.overtime_active = true;
            }
        }

        false
    }

    pub fn get_alive_players_on_team(&self, team: &Team) -> Vec<&Player> {
        self.players
            .values()
            .filter(|p| p.team == *team && p.is_alive)
            .collect()
    }

    fn calculate_loadout_cost(&self, weapon: &Weapon, armor: &ArmorType) -> u32 {
        let weapon_cost = self.weapon_stats[weapon].price;
        let armor_cost = match armor {
            ArmorType::Heavy => 1000,
            ArmorType::Light => 400,
            ArmorType::None => 0,
        };
        weapon_cost + armor_cost
    }

    fn simulate_player_purchases(&mut self) {
        // Pre-calculate all costs to avoid borrowing conflicts
        let operator_heavy_cost = self.calculate_loadout_cost(&Weapon::Operator, &ArmorType::Heavy);
        let vandal_heavy_cost = self.calculate_loadout_cost(&Weapon::Vandal, &ArmorType::Heavy);
        let spectre_cost = self.calculate_loadout_cost(&Weapon::Spectre, &ArmorType::None);
        let spectre_light_cost = self.calculate_loadout_cost(&Weapon::Spectre, &ArmorType::Light);
        let sheriff_cost = self.calculate_loadout_cost(&Weapon::Sheriff, &ArmorType::None);

        for player in self.players.values_mut() {
            // Reset loadout if they died (don't carry over equipment)
            if !player.survived_round() {
                player.current_loadout = PlayerLoadout {
                    primary_weapon: None,
                    secondary_weapon: Weapon::Classic,
                    armor: ArmorType::None,
                    abilities_purchased: Vec::new(),
                };
            }

            // Dynamic buying strategy with pre-calculated costs
            if player.current_credits >= operator_heavy_cost {
                player.current_loadout.primary_weapon = Some(Weapon::Operator);
                player.current_loadout.armor = ArmorType::Heavy;
                player.current_credits -= operator_heavy_cost;
            } else if player.current_credits >= vandal_heavy_cost {
                player.current_loadout.primary_weapon = Some(Weapon::Vandal);
                player.current_loadout.armor = ArmorType::Heavy;
                player.current_credits -= vandal_heavy_cost;
            } else if player.current_credits >= spectre_light_cost {
                player.current_loadout.primary_weapon = Some(Weapon::Spectre);
                player.current_loadout.armor = ArmorType::Light;
                player.current_credits -= spectre_light_cost;
            } else if player.current_credits >= spectre_cost {
                player.current_loadout.primary_weapon = Some(Weapon::Spectre);
                player.current_credits -= spectre_cost;
            } else if player.current_credits >= sheriff_cost {
                player.current_loadout.secondary_weapon = Weapon::Sheriff;
                player.current_credits -= sheriff_cost;
            }
        }
    }

    fn simulate_combat(&mut self, alive_attackers: &[u32], alive_defenders: &[u32]) {
        // Safety check: ensure both teams have alive players
        if alive_attackers.is_empty() || alive_defenders.is_empty() {
            return;
        }

        let mut rng = rand::rng();

        let attacker_id = alive_attackers[rng.random_range(0..alive_attackers.len())];
        let defender_id = alive_defenders[rng.random_range(0..alive_defenders.len())];

        // Double-check both players are still alive
        let attacker_still_alive = self.players.get(&attacker_id).is_some_and(|p| p.is_alive);
        let defender_still_alive = self.players.get(&defender_id).is_some_and(|p| p.is_alive);

        if !attacker_still_alive || !defender_still_alive {
            return; // Skip combat if either player is dead
        }

        let attacker_player_data = self.players.get(&attacker_id).unwrap().clone();
        let defender_player_data = self.players.get(&defender_id).unwrap().clone();

        // Use equipped weapon for combat effectiveness
        let attacker_weapon = attacker_player_data
            .current_loadout
            .primary_weapon
            .unwrap_or(
                attacker_player_data
                    .current_loadout
                    .secondary_weapon
                    .clone(),
            );
        let defender_weapon = defender_player_data
            .current_loadout
            .primary_weapon
            .unwrap_or(
                defender_player_data
                    .current_loadout
                    .secondary_weapon
                    .clone(),
            );

        // Calculate weapon effectiveness multipliers
        let attacker_weapon_effectiveness = self.calculate_weapon_effectiveness(&attacker_weapon);
        let defender_weapon_effectiveness = self.calculate_weapon_effectiveness(&defender_weapon);

        // Enhanced combat calculation with weapon stats
        let attacker_base_skill =
            attacker_player_data.skills.aim * 0.7 + attacker_player_data.skills.hs * 0.3;
        let defender_base_skill =
            defender_player_data.skills.aim * 0.7 + defender_player_data.skills.hs * 0.3;

        let attacker_effective_skill = attacker_base_skill * attacker_weapon_effectiveness;
        let defender_effective_skill = defender_base_skill * defender_weapon_effectiveness;

        // Fire rate advantage
        let attacker_fire_rate = self.weapon_stats[&attacker_weapon].fire_rate;
        let defender_fire_rate = self.weapon_stats[&defender_weapon].fire_rate;

        let fire_rate_advantage = (attacker_fire_rate / defender_fire_rate).clamp(0.5, 2.0);

        let mut attacker_win_chance =
            0.5 + (attacker_effective_skill - defender_effective_skill) * 0.3;
        attacker_win_chance *= fire_rate_advantage;
        attacker_win_chance = attacker_win_chance.clamp(0.1f32, 0.9f32);

        // Determine hit location and headshot
        let is_attacker_headshot = rng.random::<f32>() < attacker_player_data.skills.hs;
        let is_defender_headshot = rng.random::<f32>() < defender_player_data.skills.hs;

        let hit_body_part = if is_attacker_headshot || is_defender_headshot {
            BodyPart::Head
        } else if rng.random::<f32>() < 0.7 {
            BodyPart::Body
        } else {
            BodyPart::Legs
        };

        // Simulate engagement range (10-50 meters)
        let engagement_range = rng.random_range(10.0..50.0);

        if rng.random::<f32>() < attacker_win_chance {
            // Attacker wins
            let damage = self.calculate_weapon_damage(
                &attacker_weapon,
                &defender_player_data.current_loadout.armor,
                hit_body_part,
                engagement_range,
            );

            if let Some(victim) = self.players.get_mut(&defender_id) {
                victim.take_damage(damage);
            }

            // Only record kill if both killer is alive and victim actually died
            if let (Some(killer), Some(victim)) = (
                self.players.get(&attacker_id),
                self.players.get(&defender_id),
            ) {
                if killer.is_alive && !victim.is_alive {
                    self.record_event(GameEvent::Kill {
                        timestamp: self.state.current_timestamp,
                        killer_id: attacker_id,
                        victim_id: defender_id,
                        weapon: attacker_weapon,
                        is_headshot: is_attacker_headshot,
                    });
                    self.award_kill_bonus(attacker_id);
                }
            }
        } else {
            // Defender wins
            let damage = self.calculate_weapon_damage(
                &defender_weapon,
                &attacker_player_data.current_loadout.armor,
                hit_body_part,
                engagement_range,
            );

            if let Some(victim) = self.players.get_mut(&attacker_id) {
                victim.take_damage(damage);
            }

            // Only record kill if both killer is alive and victim actually died
            if let (Some(killer), Some(victim)) = (
                self.players.get(&defender_id),
                self.players.get(&attacker_id),
            ) {
                if killer.is_alive && !victim.is_alive {
                    self.record_event(GameEvent::Kill {
                        timestamp: self.state.current_timestamp,
                        killer_id: defender_id,
                        victim_id: attacker_id,
                        weapon: defender_weapon,
                        is_headshot: is_defender_headshot,
                    });
                    self.award_kill_bonus(defender_id);
                }
            }
        }
    }

    fn calculate_round_rewards(
        &mut self,
        winning_team: &Team,
        _reason: &RoundEndReason,
        spike_planted: bool,
    ) {
        // Award credits based on Valorant economy system
        for player in self.players.values_mut() {
            let mut credits_earned = 0;

            if player.team == *winning_team {
                // Win reward
                credits_earned += 3000;

                // Reset loss streak for winning team
                self.loss_streaks.insert(player.team.clone(), 0);
            } else {
                // Loss reward with streak bonus
                let loss_streak = self.loss_streaks.get(&player.team).unwrap_or(&0);
                credits_earned += match loss_streak {
                    0 => 1900, // First loss
                    1 => 2400, // Second consecutive loss
                    _ => 2900, // Third+ consecutive loss
                };

                // Update loss streak
                self.loss_streaks
                    .insert(player.team.clone(), loss_streak + 1);

                // Survival bonus (if they survived a lost round)
                if player.survived_round() {
                    credits_earned = credits_earned.min(1000); // Cap at 1000 for survival
                }
            }

            // Spike plant bonus (300 credits per team member)
            if spike_planted && player.team == Team::Attackers {
                credits_earned += 300;
            }

            player.current_credits = (player.current_credits + credits_earned).min(9000);
        }
    }

    fn calculate_weapon_damage(
        &self,
        weapon: &Weapon,
        armor_type: &ArmorType,
        body_part: BodyPart,
        range_meters: f32,
    ) -> u32 {
        let stats = &self.weapon_stats[weapon];

        let base_damage = match body_part {
            BodyPart::Head => match armor_type {
                ArmorType::None => stats.damage_head.0,
                ArmorType::Light => stats.damage_head.1,
                ArmorType::Heavy => stats.damage_head.2,
            },
            BodyPart::Body => match armor_type {
                ArmorType::None => stats.damage_body.0,
                ArmorType::Light => stats.damage_body.1,
                ArmorType::Heavy => stats.damage_body.2,
            },
            BodyPart::Legs => match armor_type {
                ArmorType::None => stats.damage_legs.0,
                ArmorType::Light => stats.damage_legs.1,
                ArmorType::Heavy => stats.damage_legs.2,
            },
        };

        // Apply range penalties (simplified)
        let damage_multiplier = match weapon {
            Weapon::Phantom => {
                if range_meters <= 15.0 {
                    1.0
                } else if range_meters <= 30.0 {
                    0.85
                } else {
                    0.7
                }
            }
            Weapon::Spectre | Weapon::Stinger => {
                if range_meters <= 20.0 {
                    1.0
                } else {
                    0.75
                }
            }
            _ => 1.0, // No damage falloff for most weapons
        };

        (base_damage as f32 * damage_multiplier) as u32
    }

    fn calculate_weapon_effectiveness(&self, weapon: &Weapon) -> f32 {
        match weapon {
            Weapon::Operator => 1.5, // Massive aim advantage
            Weapon::Vandal => 1.2,   // High damage, good accuracy
            Weapon::Phantom => 1.15, // Good balance
            Weapon::Guardian => 1.1, // High damage, slower
            Weapon::Spectre => 0.9,  // Good for close range
            Weapon::Sheriff => 0.8,  // High damage pistol
            Weapon::Ghost => 0.6,    // Balanced pistol
            Weapon::Classic => 0.4,  // Basic weapon
            _ => 0.7,                // Default effectiveness
        }
    }

    fn award_kill_bonus(&mut self, killer_id: u32) {
        if let Some(killer) = self.players.get_mut(&killer_id) {
            killer.current_credits = (killer.current_credits + 200).min(9000);
            killer.ultimate_points += 1; // TODO: Implement proper ult point system
        }
    }

    fn award_spike_plant_bonus(&mut self, planter_id: u32) {
        if let Some(planter) = self.players.get_mut(&planter_id) {
            planter.ultimate_points += 1; // TODO: Implement proper ult point system
        }
    }

    #[allow(dead_code)]
    fn simulate_buy_phase(&mut self) {
        self.record_event(GameEvent::BuyPhaseStart {
            timestamp: self.state.current_timestamp,
            round_number: self.state.current_round,
        });

        // Simple AI buying logic
        for player in self.players.values_mut() {
            // Reset loadout if they died (don't carry over equipment)
            if !player.survived_round() {
                player.current_loadout = PlayerLoadout {
                    primary_weapon: None,
                    secondary_weapon: Weapon::Classic,
                    armor: ArmorType::None,
                    abilities_purchased: Vec::new(),
                };
            }

            // Basic buying strategy
            if player.current_credits >= 5700 {
                // Operator + Heavy armor
                player.current_loadout.primary_weapon = Some(Weapon::Operator);
                player.current_loadout.armor = ArmorType::Heavy;
                player.current_credits -= 5700;
            } else if player.current_credits >= 3900 {
                // Vandal + Heavy armor
                player.current_loadout.primary_weapon = Some(Weapon::Vandal);
                player.current_loadout.armor = ArmorType::Heavy;
                player.current_credits -= 3900;
            } else if player.current_credits >= 1600 {
                // SMG buy
                player.current_loadout.primary_weapon = Some(Weapon::Spectre);
                player.current_credits -= 1600;
                if player.current_credits >= 400 {
                    player.current_loadout.armor = ArmorType::Light;
                    player.current_credits -= 400;
                }
            } else if player.current_credits >= 800 {
                // Pistol upgrade
                player.current_loadout.secondary_weapon = Weapon::Sheriff;
                player.current_credits -= 800;
            }
        }

        self.advance_time(30000); // 30 second buy phase

        self.record_event(GameEvent::BuyPhaseEnd {
            timestamp: self.state.current_timestamp,
            round_number: self.state.current_round,
        });
    }

    // Convenience method for running entire simulation at once (legacy mode)
    pub fn run_simulation_to_completion(&mut self) -> Result<(), String> {
        self.start_simulation();

        let mut tick_count = 0;
        const MAX_TICKS_PER_MATCH: u64 = 50000; // Prevent infinite loops (about 4 hours at 500ms per tick)

        while !matches!(self.state.phase, SimulationPhase::MatchEnd { .. }) {
            tick_count += 1;
            if tick_count > MAX_TICKS_PER_MATCH {
                return Err(format!(
                    "Match simulation exceeded maximum tick limit ({}). Possible infinite loop detected.",
                    MAX_TICKS_PER_MATCH
                ));
            }

            self.advance_tick()?;
        }

        Ok(())
    }

    // High-level control methods for frontend
    pub fn advance_round(&mut self) -> Result<(), String> {
        let mut tick_count = 0;
        const MAX_TICKS_PER_ROUND: u64 = 2000; // Prevent infinite loops (10 minutes at 500ms per tick)

        loop {
            tick_count += 1;
            if tick_count > MAX_TICKS_PER_ROUND {
                return Err(format!(
                    "Round advancement exceeded maximum tick limit ({}). Possible infinite loop detected.",
                    MAX_TICKS_PER_ROUND
                ));
            }

            self.advance_tick()?;
            if matches!(
                self.state.phase,
                SimulationPhase::RoundEnd { .. } | SimulationPhase::MatchEnd { .. }
            ) {
                break;
            }
        }
        Ok(())
    }

    pub fn advance_multiple_ticks(&mut self, count: u32) -> Result<(), String> {
        for _ in 0..count {
            self.advance_tick()?;
            if matches!(self.state.phase, SimulationPhase::MatchEnd { .. }) {
                break;
            }
        }
        Ok(())
    }
}
