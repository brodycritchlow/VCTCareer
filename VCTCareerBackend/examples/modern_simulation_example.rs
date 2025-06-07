use VCTCareerBackend::simulation_manager::{
    create_simulation_manager, create_simulation, get_simulation_state, 
    advance_simulation, control_simulation, get_simulation_events, get_simulation_stats,
    get_events_by_round, get_events_by_player, get_events_by_type, get_events_since,
    get_round_summary, get_live_stats, get_scoreboard, get_economy_status,
    create_checkpoint, get_events_at_timestamp, AdvanceMode, SimulationCommand
};
use VCTCareerBackend::models::{SimulationPlayer, EventFilterRequest};
use VCTCareerBackend::sim::GameEvent;

fn main() {
    println!("=== Modern VCT Career Simulation Manager Example ===\n");
    
    // Create a simulation manager using the new approach
    let manager = create_simulation_manager();
    println!("✓ Created simulation manager with safe Arc<Mutex<HashMap>> pattern");
    
    // Create high-skill players for a competitive match
    let players = vec![
        // Team A (Attackers) - Professional level skills
        SimulationPlayer {
            id: 1,
            name: "TenZ".to_string(),
            agent: "Jett".to_string(),
            team: "Attackers".to_string(),
            aim_skill: 0.95,
            hs_skill: 0.88,
            movement_skill: 0.92,
            util_skill: 0.78,
        },
        SimulationPlayer {
            id: 2,
            name: "ShahZaM".to_string(),
            agent: "Sova".to_string(),
            team: "Attackers".to_string(),
            aim_skill: 0.82,
            hs_skill: 0.75,
            movement_skill: 0.80,
            util_skill: 0.95,
        },
        SimulationPlayer {
            id: 3,
            name: "SicK".to_string(),
            agent: "Sage".to_string(),
            team: "Attackers".to_string(),
            aim_skill: 0.78,
            hs_skill: 0.70,
            movement_skill: 0.82,
            util_skill: 0.88,
        },
        SimulationPlayer {
            id: 4,
            name: "zombs".to_string(),
            agent: "Omen".to_string(),
            team: "Attackers".to_string(),
            aim_skill: 0.68,
            hs_skill: 0.58,
            movement_skill: 0.65,
            util_skill: 0.92,
        },
        SimulationPlayer {
            id: 5,
            name: "dapr".to_string(),
            agent: "Cypher".to_string(),
            team: "Attackers".to_string(),
            aim_skill: 0.75,
            hs_skill: 0.68,
            movement_skill: 0.70,
            util_skill: 0.90,
        },
        // Team B (Defenders) - Also professional level
        SimulationPlayer {
            id: 6,
            name: "yay".to_string(),
            agent: "Jett".to_string(),
            team: "Defenders".to_string(),
            aim_skill: 0.98,
            hs_skill: 0.92,
            movement_skill: 0.88,
            util_skill: 0.75,
        },
        SimulationPlayer {
            id: 7,
            name: "FNS".to_string(),
            agent: "Omen".to_string(),
            team: "Defenders".to_string(),
            aim_skill: 0.65,
            hs_skill: 0.55,
            movement_skill: 0.60,
            util_skill: 0.98,
        },
        SimulationPlayer {
            id: 8,
            name: "crashies".to_string(),
            agent: "Sova".to_string(),
            team: "Defenders".to_string(),
            aim_skill: 0.85,
            hs_skill: 0.78,
            movement_skill: 0.80,
            util_skill: 0.92,
        },
        SimulationPlayer {
            id: 9,
            name: "Victor".to_string(),
            agent: "Raze".to_string(),
            team: "Defenders".to_string(),
            aim_skill: 0.88,
            hs_skill: 0.80,
            movement_skill: 0.92,
            util_skill: 0.78,
        },
        SimulationPlayer {
            id: 10,
            name: "Marved".to_string(),
            agent: "Viper".to_string(),
            team: "Defenders".to_string(),
            aim_skill: 0.80,
            hs_skill: 0.72,
            movement_skill: 0.75,
            util_skill: 0.88,
        },
    ];
    
    // Create simulation using new type-safe approach
    let simulation_id = create_simulation(&manager, players).expect("Failed to create simulation");
    println!("✓ Created simulation with ID: {}", simulation_id);
    
    // Demonstrate new type-safe control commands
    println!("\n=== Modern Simulation Control (Type-Safe Enums) ===");
    
    // Pause simulation using enum instead of string
    control_simulation(&manager, &simulation_id, SimulationCommand::Pause)
        .expect("Failed to pause simulation");
    println!("✓ Paused simulation using SimulationCommand::Pause");
    
    // Set playback speed using enum
    control_simulation(&manager, &simulation_id, SimulationCommand::SetSpeed(2.5))
        .expect("Failed to set speed");
    println!("✓ Set playback speed to 2.5x using SimulationCommand::SetSpeed(2.5)");
    
    // Resume simulation
    control_simulation(&manager, &simulation_id, SimulationCommand::Resume)
        .expect("Failed to resume simulation");
    println!("✓ Resumed simulation using SimulationCommand::Resume");
    
    // Demonstrate new type-safe advancement modes
    println!("\n=== Modern Advancement Control (Type-Safe Enums) ===");
    
    // Advance by specific number of ticks using enum
    advance_simulation(&manager, &simulation_id, AdvanceMode::Tick(10))
        .expect("Failed to advance ticks");
    println!("✓ Advanced 10 ticks using AdvanceMode::Tick(10)");
    
    // Advance a full round using enum
    advance_simulation(&manager, &simulation_id, AdvanceMode::Round)
        .expect("Failed to advance round");
    println!("✓ Advanced full round using AdvanceMode::Round");
    
    // Check current state
    let state = get_simulation_state(&manager, &simulation_id).expect("Failed to get state");
    println!("✓ Current state: Round {}, Score: {}-{}, Ticks: {}", 
             state.current_round, state.attacker_score, state.defender_score, state.tick_count);
    
    // Demonstrate Phase 2 features - Real-time statistics
    println!("\n=== Phase 2 Features: Real-time Statistics ===");
    
    // Get live match statistics
    let live_stats = get_live_stats(&manager, &simulation_id).expect("Failed to get live stats");
    println!("✓ Live Stats:");
    println!("  - Match Duration: {} ms", live_stats.match_duration_ms);
    println!("  - Current Phase: {}", live_stats.match_phase);
    println!("  - Top Fraggers:");
    for (i, fragger) in live_stats.top_fraggers.iter().take(3).enumerate() {
        println!("    {}. {} - {} kills, {:.1} K/D, {:.1}% HS", 
                 i + 1, fragger.player_name, fragger.kills, fragger.kd_ratio, fragger.headshot_percentage);
    }
    
    // Get comprehensive scoreboard
    let scoreboard = get_scoreboard(&manager, &simulation_id).expect("Failed to get scoreboard");
    println!("✓ Scoreboard:");
    println!("  - Match Score: {}-{} (Round {})", 
             scoreboard.match_score.attacker_score, 
             scoreboard.match_score.defender_score,
             scoreboard.match_score.current_round);
    println!("  - Player Rankings:");
    for ranking in scoreboard.player_rankings.iter().take(5) {
        println!("    {}. {} - Rating: {:.1}, {}/{}/{} (K/D/A)", 
                 ranking.rank, ranking.player_name, ranking.rating,
                 ranking.kills, ranking.deaths, ranking.assists);
    }
    
    // Get economy analysis
    let economy = get_economy_status(&manager, &simulation_id).expect("Failed to get economy");
    println!("✓ Economy Status:");
    println!("  - Attacker Credits: {} ({})", economy.attacker_average_credits, economy.attacker_buy_strength);
    println!("  - Defender Credits: {} ({})", economy.defender_average_credits, economy.defender_buy_strength);
    println!("  - Loss Streaks: Attackers: {:?}, Defenders: {:?}", 
             economy.loss_streaks.get(&VCTCareerBackend::sim::Team::Attackers),
             economy.loss_streaks.get(&VCTCareerBackend::sim::Team::Defenders));
    
    // Advance more to generate events for filtering
    advance_simulation(&manager, &simulation_id, AdvanceMode::Tick(50))
        .expect("Failed to advance more ticks");
    println!("\n✓ Advanced 50 more ticks to generate events");
    
    // Demonstrate Phase 3 features - Advanced event querying
    println!("\n=== Phase 3 Features: Advanced Event Querying ===");
    
    // Create a checkpoint for replay capabilities
    let checkpoint_id = create_checkpoint(&manager, &simulation_id, Some("Mid-match checkpoint".to_string()))
        .expect("Failed to create checkpoint");
    println!("✓ Created checkpoint: {}", checkpoint_id);
    
    // Time-travel event querying - get events around current timestamp
    let current_time = get_simulation_state(&manager, &simulation_id)
        .expect("Failed to get state").current_timestamp;
    let events_at_time = get_events_at_timestamp(&manager, &simulation_id, current_time, 10000)
        .expect("Failed to get events at timestamp");
    println!("✓ Found {} events around timestamp {} (±5s window)", events_at_time.len(), current_time);
    
    // Demonstrate enhanced event filtering
    println!("\n=== Enhanced Event Filtering & Analysis ===");
    
    // Get events by specific round
    let round1_events = get_events_by_round(&manager, &simulation_id, 1)
        .expect("Failed to get round 1 events");
    println!("✓ Round 1 events: {}", round1_events.len());
    
    // Get events for specific high-skill players (TenZ and yay)
    let star_player_events = get_events_by_player(&manager, &simulation_id, 1)
        .expect("Failed to get TenZ events");
    println!("✓ TenZ (Player 1) events: {}", star_player_events.len());
    
    let yay_events = get_events_by_player(&manager, &simulation_id, 6)
        .expect("Failed to get yay events");
    println!("✓ yay (Player 6) events: {}", yay_events.len());
    
    // Get only kill events to analyze frags
    let kill_events = get_events_by_type(&manager, &simulation_id, "Kill")
        .expect("Failed to get kill events");
    println!("✓ Total kills in match: {}", kill_events.len());
    
    // Analyze kill events
    let mut kill_count = std::collections::HashMap::new();
    for event in &kill_events {
        if let GameEvent::Kill { killer_id, .. } = event {
            *kill_count.entry(*killer_id).or_insert(0) += 1;
        }
    }
    
    println!("✓ Kill distribution:");
    let mut sorted_kills: Vec<_> = kill_count.iter().collect();
    sorted_kills.sort_by(|a, b| b.1.cmp(a.1));
    for (player_id, kills) in sorted_kills.iter().take(5) {
        println!("  - Player {}: {} kills", player_id, kills);
    }
    
    // Get round summary for detailed analysis
    let round_summary = get_round_summary(&manager, &simulation_id, 1)
        .expect("Failed to get round summary");
    println!("✓ Round 1 Summary:");
    println!("  - Winner: {:?}", round_summary.winner);
    println!("  - End Reason: {}", round_summary.end_reason);
    println!("  - Events: {}", round_summary.events_count);
    
    // Get events since a specific timestamp for real-time updates
    let recent_events = get_events_since(&manager, &simulation_id, current_time - 5000)
        .expect("Failed to get recent events");
    println!("✓ Recent events (last 5 seconds): {}", recent_events.len());
    
    // Advanced filtering with multiple criteria
    let complex_filter = EventFilterRequest {
        event_types: Some(vec!["Kill".to_string(), "Damage".to_string()]),
        player_ids: Some(vec![1, 6]), // TenZ and yay only
        round_numbers: None,
        start_timestamp: Some(0),
        end_timestamp: Some(current_time),
    };
    
    let filtered_events = get_simulation_events(&manager, &simulation_id, complex_filter)
        .expect("Failed to get filtered events");
    println!("✓ Kill/Damage events for TenZ and yay: {}", filtered_events.len());
    
    // Final statistics comparison
    println!("\n=== Final Performance Analysis ===");
    let final_stats = get_simulation_stats(&manager, &simulation_id).expect("Failed to get final stats");
    
    println!("Player Performance Summary:");
    println!("{:<15} {:<8} {:<8} {:<8} {:<10} {:<8} {:<8}", "Name", "Kills", "Deaths", "Assists", "Damage", "HS%", "Credits");
    println!("{}", "-".repeat(75));
    
    for stats in &final_stats {
        let player_name = match stats.player_id {
            1 => "TenZ",
            2 => "ShahZaM", 
            3 => "SicK",
            4 => "zombs",
            5 => "dapr",
            6 => "yay",
            7 => "FNS",
            8 => "crashies", 
            9 => "Victor",
            10 => "Marved",
            _ => "Unknown",
        };
        
        println!("{:<15} {:<8} {:<8} {:<8} {:<10} {:<7.1}% {:<8}", 
                 player_name,
                 stats.kills,
                 stats.deaths,
                 stats.assists,
                 stats.damage_dealt,
                 stats.headshot_percentage,
                 stats.credits);
    }
    
    // Get final state
    let final_state = get_simulation_state(&manager, &simulation_id).expect("Failed to get final state");
    println!("\n=== Final Match State ===");
    println!("✓ Match completed successfully!");
    println!("  - Final Score: {} - {}", final_state.attacker_score, final_state.defender_score);
    println!("  - Total Rounds: {}", final_state.current_round);
    println!("  - Total Ticks: {}", final_state.tick_count);
    println!("  - Final Phase: {:?}", final_state.phase);
    println!("  - Simulation Mode: {:?}", final_state.mode);
    
    println!("\n=== Modern Simulation Manager Demo Complete ===");
    println!("✓ Demonstrated all Phase 1, 2, and 3 features");
    println!("✓ Used type-safe enums instead of string literals");
    println!("✓ Safe mutex handling with proper error management");
    println!("✓ Real-time statistics and comprehensive analysis");
    println!("✓ Advanced event filtering and time-travel queries");
    println!("✓ Professional-grade simulation control");
}