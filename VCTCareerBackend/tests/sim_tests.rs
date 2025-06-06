use VCTCareerBackend::sim::{ValorantSimulation, Player, Agent, Team, GameEvent};

#[test]
fn test_dead_players_cannot_get_kills() {
    let mut sim = ValorantSimulation::new();
    
    // Add players with predictable stats to make the test more deterministic
    sim.add_player(Player::new(1, "Attacker1".to_string(), Agent::Jett, Team::Attackers, 0.95, 0.95, 0.95, 0.95));
    sim.add_player(Player::new(2, "Defender1".to_string(), Agent::Omen, Team::Defenders, 0.05, 0.05, 0.05, 0.05));
    
    // Manually set up a round state
    sim.current_round = 1;
    sim.current_timestamp = 1000;
    
    // Simulate one player dying
    if let Some(player) = sim.players.get_mut(&2) {
        player.take_damage(200); // Ensure player dies
        assert!(!player.is_alive, "Player should be dead after taking fatal damage");
    }
    
    // Clear any previous events
    sim.events.clear();
    
    // Now simulate combat between the dead player and alive player
    // This should not result in the dead player getting a kill
    let alive_attackers: Vec<u32> = sim.get_alive_players_on_team(&Team::Attackers).into_iter().map(|p| p.id).collect();
    let alive_defenders: Vec<u32> = sim.get_alive_players_on_team(&Team::Defenders).into_iter().map(|p| p.id).collect();
    
    // Verify that the dead player is not in the alive list
    assert!(!alive_defenders.contains(&2), "Dead player should not be in alive players list");
    assert!(alive_attackers.contains(&1), "Alive player should be in alive players list");
    
    // Now let's test the actual combat logic by running a partial simulation
    // We'll manually trigger the combat logic to see if dead players can get kills
    
    // Run simulation for a short time to see if any events are generated
    let events_before = sim.events.len();
    
    // Try to simulate a few ticks of combat
    for _ in 0..10 {
        let alive_attackers_ids: Vec<u32> = sim.get_alive_players_on_team(&Team::Attackers).into_iter().map(|p| p.id).collect();
        let alive_defenders_ids: Vec<u32> = sim.get_alive_players_on_team(&Team::Defenders).into_iter().map(|p| p.id).collect();
        
        // If no defenders are alive, break
        if alive_defenders_ids.is_empty() {
            break;
        }
        
        // The combat simulation should not generate any kills from dead players
        sim.advance_time(500);
    }
    
    // Check that no kills were recorded by dead players
    for event in &sim.events[events_before..] {
        if let GameEvent::Kill { killer_id, .. } = event {
            let killer = sim.players.get(killer_id).unwrap();
            assert!(killer.is_alive, "Killer with ID {} should be alive when recording a kill", killer_id);
        }
    }
}

#[test]
fn test_simulation_integrity_with_deaths() {
    let mut sim = ValorantSimulation::new();
    
    // Add a full team setup
    for i in 1..=5 {
        sim.add_player(Player::new(i, format!("Attacker{}", i), Agent::Jett, Team::Attackers, 0.7, 0.7, 0.7, 0.7));
    }
    for i in 6..=10 {
        sim.add_player(Player::new(i, format!("Defender{}", i), Agent::Omen, Team::Defenders, 0.7, 0.7, 0.7, 0.7));
    }
    
    // Run a short simulation
    sim.run_simulation();
    
    // Build a map of rounds to track deaths and kills per round
    let mut current_round = 0u8;
    let mut round_deaths: std::collections::HashMap<(u8, u32), u64> = std::collections::HashMap::new(); // (round, player_id) -> death_timestamp
    
    // Verify that no dead players recorded kills within the same round
    for event in &sim.events {
        match event {
            GameEvent::RoundStart { round_number, .. } => {
                current_round = *round_number;
            }
            GameEvent::Kill { killer_id, victim_id, timestamp, .. } => {
                // Record the death of the victim in this round
                round_deaths.insert((current_round, *victim_id), *timestamp);
                
                // Check if the killer had died in this same round before making this kill
                if let Some(killer_death_time) = round_deaths.get(&(current_round, *killer_id)) {
                    assert!(*timestamp < *killer_death_time, 
                        "Player {} recorded a kill at {} after dying at {} in round {}", 
                        killer_id, timestamp, killer_death_time, current_round);
                }
            }
            _ => {}
        }
    }
}

#[test]
fn test_dead_player_exclusion_from_combat() {
    let mut sim = ValorantSimulation::new();
    
    // Add two players
    sim.add_player(Player::new(1, "Alive".to_string(), Agent::Jett, Team::Attackers, 0.8, 0.8, 0.8, 0.8));
    sim.add_player(Player::new(2, "Dead".to_string(), Agent::Omen, Team::Defenders, 0.8, 0.8, 0.8, 0.8));
    
    // Kill one player
    if let Some(player) = sim.players.get_mut(&2) {
        player.take_damage(200);
        assert!(!player.is_alive);
    }
    
    // Test that dead player is not included in alive players list
    let alive_attackers = sim.get_alive_players_on_team(&Team::Attackers);
    let alive_defenders = sim.get_alive_players_on_team(&Team::Defenders);
    
    assert_eq!(alive_attackers.len(), 1);
    assert_eq!(alive_defenders.len(), 0);
    assert_eq!(alive_attackers[0].id, 1);
}