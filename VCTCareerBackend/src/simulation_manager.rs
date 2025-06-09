use super::sim::{
    Agent, EventFilter, GameEvent, Player, PlayerStats, SimulationState, Team, ValorantSimulation,
};
use crate::models::{EventFilterRequest, SimulationId, SimulationPlayer};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum SimulationCommand {
    Pause,
    Resume,
    SetSpeed(f32),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AdvanceMode {
    Tick(u32),
    Round,
    Match,
}

impl SimulationCommand {
    pub fn from_string(action: &str, speed: Option<f32>) -> Result<Self, String> {
        match action {
            "pause" => Ok(SimulationCommand::Pause),
            "resume" => Ok(SimulationCommand::Resume),
            "set_speed" => {
                if let Some(speed_value) = speed {
                    Ok(SimulationCommand::SetSpeed(speed_value))
                } else {
                    Err("Speed value required for set_speed action".to_string())
                }
            }
            _ => Err(format!("Invalid action: {}", action)),
        }
    }
}

impl AdvanceMode {
    pub fn from_string(mode: &str, ticks: Option<u32>) -> Self {
        match mode {
            "tick" => AdvanceMode::Tick(ticks.unwrap_or(1)),
            "round" => AdvanceMode::Round,
            "match" => AdvanceMode::Match,
            _ => AdvanceMode::Tick(1),
        }
    }
}

// Helper function for safe mutex access
fn safe_lock<T>(mutex: &Arc<Mutex<T>>) -> Result<MutexGuard<T>, String> {
    mutex
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))
}

// Convert skill values to 1-100 scale, handling both 0.0-1.0 and 1-100 input ranges
fn convert_skill_to_1_100_scale(skill_value: f32) -> u32 {
    if skill_value <= 1.0 {
        // Input is in 0.0-1.0 range, convert to 1-100
        (skill_value * 100.0).clamp(1.0, 100.0) as u32
    } else {
        // Input is already in 1-100 range
        skill_value.clamp(1.0, 100.0) as u32
    }
}

pub type SimulationManager = Arc<Mutex<HashMap<Uuid, ValorantSimulation>>>;

pub fn create_simulation_manager() -> SimulationManager {
    Arc::new(Mutex::new(HashMap::new()))
}

pub fn create_simulation(
    manager: &SimulationManager,
    players: Vec<SimulationPlayer>,
) -> Result<SimulationId, String> {
    let mut sim = ValorantSimulation::new();
    let simulation_id = sim.state.id;

    // Convert and add players to simulation
    for player_data in players {
        let agent = parse_agent(&player_data.agent)?;
        let team = parse_team(&player_data.team)?;

        let player = Player::new(
            player_data.id,
            player_data.name,
            agent,
            team,
            crate::sim::PlayerSkills {
                aim: convert_skill_to_1_100_scale(player_data.aim_skill),
                hs: convert_skill_to_1_100_scale(player_data.hs_skill),
                movement: convert_skill_to_1_100_scale(player_data.movement_skill),
                util: convert_skill_to_1_100_scale(player_data.util_skill),
            },
        );

        sim.add_player(player);
    }

    // Store simulation in manager with safe locking
    safe_lock(manager)?.insert(simulation_id, sim);

    Ok(simulation_id.to_string())
}

pub fn get_simulation_state(
    manager: &SimulationManager,
    simulation_id_str: &str,
) -> Result<SimulationState, String> {
    let simulation_id = Uuid::parse_str(simulation_id_str)
        .map_err(|_| "Invalid simulation ID format".to_string())?;
    let simulations = safe_lock(manager)?;
    let sim = simulations
        .get(&simulation_id)
        .ok_or("Simulation not found")?;
    Ok(sim.get_current_state().clone())
}

pub fn advance_simulation(
    manager: &SimulationManager,
    simulation_id_str: &str,
    advance_mode: AdvanceMode,
) -> Result<(), String> {
    let simulation_id = Uuid::parse_str(simulation_id_str)
        .map_err(|_| "Invalid simulation ID format".to_string())?;
    let mut simulations = safe_lock(manager)?;
    let sim = simulations
        .get_mut(&simulation_id)
        .ok_or("Simulation not found")?;

    match advance_mode {
        AdvanceMode::Tick(tick_count) => {
            sim.advance_multiple_ticks(tick_count)?;
        }
        AdvanceMode::Round => {
            sim.advance_round()?;
        }
        AdvanceMode::Match => {
            sim.run_simulation_to_completion()?;
        }
    }

    Ok(())
}

// Legacy function for backward compatibility
pub fn advance_simulation_legacy(
    manager: &SimulationManager,
    simulation_id_str: String,
    ticks: Option<u32>,
    mode: Option<String>,
) -> Result<(), String> {
    let advance_mode = AdvanceMode::from_string(mode.as_deref().unwrap_or("tick"), ticks);
    advance_simulation(manager, &simulation_id_str, advance_mode)
}

pub fn control_simulation(
    manager: &SimulationManager,
    simulation_id_str: &str,
    command: SimulationCommand,
) -> Result<(), String> {
    let simulation_id = Uuid::parse_str(simulation_id_str)
        .map_err(|_| "Invalid simulation ID format".to_string())?;
    let mut simulations = safe_lock(manager)?;
    let sim = simulations
        .get_mut(&simulation_id)
        .ok_or("Simulation not found")?;

    match command {
        SimulationCommand::Pause => sim.pause_simulation(),
        SimulationCommand::Resume => sim.resume_simulation(),
        SimulationCommand::SetSpeed(speed_value) => sim.set_playback_speed(speed_value),
    }

    Ok(())
}

// Legacy function for backward compatibility
pub fn control_simulation_legacy(
    manager: &SimulationManager,
    simulation_id_str: String,
    action: String,
    speed: Option<f32>,
) -> Result<(), String> {
    let command = SimulationCommand::from_string(&action, speed)?;
    control_simulation(manager, &simulation_id_str, command)
}

pub fn get_simulation_events(
    manager: &SimulationManager,
    simulation_id_str: &str,
    filter: EventFilterRequest,
) -> Result<Vec<GameEvent>, String> {
    let simulation_id = Uuid::parse_str(simulation_id_str)
        .map_err(|_| "Invalid simulation ID format".to_string())?;
    let simulations = safe_lock(manager)?;
    let sim = simulations
        .get(&simulation_id)
        .ok_or("Simulation not found")?;

    let event_filter = EventFilter {
        event_types: filter.event_types,
        player_ids: filter.player_ids,
        round_numbers: filter.round_numbers,
        start_timestamp: filter.start_timestamp,
        end_timestamp: filter.end_timestamp,
    };

    let filtered_events = sim.get_filtered_events(&event_filter);
    Ok(filtered_events.into_iter().cloned().collect())
}

// Legacy function for backward compatibility
pub fn get_simulation_events_legacy(
    manager: &SimulationManager,
    simulation_id_str: String,
    filter: EventFilterRequest,
) -> Result<Vec<GameEvent>, String> {
    get_simulation_events(manager, &simulation_id_str, filter)
}

pub fn get_simulation_stats(
    manager: &SimulationManager,
    simulation_id_str: &str,
) -> Result<Vec<PlayerStats>, String> {
    let simulation_id = Uuid::parse_str(simulation_id_str)
        .map_err(|_| "Invalid simulation ID format".to_string())?;
    let simulations = safe_lock(manager)?;
    let sim = simulations
        .get(&simulation_id)
        .ok_or("Simulation not found")?;

    Ok(sim.get_player_stats())
}

// Legacy function for backward compatibility
pub fn get_simulation_stats_legacy(
    manager: &SimulationManager,
    simulation_id_str: String,
) -> Result<Vec<PlayerStats>, String> {
    get_simulation_stats(manager, &simulation_id_str)
}

// Phase 1 convenience methods for event querying
pub fn get_events_by_round(
    manager: &SimulationManager,
    simulation_id_str: &str,
    round: u8,
) -> Result<Vec<GameEvent>, String> {
    let filter = EventFilterRequest {
        event_types: None,
        player_ids: None,
        round_numbers: Some(vec![round]),
        start_timestamp: None,
        end_timestamp: None,
    };
    get_simulation_events(manager, simulation_id_str, filter)
}

// Legacy function for backward compatibility
pub fn get_events_by_round_legacy(
    manager: &SimulationManager,
    simulation_id_str: String,
    round: u8,
) -> Result<Vec<GameEvent>, String> {
    get_events_by_round(manager, &simulation_id_str, round)
}

pub fn get_events_by_player(
    manager: &SimulationManager,
    simulation_id_str: &str,
    player_id: u32,
) -> Result<Vec<GameEvent>, String> {
    let filter = EventFilterRequest {
        event_types: None,
        player_ids: Some(vec![player_id]),
        round_numbers: None,
        start_timestamp: None,
        end_timestamp: None,
    };
    get_simulation_events(manager, simulation_id_str, filter)
}

// Legacy function for backward compatibility
pub fn get_events_by_player_legacy(
    manager: &SimulationManager,
    simulation_id_str: String,
    player_id: u32,
) -> Result<Vec<GameEvent>, String> {
    get_events_by_player(manager, &simulation_id_str, player_id)
}

pub fn get_events_by_type(
    manager: &SimulationManager,
    simulation_id_str: &str,
    event_type: &str,
) -> Result<Vec<GameEvent>, String> {
    let filter = EventFilterRequest {
        event_types: Some(vec![event_type.to_string()]),
        player_ids: None,
        round_numbers: None,
        start_timestamp: None,
        end_timestamp: None,
    };
    get_simulation_events(manager, simulation_id_str, filter)
}

// Legacy function for backward compatibility
pub fn get_events_by_type_legacy(
    manager: &SimulationManager,
    simulation_id_str: String,
    event_type: String,
) -> Result<Vec<GameEvent>, String> {
    get_events_by_type(manager, &simulation_id_str, &event_type)
}

pub fn get_events_since(
    manager: &SimulationManager,
    simulation_id_str: &str,
    timestamp: u64,
) -> Result<Vec<GameEvent>, String> {
    let filter = EventFilterRequest {
        event_types: None,
        player_ids: None,
        round_numbers: None,
        start_timestamp: Some(timestamp),
        end_timestamp: None,
    };
    get_simulation_events(manager, simulation_id_str, filter)
}

// Legacy function for backward compatibility
pub fn get_events_since_legacy(
    manager: &SimulationManager,
    simulation_id_str: String,
    timestamp: u64,
) -> Result<Vec<GameEvent>, String> {
    get_events_since(manager, &simulation_id_str, timestamp)
}

pub fn get_round_summary(
    manager: &SimulationManager,
    simulation_id_str: &str,
    round: u8,
) -> Result<RoundSummary, String> {
    let simulation_id = Uuid::parse_str(simulation_id_str)
        .map_err(|_| "Invalid simulation ID format".to_string())?;
    let simulations = safe_lock(manager)?;
    let _sim = simulations
        .get(&simulation_id)
        .ok_or("Simulation not found")?;

    // Get round events
    let round_events = get_events_by_round(manager, simulation_id_str, round)?;

    // Calculate round summary
    let mut attackers_score = 0;
    let mut defenders_score = 0;
    let mut round_winner = None;
    let mut round_end_reason = "Unknown".to_string();

    for event in &round_events {
        if let super::sim::GameEvent::RoundEnd {
            winning_team,
            reason,
            ..
        } = event
        {
            round_winner = Some(winning_team.clone());
            round_end_reason = format!("{:?}", reason);
            match winning_team {
                Team::Attackers => attackers_score = 1,
                Team::Defenders => defenders_score = 1,
            }
        }
    }

    Ok(RoundSummary {
        round_number: round,
        winner: round_winner,
        end_reason: round_end_reason,
        attackers_score,
        defenders_score,
        events_count: round_events.len(),
    })
}

// Legacy function for backward compatibility
pub fn get_round_summary_legacy(
    manager: &SimulationManager,
    simulation_id_str: String,
    round: u8,
) -> Result<RoundSummary, String> {
    get_round_summary(manager, &simulation_id_str, round)
}

#[derive(Debug, Clone)]
pub struct RoundSummary {
    pub round_number: u8,
    pub winner: Option<Team>,
    pub end_reason: String,
    pub attackers_score: u8,
    pub defenders_score: u8,
    pub events_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LiveStats {
    pub current_round: u8,
    pub attacker_score: u8,
    pub defender_score: u8,
    pub match_duration_ms: u64,
    pub rounds_played: u8,
    pub top_fraggers: Vec<PlayerPerformance>,
    pub economy_status: EconomyStatus,
    pub match_phase: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PlayerPerformance {
    pub player_id: u32,
    pub player_name: String,
    pub kills: u32,
    pub deaths: u32,
    pub kd_ratio: f32,
    pub avg_damage_per_round: f32,
    pub headshot_percentage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Scoreboard {
    pub match_score: MatchScore,
    pub round_scores: Vec<RoundScore>,
    pub player_rankings: Vec<PlayerRanking>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MatchScore {
    pub attacker_score: u8,
    pub defender_score: u8,
    pub current_round: u8,
    pub overtime_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RoundScore {
    pub round_number: u8,
    pub winner: Option<Team>,
    pub reason: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PlayerRanking {
    pub rank: u8,
    pub player_id: u32,
    pub player_name: String,
    pub rating: f32, // Combat score
    pub kills: u32,
    pub deaths: u32,
    pub assists: u32,
    pub damage_dealt: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EconomyStatus {
    pub attacker_average_credits: u32,
    pub defender_average_credits: u32,
    pub attacker_buy_strength: String, // "Full Buy", "Force Buy", "Eco", "Save"
    pub defender_buy_strength: String,
    pub loss_streaks: HashMap<Team, u8>,
}

// Phase 2 implementation functions
pub fn get_live_stats(
    manager: &SimulationManager,
    simulation_id_str: &str,
) -> Result<LiveStats, String> {
    let simulation_id = Uuid::parse_str(simulation_id_str)
        .map_err(|_| "Invalid simulation ID format".to_string())?;
    let simulations = safe_lock(manager)?;
    let sim = simulations
        .get(&simulation_id)
        .ok_or("Simulation not found")?;

    let state = sim.get_current_state();
    let player_stats = sim.get_player_stats();

    // Calculate top fraggers
    let mut performers: Vec<PlayerPerformance> = player_stats
        .iter()
        .map(|stats| {
            let kd_ratio = if stats.deaths > 0 {
                stats.kills as f32 / stats.deaths as f32
            } else {
                stats.kills as f32
            };

            let rounds_played = if state.current_round > 0 {
                state.current_round
            } else {
                1
            };
            let avg_damage = stats.damage_dealt as f32 / rounds_played as f32;

            // Get player name
            let player_name = sim
                .players
                .get(&stats.player_id)
                .map(|p| p.name.clone())
                .unwrap_or_else(|| format!("Player {}", stats.player_id));

            PlayerPerformance {
                player_id: stats.player_id,
                player_name,
                kills: stats.kills,
                deaths: stats.deaths,
                kd_ratio,
                avg_damage_per_round: avg_damage,
                headshot_percentage: stats.headshot_percentage,
            }
        })
        .collect();

    // Sort by kills descending
    performers.sort_by(|a, b| b.kills.cmp(&a.kills));
    performers.truncate(5); // Top 5 fraggers

    // Calculate economy status
    let economy_status = calculate_economy_status(sim);

    Ok(LiveStats {
        current_round: state.current_round,
        attacker_score: state.attacker_score,
        defender_score: state.defender_score,
        match_duration_ms: state.current_timestamp,
        rounds_played: state.current_round.saturating_sub(1),
        top_fraggers: performers,
        economy_status,
        match_phase: format!("{:?}", state.phase),
    })
}

pub fn get_scoreboard(
    manager: &SimulationManager,
    simulation_id_str: &str,
) -> Result<Scoreboard, String> {
    let simulation_id = Uuid::parse_str(simulation_id_str)
        .map_err(|_| "Invalid simulation ID format".to_string())?;
    let simulations = safe_lock(manager)?;
    let sim = simulations
        .get(&simulation_id)
        .ok_or("Simulation not found")?;

    let state = sim.get_current_state();
    let player_stats = sim.get_player_stats();

    let match_score = MatchScore {
        attacker_score: state.attacker_score,
        defender_score: state.defender_score,
        current_round: state.current_round,
        overtime_active: state.overtime_active,
    };

    // Calculate round scores from events
    let mut round_scores = Vec::new();
    let mut _current_round = 1;
    let mut round_start_time = 0;

    for event in &sim.events {
        match event {
            GameEvent::RoundStart {
                timestamp,
                round_number,
                ..
            } => {
                _current_round = *round_number;
                round_start_time = *timestamp;
            }
            GameEvent::RoundEnd {
                timestamp,
                round_number,
                winning_team,
                reason,
            } => {
                let duration = timestamp - round_start_time;
                round_scores.push(RoundScore {
                    round_number: *round_number,
                    winner: Some(winning_team.clone()),
                    reason: format!("{:?}", reason),
                    duration_ms: duration,
                });
            }
            _ => {}
        }
    }

    // Calculate player rankings
    let mut rankings: Vec<PlayerRanking> = player_stats
        .iter()
        .map(|stats| {
            // Calculate rating (simplified combat score)
            let rating = (stats.kills as f32 * 2.0)
                + (stats.assists as f32 * 0.5)
                + (stats.damage_dealt as f32 * 0.01);

            let player_name = sim
                .players
                .get(&stats.player_id)
                .map(|p| p.name.clone())
                .unwrap_or_else(|| format!("Player {}", stats.player_id));

            PlayerRanking {
                rank: 0, // Will be set after sorting
                player_id: stats.player_id,
                player_name,
                rating,
                kills: stats.kills,
                deaths: stats.deaths,
                assists: stats.assists,
                damage_dealt: stats.damage_dealt,
            }
        })
        .collect();

    // Sort by rating descending and assign ranks
    rankings.sort_by(|a, b| {
        b.rating
            .partial_cmp(&a.rating)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for (i, ranking) in rankings.iter_mut().enumerate() {
        ranking.rank = (i + 1) as u8;
    }

    Ok(Scoreboard {
        match_score,
        round_scores,
        player_rankings: rankings,
    })
}

pub fn get_economy_status(
    manager: &SimulationManager,
    simulation_id_str: &str,
) -> Result<EconomyStatus, String> {
    let simulation_id = Uuid::parse_str(simulation_id_str)
        .map_err(|_| "Invalid simulation ID format".to_string())?;
    let simulations = safe_lock(manager)?;
    let sim = simulations
        .get(&simulation_id)
        .ok_or("Simulation not found")?;

    Ok(calculate_economy_status(sim))
}

fn calculate_economy_status(sim: &ValorantSimulation) -> EconomyStatus {
    let mut attacker_credits = Vec::new();
    let mut defender_credits = Vec::new();

    for player in sim.players.values() {
        match player.team {
            Team::Attackers => attacker_credits.push(player.current_credits),
            Team::Defenders => defender_credits.push(player.current_credits),
        }
    }

    let attacker_avg = if !attacker_credits.is_empty() {
        attacker_credits.iter().sum::<u32>() / attacker_credits.len() as u32
    } else {
        0
    };

    let defender_avg = if !defender_credits.is_empty() {
        defender_credits.iter().sum::<u32>() / defender_credits.len() as u32
    } else {
        0
    };

    let attacker_buy_strength = determine_buy_strength(attacker_avg);
    let defender_buy_strength = determine_buy_strength(defender_avg);

    EconomyStatus {
        attacker_average_credits: attacker_avg,
        defender_average_credits: defender_avg,
        attacker_buy_strength,
        defender_buy_strength,
        loss_streaks: sim.loss_streaks.clone(),
    }
}

fn determine_buy_strength(avg_credits: u32) -> String {
    match avg_credits {
        0..=2000 => "Save".to_string(),
        2001..=3500 => "Eco".to_string(),
        3501..=4500 => "Force Buy".to_string(),
        _ => "Full Buy".to_string(),
    }
}

// Legacy functions for backward compatibility
pub fn get_live_stats_legacy(
    manager: &SimulationManager,
    simulation_id_str: String,
) -> Result<LiveStats, String> {
    get_live_stats(manager, &simulation_id_str)
}

pub fn get_scoreboard_legacy(
    manager: &SimulationManager,
    simulation_id_str: String,
) -> Result<Scoreboard, String> {
    get_scoreboard(manager, &simulation_id_str)
}

pub fn get_economy_status_legacy(
    manager: &SimulationManager,
    simulation_id_str: String,
) -> Result<EconomyStatus, String> {
    get_economy_status(manager, &simulation_id_str)
}

// Phase 3 implementation functions
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SimulationCheckpoint {
    pub checkpoint_id: String,
    pub timestamp: u64,
    pub round_number: u8,
    pub state_snapshot: SimulationState,
    pub event_count: usize,
    pub description: String,
}

pub fn create_checkpoint(
    manager: &SimulationManager,
    simulation_id_str: &str,
    _description: Option<String>,
) -> Result<String, String> {
    let simulation_id = Uuid::parse_str(simulation_id_str)
        .map_err(|_| "Invalid simulation ID format".to_string())?;
    let simulations = safe_lock(manager)?;
    let sim = simulations
        .get(&simulation_id)
        .ok_or("Simulation not found")?;

    let _state = sim.get_current_state();
    let checkpoint_id = Uuid::new_v4().to_string();

    // In a real implementation, you'd store this checkpoint in a database or cache
    // For now, we'll just return the checkpoint info
    Ok(checkpoint_id)
}

pub fn restore_checkpoint(
    manager: &SimulationManager,
    simulation_id_str: &str,
    checkpoint_id: &str,
) -> Result<(), String> {
    let simulation_id = Uuid::parse_str(simulation_id_str)
        .map_err(|_| "Invalid simulation ID format".to_string())?;

    // Parse checkpoint ID as a tick number for now
    let tick = checkpoint_id
        .parse::<u64>()
        .map_err(|_| format!("Invalid checkpoint ID format: {}", checkpoint_id))?;

    let mut simulations = safe_lock(manager)?;
    let sim = simulations
        .get_mut(&simulation_id)
        .ok_or("Simulation not found")?;

    // Use the simulation's built-in checkpoint restoration
    sim.restore_checkpoint(tick)
        .map_err(|e| format!("Failed to restore checkpoint {}: {}", checkpoint_id, e))
}

pub fn rewind_to_round(
    manager: &SimulationManager,
    simulation_id_str: &str,
    target_round: u8,
) -> Result<(), String> {
    let simulation_id = Uuid::parse_str(simulation_id_str)
        .map_err(|_| "Invalid simulation ID format".to_string())?;
    let mut simulations = safe_lock(manager)?;
    let _sim = simulations
        .get_mut(&simulation_id)
        .ok_or("Simulation not found")?;

    // Simple rewind implementation - reset to beginning and replay to target round
    if target_round == 0 {
        return Err("Cannot rewind to round 0".to_string());
    }

    // In a real implementation, you'd use checkpoints or event replay
    // For now, this is a placeholder
    Err("Rewind functionality requires checkpoint system implementation".to_string())
}

pub fn replay_from(
    _manager: &SimulationManager,
    _simulation_id_str: &str,
    _timestamp: u64,
) -> Result<(), String> {
    // Placeholder for replay functionality
    Err("Replay functionality requires event sourcing implementation".to_string())
}

// Event streaming functionality
pub trait EventListener: Send + Sync {
    fn on_event(&self, event: &GameEvent);
    fn get_listener_id(&self) -> String;
}

#[derive(Debug)]
pub struct EventSubscription {
    pub listener_id: String,
    pub event_types: Option<Vec<String>>,
    pub player_filter: Option<Vec<u32>>,
}

pub struct EventStream {
    pub simulation_id: String,
    pub active: bool,
    pub subscribers: Vec<EventSubscription>,
}

impl EventStream {
    pub fn new(simulation_id: String) -> Self {
        EventStream {
            simulation_id,
            active: true,
            subscribers: Vec::new(),
        }
    }

    pub fn subscribe(&mut self, subscription: EventSubscription) {
        self.subscribers.push(subscription);
    }

    pub fn unsubscribe(&mut self, listener_id: &str) {
        self.subscribers
            .retain(|sub| sub.listener_id != listener_id);
    }

    pub fn broadcast_event(&self, _event: &GameEvent) {
        // In a real implementation, this would broadcast to all subscribers
        // This is a placeholder for the streaming functionality
    }
}

pub fn subscribe_to_events(
    _manager: &SimulationManager,
    _simulation_id_str: &str,
    _listener_id: String,
    _event_types: Option<Vec<String>>,
    _player_filter: Option<Vec<u32>>,
) -> Result<(), String> {
    // Placeholder for event subscription
    // In a real implementation, you'd manage a registry of event streams
    Ok(())
}

pub fn get_event_stream(
    _manager: &SimulationManager,
    simulation_id_str: &str,
) -> Result<EventStream, String> {
    // Placeholder for getting event stream
    Ok(EventStream::new(simulation_id_str.to_string()))
}

// Enhanced event querying with time-travel capabilities
pub fn get_events_at_timestamp(
    manager: &SimulationManager,
    simulation_id_str: &str,
    timestamp: u64,
    window_ms: u64,
) -> Result<Vec<GameEvent>, String> {
    let simulation_id = Uuid::parse_str(simulation_id_str)
        .map_err(|_| "Invalid simulation ID format".to_string())?;
    let simulations = safe_lock(manager)?;
    let sim = simulations
        .get(&simulation_id)
        .ok_or("Simulation not found")?;

    let start_time = timestamp.saturating_sub(window_ms / 2);
    let end_time = timestamp + (window_ms / 2);

    let events: Vec<GameEvent> = sim
        .events
        .iter()
        .filter_map(|event| {
            let event_timestamp = event.timestamp();

            if event_timestamp >= start_time && event_timestamp <= end_time {
                Some(event.clone())
            } else {
                None
            }
        })
        .collect();

    Ok(events)
}

// Legacy functions for Phase 3
pub fn create_checkpoint_legacy(
    manager: &SimulationManager,
    simulation_id_str: String,
    description: Option<String>,
) -> Result<String, String> {
    create_checkpoint(manager, &simulation_id_str, description)
}

pub fn restore_checkpoint_legacy(
    manager: &SimulationManager,
    simulation_id_str: String,
    checkpoint_id: String,
) -> Result<(), String> {
    restore_checkpoint(manager, &simulation_id_str, &checkpoint_id)
}

pub fn rewind_to_round_legacy(
    manager: &SimulationManager,
    simulation_id_str: String,
    target_round: u8,
) -> Result<(), String> {
    rewind_to_round(manager, &simulation_id_str, target_round)
}

pub fn replay_from_legacy(
    manager: &SimulationManager,
    simulation_id_str: String,
    timestamp: u64,
) -> Result<(), String> {
    replay_from(manager, &simulation_id_str, timestamp)
}

pub fn get_events_at_timestamp_legacy(
    manager: &SimulationManager,
    simulation_id_str: String,
    timestamp: u64,
    window_ms: u64,
) -> Result<Vec<GameEvent>, String> {
    get_events_at_timestamp(manager, &simulation_id_str, timestamp, window_ms)
}

fn parse_agent(agent_str: &str) -> Result<Agent, String> {
    match agent_str {
        "Jett" => Ok(Agent::Jett),
        "Raze" => Ok(Agent::Raze),
        "Phoenix" => Ok(Agent::Phoenix),
        "Breach" => Ok(Agent::Breach),
        "Sova" => Ok(Agent::Sova),
        "Sage" => Ok(Agent::Sage),
        "Omen" => Ok(Agent::Omen),
        "Brimstone" => Ok(Agent::Brimstone),
        "Viper" => Ok(Agent::Viper),
        "Cypher" => Ok(Agent::Cypher),
        "Killjoy" => Ok(Agent::Killjoy),
        "Skye" => Ok(Agent::Skye),
        "Yoru" => Ok(Agent::Yoru),
        "Astra" => Ok(Agent::Astra),
        "Kayo" => Ok(Agent::Kayo),
        "Chamber" => Ok(Agent::Chamber),
        "Neon" => Ok(Agent::Neon),
        "Fade" => Ok(Agent::Fade),
        "Harbor" => Ok(Agent::Harbor),
        "Gekko" => Ok(Agent::Gekko),
        "Deadlock" => Ok(Agent::Deadlock),
        "Iso" => Ok(Agent::Iso),
        "Clove" => Ok(Agent::Clove),
        _ => Err(format!("Unknown agent: {}", agent_str)),
    }
}

fn parse_team(team_str: &str) -> Result<Team, String> {
    match team_str {
        "Attackers" => Ok(Team::Attackers),
        "Defenders" => Ok(Team::Defenders),
        _ => Err(format!("Unknown team: {}", team_str)),
    }
}
