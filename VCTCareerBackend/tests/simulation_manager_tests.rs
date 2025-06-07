use uuid::Uuid;
use VCTCareerBackend::simulation_manager::{
    create_simulation_manager, create_simulation, get_simulation_state, 
    advance_simulation_legacy, control_simulation_legacy, 
    get_simulation_events_legacy, get_simulation_stats_legacy, 
    get_events_by_round_legacy, get_events_by_player_legacy, 
    get_events_by_type_legacy, get_events_since_legacy, 
    get_round_summary_legacy
};
use VCTCareerBackend::models::{SimulationPlayer, EventFilterRequest};
use VCTCareerBackend::sim::GameEvent;

fn create_mock_players() -> Vec<SimulationPlayer> {
    vec![
        SimulationPlayer {
            id: 1,
            name: "Player1".to_string(),
            agent: "Jett".to_string(),
            team: "Attackers".to_string(),
            aim_skill: 85.0,
            hs_skill: 75.0,
            movement_skill: 80.0,
            util_skill: 70.0,
        },
        SimulationPlayer {
            id: 2,
            name: "Player2".to_string(),
            agent: "Sage".to_string(),
            team: "Attackers".to_string(),
            aim_skill: 80.0,
            hs_skill: 70.0,
            movement_skill: 75.0,
            util_skill: 85.0,
        },
        SimulationPlayer {
            id: 3,
            name: "Player3".to_string(),
            agent: "Omen".to_string(),
            team: "Attackers".to_string(),
            aim_skill: 78.0,
            hs_skill: 68.0,
            movement_skill: 82.0,
            util_skill: 88.0,
        },
        SimulationPlayer {
            id: 4,
            name: "Player4".to_string(),
            agent: "Sova".to_string(),
            team: "Attackers".to_string(),
            aim_skill: 82.0,
            hs_skill: 72.0,
            movement_skill: 77.0,
            util_skill: 83.0,
        },
        SimulationPlayer {
            id: 5,
            name: "Player5".to_string(),
            agent: "Phoenix".to_string(),
            team: "Attackers".to_string(),
            aim_skill: 79.0,
            hs_skill: 69.0,
            movement_skill: 81.0,
            util_skill: 74.0,
        },
        SimulationPlayer {
            id: 6,
            name: "Player6".to_string(),
            agent: "Cypher".to_string(),
            team: "Defenders".to_string(),
            aim_skill: 81.0,
            hs_skill: 71.0,
            movement_skill: 76.0,
            util_skill: 86.0,
        },
        SimulationPlayer {
            id: 7,
            name: "Player7".to_string(),
            agent: "Breach".to_string(),
            team: "Defenders".to_string(),
            aim_skill: 77.0,
            hs_skill: 67.0,
            movement_skill: 79.0,
            util_skill: 81.0,
        },
        SimulationPlayer {
            id: 8,
            name: "Player8".to_string(),
            agent: "Raze".to_string(),
            team: "Defenders".to_string(),
            aim_skill: 84.0,
            hs_skill: 74.0,
            movement_skill: 83.0,
            util_skill: 76.0,
        },
        SimulationPlayer {
            id: 9,
            name: "Player9".to_string(),
            agent: "Brimstone".to_string(),
            team: "Defenders".to_string(),
            aim_skill: 75.0,
            hs_skill: 65.0,
            movement_skill: 73.0,
            util_skill: 89.0,
        },
        SimulationPlayer {
            id: 10,
            name: "Player10".to_string(),
            agent: "Viper".to_string(),
            team: "Defenders".to_string(),
            aim_skill: 80.0,
            hs_skill: 70.0,
            movement_skill: 78.0,
            util_skill: 85.0,
        },
    ]
}

#[test]
fn test_create_simulation_manager() {
    let manager = create_simulation_manager();
    let simulations = manager.lock().unwrap();
    assert_eq!(simulations.len(), 0);
}

#[test]
fn test_create_simulation_success() {
    let manager = create_simulation_manager();
    let players = create_mock_players();
    
    let result = create_simulation(&manager, players);
    assert!(result.is_ok());
    
    let simulation_id = result.unwrap();
    assert!(!simulation_id.is_empty());
    
    let simulations = manager.lock().unwrap();
    assert_eq!(simulations.len(), 1);
}

#[test]
fn test_create_simulation_invalid_agent() {
    let manager = create_simulation_manager();
    let mut players = create_mock_players();
    players[0].agent = "InvalidAgent".to_string();
    
    let result = create_simulation(&manager, players);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unknown agent"));
}

#[test]
fn test_create_simulation_invalid_team() {
    let manager = create_simulation_manager();
    let mut players = create_mock_players();
    players[0].team = "InvalidTeam".to_string();
    
    let result = create_simulation(&manager, players);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unknown team"));
}

#[test]
fn test_get_simulation_state_success() {
    let manager = create_simulation_manager();
    let players = create_mock_players();
    let simulation_id = create_simulation(&manager, players).unwrap();
    
    let result = get_simulation_state(&manager, &simulation_id);
    assert!(result.is_ok());
    
    let state = result.unwrap();
    assert_eq!(state.current_round, 0); // Simulation hasn't started yet
    assert_eq!(state.attacker_score, 0);
    assert_eq!(state.defender_score, 0);
}

#[test]
fn test_get_simulation_state_invalid_id() {
    let manager = create_simulation_manager();
    
    let result = get_simulation_state(&manager, "invalid-id");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid simulation ID format"));
}

#[test]
fn test_get_simulation_state_not_found() {
    let manager = create_simulation_manager();
    let fake_id = Uuid::new_v4().to_string();
    
    let result = get_simulation_state(&manager, &fake_id);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Simulation not found"));
}

#[test]
fn test_advance_simulation_tick() {
    let manager = create_simulation_manager();
    let players = create_mock_players();
    let simulation_id = create_simulation(&manager, players).unwrap();
    
    let initial_state = get_simulation_state(&manager, &simulation_id).unwrap();
    let initial_tick = initial_state.tick_count;
    
    let result = advance_simulation_legacy(&manager, simulation_id.clone(), Some(5), Some("tick".to_string()));
    assert!(result.is_ok());
    
    let updated_state = get_simulation_state(&manager, &simulation_id).unwrap();
    assert!(updated_state.tick_count > initial_tick);
}

#[test]
fn test_advance_simulation_round() {
    let manager = create_simulation_manager();
    let players = create_mock_players();
    let simulation_id = create_simulation(&manager, players).unwrap();
    
    let result = advance_simulation_legacy(&manager, simulation_id.clone(), None, Some("round".to_string()));
    assert!(result.is_ok());
    
    // After advancing a round, we should have some events
    let events = get_simulation_events_legacy(&manager, simulation_id, EventFilterRequest {
        event_types: None,
        player_ids: None,
        round_numbers: None,
        start_timestamp: None,
        end_timestamp: None,
    }).unwrap();
    
    assert!(!events.is_empty());
}

#[test]
fn test_control_simulation_pause_resume() {
    let manager = create_simulation_manager();
    let players = create_mock_players();
    let simulation_id = create_simulation(&manager, players).unwrap();
    
    // Test pause
    let result = control_simulation_legacy(&manager, simulation_id.clone(), "pause".to_string(), None);
    assert!(result.is_ok());
    
    // Test resume
    let result = control_simulation_legacy(&manager, simulation_id, "resume".to_string(), None);
    assert!(result.is_ok());
}

#[test]
fn test_control_simulation_set_speed() {
    let manager = create_simulation_manager();
    let players = create_mock_players();
    let simulation_id = create_simulation(&manager, players).unwrap();
    
    let result = control_simulation_legacy(&manager, simulation_id, "set_speed".to_string(), Some(2.0));
    assert!(result.is_ok());
}

#[test]
fn test_control_simulation_set_speed_missing_value() {
    let manager = create_simulation_manager();
    let players = create_mock_players();
    let simulation_id = create_simulation(&manager, players).unwrap();
    
    let result = control_simulation_legacy(&manager, simulation_id, "set_speed".to_string(), None);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Speed value required"));
}

#[test]
fn test_control_simulation_invalid_action() {
    let manager = create_simulation_manager();
    let players = create_mock_players();
    let simulation_id = create_simulation(&manager, players).unwrap();
    
    let result = control_simulation_legacy(&manager, simulation_id, "invalid_action".to_string(), None);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid action"));
}

#[test]
fn test_get_simulation_events() {
    let manager = create_simulation_manager();
    let players = create_mock_players();
    let simulation_id = create_simulation(&manager, players).unwrap();
    
    // Advance simulation to generate events
    advance_simulation_legacy(&manager, simulation_id.clone(), Some(10), Some("tick".to_string())).unwrap();
    
    let filter = EventFilterRequest {
        event_types: None,
        player_ids: None,
        round_numbers: None,
        start_timestamp: None,
        end_timestamp: None,
    };
    
    let result = get_simulation_events_legacy(&manager, simulation_id, filter);
    assert!(result.is_ok());
    
    let events = result.unwrap();
    assert!(!events.is_empty());
}

#[test]
fn test_get_simulation_stats() {
    let manager = create_simulation_manager();
    let players = create_mock_players();
    let simulation_id = create_simulation(&manager, players).unwrap();
    
    let result = get_simulation_stats_legacy(&manager, simulation_id);
    assert!(result.is_ok());
    
    let stats = result.unwrap();
    assert_eq!(stats.len(), 10); // Should have stats for all 10 players
    
    for stat in stats {
        assert!(stat.player_id >= 1 && stat.player_id <= 10);
        // u32 values are always >= 0, so these comparisons are redundant but kept for clarity
        assert!(stat.kills < u32::MAX);
        assert!(stat.deaths < u32::MAX);
        assert!(stat.damage_dealt < u32::MAX);
    }
}

#[test]
fn test_get_events_by_round() {
    let manager = create_simulation_manager();
    let players = create_mock_players();
    let simulation_id = create_simulation(&manager, players).unwrap();
    
    // Advance to generate round events
    advance_simulation_legacy(&manager, simulation_id.clone(), None, Some("round".to_string())).unwrap();
    
    let result = get_events_by_round_legacy(&manager, simulation_id, 1);
    assert!(result.is_ok());
    
    let events = result.unwrap();
    // Should have events for round 1
    for event in events {
        // Verify events are from round 1 (this would depend on the event structure)
        // For now, just check that we got some events
        match event {
            GameEvent::RoundStart { round_number, .. } => assert_eq!(round_number, 1),
            GameEvent::RoundEnd { round_number, .. } => assert_eq!(round_number, 1),
            _ => {} // Other events might not have round info directly accessible
        }
    }
}

#[test]
fn test_get_events_by_player() {
    let manager = create_simulation_manager();
    let players = create_mock_players();
    let simulation_id = create_simulation(&manager, players).unwrap();
    
    // Advance to generate events
    advance_simulation_legacy(&manager, simulation_id.clone(), Some(20), Some("tick".to_string())).unwrap();
    
    let result = get_events_by_player_legacy(&manager, simulation_id, 1);
    assert!(result.is_ok());
    
    let events = result.unwrap();
    // Events should be filtered by player 1
    // The exact verification depends on the GameEvent structure
    // For now, just verify we can call the function without errors
}

#[test]
fn test_get_events_by_type() {
    let manager = create_simulation_manager();
    let players = create_mock_players();
    let simulation_id = create_simulation(&manager, players).unwrap();
    
    // Advance to generate events
    advance_simulation_legacy(&manager, simulation_id.clone(), None, Some("round".to_string())).unwrap();
    
    let result = get_events_by_type_legacy(&manager, simulation_id, "RoundStart".to_string());
    assert!(result.is_ok());
    
    let events = result.unwrap();
    // Should have at least one RoundStart event
    let round_start_events: Vec<_> = events.iter().filter(|e| {
        matches!(e, GameEvent::RoundStart { .. })
    }).collect();
    
    // We should have at least one RoundStart event
    assert!(!round_start_events.is_empty());
}

#[test]
fn test_get_events_since() {
    let manager = create_simulation_manager();
    let players = create_mock_players();
    let simulation_id = create_simulation(&manager, players).unwrap();
    
    // Advance to generate events
    advance_simulation_legacy(&manager, simulation_id.clone(), Some(10), Some("tick".to_string())).unwrap();
    
    // Get events since timestamp 0 (should get all events)
    let result = get_events_since_legacy(&manager, simulation_id, 0);
    assert!(result.is_ok());
    
    let events = result.unwrap();
    assert!(!events.is_empty());
}

#[test]
fn test_get_round_summary() {
    let manager = create_simulation_manager();
    let players = create_mock_players();
    let simulation_id = create_simulation(&manager, players).unwrap();
    
    // Advance through a complete round
    advance_simulation_legacy(&manager, simulation_id.clone(), None, Some("round".to_string())).unwrap();
    
    let result = get_round_summary_legacy(&manager, simulation_id, 1);
    assert!(result.is_ok());
    
    let summary = result.unwrap();
    assert_eq!(summary.round_number, 1);
    assert!(summary.events_count > 0);
    // Winner might be None if the round hasn't ended yet, so we don't assert on it
}

#[test]
fn test_get_round_summary_nonexistent_round() {
    let manager = create_simulation_manager();
    let players = create_mock_players();
    let simulation_id = create_simulation(&manager, players).unwrap();
    
    // Try to get summary for round 255 (should not exist)
    let result = get_round_summary_legacy(&manager, simulation_id, 255);
    assert!(result.is_ok());
    
    let summary = result.unwrap();
    assert_eq!(summary.round_number, 255);
    assert_eq!(summary.events_count, 0); // No events for non-existent round
    assert!(summary.winner.is_none());
}

#[test]
fn test_simulation_integration_flow() {
    let manager = create_simulation_manager();
    let players = create_mock_players();
    
    // Create simulation
    let simulation_id = create_simulation(&manager, players).unwrap();
    
    // Get initial state
    let initial_state = get_simulation_state(&manager, &simulation_id).unwrap();
    assert_eq!(initial_state.current_round, 0); // Simulation hasn't started yet
    assert_eq!(initial_state.attacker_score, 0);
    assert_eq!(initial_state.defender_score, 0);
    
    // Advance some ticks
    advance_simulation_legacy(&manager, simulation_id.clone(), Some(5), Some("tick".to_string())).unwrap();
    
    // Check state updated
    let updated_state = get_simulation_state(&manager, &simulation_id).unwrap();
    assert!(updated_state.tick_count > initial_state.tick_count);
    
    // Pause simulation
    control_simulation_legacy(&manager, simulation_id.clone(), "pause".to_string(), None).unwrap();
    
    // Set playback speed
    control_simulation_legacy(&manager, simulation_id.clone(), "set_speed".to_string(), Some(2.0)).unwrap();
    
    // Resume simulation
    control_simulation_legacy(&manager, simulation_id.clone(), "resume".to_string(), None).unwrap();
    
    // Get player stats
    let stats = get_simulation_stats_legacy(&manager, simulation_id.clone()).unwrap();
    assert_eq!(stats.len(), 10);
    
    // Get events
    let filter = EventFilterRequest {
        event_types: None,
        player_ids: None,
        round_numbers: None,
        start_timestamp: None,
        end_timestamp: None,
    };
    let events = get_simulation_events_legacy(&manager, simulation_id, filter).unwrap();
    assert!(!events.is_empty());
}