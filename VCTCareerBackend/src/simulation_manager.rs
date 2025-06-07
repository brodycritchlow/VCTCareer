use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use super::sim::{ValorantSimulation, Player, Agent, Team, EventFilter, SimulationState, PlayerStats, GameEvent};
use crate::models::{SimulationPlayer, EventFilterRequest, SimulationId};

pub type SimulationManager = Arc<Mutex<HashMap<Uuid, ValorantSimulation>>>;

pub fn create_simulation_manager() -> SimulationManager {
    Arc::new(Mutex::new(HashMap::new()))
}

pub fn create_simulation(
    manager: &SimulationManager, 
    players: Vec<SimulationPlayer>
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
            player_data.aim_skill,
            player_data.hs_skill,
            player_data.movement_skill,
            player_data.util_skill,
        );
        
        sim.add_player(player);
    }
    
    // Store simulation in manager
    manager.lock().unwrap().insert(simulation_id, sim);
    
    Ok(simulation_id.to_string())
}

pub fn get_simulation_state(
    manager: &SimulationManager,
    simulation_id_str: String
) -> Result<SimulationState, String> {
    let simulation_id = Uuid::parse_str(&simulation_id_str)
        .map_err(|_| "Invalid simulation ID format".to_string())?;
    let simulations = manager.lock().unwrap();
    let sim = simulations.get(&simulation_id)
        .ok_or("Simulation not found")?;
    Ok(sim.get_current_state().clone())
}

pub fn advance_simulation(
    manager: &SimulationManager,
    simulation_id_str: String,
    ticks: Option<u32>,
    mode: Option<String>
) -> Result<(), String> {
    let simulation_id = Uuid::parse_str(&simulation_id_str)
        .map_err(|_| "Invalid simulation ID format".to_string())?;
    let mut simulations = manager.lock().unwrap();
    let sim = simulations.get_mut(&simulation_id)
        .ok_or("Simulation not found")?;
    
    match mode.as_deref() {
        Some("tick") => {
            let tick_count = ticks.unwrap_or(1);
            sim.advance_multiple_ticks(tick_count)?;
        }
        Some("round") => {
            sim.advance_round()?;
        }
        Some("match") => {
            sim.run_simulation_to_completion()?;
        }
        _ => {
            sim.advance_tick()?;
        }
    }
    
    Ok(())
}

pub fn control_simulation(
    manager: &SimulationManager,
    simulation_id_str: String,
    action: String,
    speed: Option<f32>
) -> Result<(), String> {
    let simulation_id = Uuid::parse_str(&simulation_id_str)
        .map_err(|_| "Invalid simulation ID format".to_string())?;
    let mut simulations = manager.lock().unwrap();
    let sim = simulations.get_mut(&simulation_id)
        .ok_or("Simulation not found")?;
    
    match action.as_str() {
        "pause" => sim.pause_simulation(),
        "resume" => sim.resume_simulation(),
        "set_speed" => {
            if let Some(speed_value) = speed {
                sim.set_playback_speed(speed_value);
            } else {
                return Err("Speed value required for set_speed action".to_string());
            }
        }
        _ => return Err("Invalid action".to_string()),
    }
    
    Ok(())
}

pub fn get_simulation_events(
    manager: &SimulationManager,
    simulation_id_str: String,
    filter: EventFilterRequest
) -> Result<Vec<GameEvent>, String> {
    let simulation_id = Uuid::parse_str(&simulation_id_str)
        .map_err(|_| "Invalid simulation ID format".to_string())?;
    let simulations = manager.lock().unwrap();
    let sim = simulations.get(&simulation_id)
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

pub fn get_simulation_stats(
    manager: &SimulationManager,
    simulation_id_str: String
) -> Result<Vec<PlayerStats>, String> {
    let simulation_id = Uuid::parse_str(&simulation_id_str)
        .map_err(|_| "Invalid simulation ID format".to_string())?;
    let simulations = manager.lock().unwrap();
    let sim = simulations.get(&simulation_id)
        .ok_or("Simulation not found")?;
    
    Ok(sim.get_player_stats())
}

// Phase 1 convenience methods for event querying
pub fn get_events_by_round(
    manager: &SimulationManager,
    simulation_id_str: String,
    round: u8
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

pub fn get_events_by_player(
    manager: &SimulationManager,
    simulation_id_str: String,
    player_id: u32
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

pub fn get_events_by_type(
    manager: &SimulationManager,
    simulation_id_str: String,
    event_type: String
) -> Result<Vec<GameEvent>, String> {
    let filter = EventFilterRequest {
        event_types: Some(vec![event_type]),
        player_ids: None,
        round_numbers: None,
        start_timestamp: None,
        end_timestamp: None,
    };
    get_simulation_events(manager, simulation_id_str, filter)
}

pub fn get_events_since(
    manager: &SimulationManager,
    simulation_id_str: String,
    timestamp: u64
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

pub fn get_round_summary(
    manager: &SimulationManager,
    simulation_id_str: String,
    round: u8
) -> Result<RoundSummary, String> {
    let simulation_id = Uuid::parse_str(&simulation_id_str)
        .map_err(|_| "Invalid simulation ID format".to_string())?;
    let simulations = manager.lock().unwrap();
    let sim = simulations.get(&simulation_id)
        .ok_or("Simulation not found")?;
    
    // Get round events
    let round_events = get_events_by_round(manager, simulation_id_str.clone(), round)?;
    
    // Calculate round summary
    let mut attackers_score = 0;
    let mut defenders_score = 0;
    let mut round_winner = None;
    let mut round_end_reason = "Unknown".to_string();
    
    for event in &round_events {
        if let super::sim::GameEvent::RoundEnd { winning_team, reason, .. } = event {
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

#[derive(Debug, Clone)]
pub struct RoundSummary {
    pub round_number: u8,
    pub winner: Option<Team>,
    pub end_reason: String,
    pub attackers_score: u8,
    pub defenders_score: u8,
    pub events_count: usize,
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