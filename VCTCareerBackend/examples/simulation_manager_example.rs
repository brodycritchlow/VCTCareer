use vctcareer_backend::models::{EventFilterRequest, SimulationPlayer};
use vctcareer_backend::simulation_manager::{
    advance_simulation_legacy, control_simulation_legacy, create_simulation,
    create_simulation_manager, get_simulation_events_legacy, get_simulation_state,
    get_simulation_stats_legacy,
};

fn main() {
    println!("=== VCT Career Simulation Manager Example ===\n");

    // Create a simulation manager
    let manager = create_simulation_manager();
    println!("✓ Simulation manager created");

    // Create players for the simulation
    let players = vec![
        // Attackers
        SimulationPlayer {
            id: 1,
            name: "TenZ".to_string(),
            agent: "Jett".to_string(),
            team: "Attackers".to_string(),
            aim_skill: 0.95,
            hs_skill: 0.85,
            movement_skill: 0.9,
            util_skill: 0.75,
        },
        SimulationPlayer {
            id: 2,
            name: "ShahZaM".to_string(),
            agent: "Sova".to_string(),
            team: "Attackers".to_string(),
            aim_skill: 0.8,
            hs_skill: 0.7,
            movement_skill: 0.75,
            util_skill: 0.95,
        },
        SimulationPlayer {
            id: 3,
            name: "SicK".to_string(),
            agent: "Phoenix".to_string(),
            team: "Attackers".to_string(),
            aim_skill: 0.75,
            hs_skill: 0.65,
            movement_skill: 0.8,
            util_skill: 0.7,
        },
        SimulationPlayer {
            id: 4,
            name: "zombs".to_string(),
            agent: "Omen".to_string(),
            team: "Attackers".to_string(),
            aim_skill: 0.65,
            hs_skill: 0.55,
            movement_skill: 0.6,
            util_skill: 0.85,
        },
        SimulationPlayer {
            id: 5,
            name: "dapr".to_string(),
            agent: "Cypher".to_string(),
            team: "Attackers".to_string(),
            aim_skill: 0.7,
            hs_skill: 0.6,
            movement_skill: 0.65,
            util_skill: 0.9,
        },
        // Defenders
        SimulationPlayer {
            id: 6,
            name: "yay".to_string(),
            agent: "Jett".to_string(),
            team: "Defenders".to_string(),
            aim_skill: 0.98,
            hs_skill: 0.9,
            movement_skill: 0.85,
            util_skill: 0.7,
        },
        SimulationPlayer {
            id: 7,
            name: "FNS".to_string(),
            agent: "Omen".to_string(),
            team: "Defenders".to_string(),
            aim_skill: 0.6,
            hs_skill: 0.5,
            movement_skill: 0.55,
            util_skill: 0.98,
        },
        SimulationPlayer {
            id: 8,
            name: "crashies".to_string(),
            agent: "Sova".to_string(),
            team: "Defenders".to_string(),
            aim_skill: 0.8,
            hs_skill: 0.7,
            movement_skill: 0.75,
            util_skill: 0.9,
        },
        SimulationPlayer {
            id: 9,
            name: "Victor".to_string(),
            agent: "Raze".to_string(),
            team: "Defenders".to_string(),
            aim_skill: 0.85,
            hs_skill: 0.75,
            movement_skill: 0.9,
            util_skill: 0.75,
        },
        SimulationPlayer {
            id: 10,
            name: "Marved".to_string(),
            agent: "Viper".to_string(),
            team: "Defenders".to_string(),
            aim_skill: 0.75,
            hs_skill: 0.65,
            movement_skill: 0.7,
            util_skill: 0.85,
        },
    ];

    // Create the simulation
    let simulation_id = create_simulation(&manager, players).expect("Failed to create simulation");
    println!("✓ Simulation created with ID: {}", simulation_id);

    // Get initial simulation state
    let initial_state =
        get_simulation_state(&manager, &simulation_id).expect("Failed to get initial state");
    println!("✓ Initial simulation state retrieved");
    println!("  - Round: {}", initial_state.current_round);
    println!("  - Phase: {:?}", initial_state.phase);
    println!(
        "  - Score: {} - {}",
        initial_state.attacker_score, initial_state.defender_score
    );
    println!("  - Tick Count: {}", initial_state.tick_count);

    // Demonstrate different simulation advancement modes
    println!("\n=== Advancing Simulation ===");

    // Advance by single tick
    advance_simulation_legacy(&manager, simulation_id.clone(), None, None)
        .expect("Failed to advance tick");
    println!("✓ Advanced by 1 tick");

    // Advance by multiple ticks
    advance_simulation_legacy(
        &manager,
        simulation_id.clone(),
        Some(5),
        Some("tick".to_string()),
    )
    .expect("Failed to advance multiple ticks");
    println!("✓ Advanced by 5 ticks");

    // Advance by more ticks to see progression
    advance_simulation_legacy(
        &manager,
        simulation_id.clone(),
        Some(50),
        Some("tick".to_string()),
    )
    .expect("Failed to advance multiple ticks");
    println!("✓ Advanced by 50 ticks");

    let mid_state =
        get_simulation_state(&manager, &simulation_id).expect("Failed to get mid state");
    println!("  - Current Round: {}", mid_state.current_round);
    println!(
        "  - Current Score: {} - {}",
        mid_state.attacker_score, mid_state.defender_score
    );
    println!("  - Current Phase: {:?}", mid_state.phase);

    // Demonstrate simulation control
    println!("\n=== Simulation Control ===");

    // Pause simulation
    control_simulation_legacy(&manager, simulation_id.clone(), "pause".to_string(), None)
        .expect("Failed to pause");
    println!("✓ Simulation paused");

    // Set playback speed
    control_simulation_legacy(
        &manager,
        simulation_id.clone(),
        "set_speed".to_string(),
        Some(2.0),
    )
    .expect("Failed to set speed");
    println!("✓ Playback speed set to 2.0x");

    // Resume simulation
    control_simulation_legacy(&manager, simulation_id.clone(), "resume".to_string(), None)
        .expect("Failed to resume");
    println!("✓ Simulation resumed");

    // Run more ticks to generate events
    advance_simulation_legacy(
        &manager,
        simulation_id.clone(),
        Some(100),
        Some("tick".to_string()),
    )
    .expect("Failed to advance more ticks");
    println!("✓ Advanced by 100 more ticks for event generation");

    // Get and display filtered events
    println!("\n=== Event Filtering Examples ===");

    // Filter for kill events only
    let kill_filter = EventFilterRequest {
        event_types: Some(vec!["Kill".to_string()]),
        player_ids: None,
        round_numbers: None,
        start_timestamp: None,
        end_timestamp: None,
    };

    let kill_events = get_simulation_events_legacy(&manager, simulation_id.clone(), kill_filter)
        .expect("Failed to get kill events");
    println!("✓ Found {} kill events", kill_events.len());

    // Show first few kill events
    for (i, event) in kill_events.iter().take(3).enumerate() {
        println!("  {}. {:?}", i + 1, event);
    }

    // Filter for specific player events
    let player_filter = EventFilterRequest {
        event_types: None,
        player_ids: Some(vec![1, 6]), // TenZ and yay
        round_numbers: None,
        start_timestamp: None,
        end_timestamp: None,
    };

    let player_events =
        get_simulation_events_legacy(&manager, simulation_id.clone(), player_filter)
            .expect("Failed to get player events");
    println!("✓ Found {} events for TenZ and yay", player_events.len());

    // Filter for specific rounds
    let round_filter = EventFilterRequest {
        event_types: None,
        player_ids: None,
        round_numbers: Some(vec![1, 2, 3]),
        start_timestamp: None,
        end_timestamp: None,
    };

    let round_events = get_simulation_events_legacy(&manager, simulation_id.clone(), round_filter)
        .expect("Failed to get round events");
    println!("✓ Found {} events in rounds 1-3", round_events.len());

    // Get player statistics
    println!("\n=== Player Statistics ===");
    let player_stats = get_simulation_stats_legacy(&manager, simulation_id.clone())
        .expect("Failed to get player stats");

    println!("Player Performance Summary:");
    println!(
        "{:<15} {:<8} {:<8} {:<8} {:<10} {:<8}",
        "Name", "Kills", "Deaths", "Assists", "Damage", "HS%"
    );
    println!("{}", "-".repeat(65));

    for stats in &player_stats {
        println!(
            "{:<15} {:<8} {:<8} {:<8} {:<10} {:<7.1}%",
            format!("Player {}", stats.player_id),
            stats.kills,
            stats.deaths,
            stats.assists,
            stats.damage_dealt,
            stats.headshot_percentage
        );
    }

    // Get current simulation state
    println!("\n=== Current Simulation State ===");
    let final_state =
        get_simulation_state(&manager, &simulation_id).expect("Failed to get final state");
    println!("✓ Simulation state retrieved!");
    println!(
        "  - Current Score: {} - {}",
        final_state.attacker_score, final_state.defender_score
    );
    println!("  - Current Round: {}", final_state.current_round);
    println!("  - Simulation Mode: {:?}", final_state.mode);
    println!("  - Current Phase: {:?}", final_state.phase);
    println!("  - Total Ticks: {}", final_state.tick_count);

    // Get final statistics
    let final_stats =
        get_simulation_stats_legacy(&manager, simulation_id).expect("Failed to get final stats");

    // Find MVP (highest K/D ratio)
    let mvp = final_stats.iter().max_by(|a, b| {
        let a_kd = if a.deaths > 0 {
            a.kills as f32 / a.deaths as f32
        } else {
            a.kills as f32
        };
        let b_kd = if b.deaths > 0 {
            b.kills as f32 / b.deaths as f32
        } else {
            b.kills as f32
        };
        a_kd.partial_cmp(&b_kd).unwrap()
    });

    if let Some(mvp_stats) = mvp {
        let kd_ratio = if mvp_stats.deaths > 0 {
            mvp_stats.kills as f32 / mvp_stats.deaths as f32
        } else {
            mvp_stats.kills as f32
        };
        println!(
            "\n🏆 MVP: Player {} with {:.2} K/D ratio ({} kills, {} deaths)",
            mvp_stats.player_id, kd_ratio, mvp_stats.kills, mvp_stats.deaths
        );
    }

    println!("\n=== Example Complete ===");
}
