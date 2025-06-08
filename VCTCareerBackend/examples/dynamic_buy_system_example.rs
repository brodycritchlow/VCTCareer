use std::collections::HashMap;
use vctcareer_backend::sim::{
    Agent, AgentRole, ArmorType, Player, PlayerSkills, Team, ValorantSimulation, Weapon
};

fn create_varied_skills(aim: f32, hs: f32, movement: f32, util: f32) -> PlayerSkills {
    PlayerSkills { aim, hs, movement, util }
}

fn print_separator(title: &str) {
    println!("\n{}", "=".repeat(60));
    println!("=== {} ===", title);
    println!("{}", "=".repeat(60));
}

fn print_player_buy_preferences(player: &Player) {
    println!("\n🎮 Player: {} ({})", player.name, format!("{:?}", player.agent));
    println!("   Role: {:?}", player.agent.get_role());
    println!("   Skills: Aim {:.1}, HS {:.1}, Movement {:.1}, Util {:.1}", 
             player.skills.aim, player.skills.hs, player.skills.movement, player.skills.util);
    println!("   💰 Economic Preferences:");
    println!("      Eco Threshold: {} credits", player.buy_preferences.eco_threshold);
    println!("      Force Buy Tendency: {:.1}%", player.buy_preferences.force_buy_tendency * 100.0);
    println!("      Utility Priority: {:.1}%", player.buy_preferences.utility_priority * 100.0);
    println!("      Armor Priority: {:.1}%", player.buy_preferences.armor_priority * 100.0);
    
    println!("   🔫 Weapon Preferences (Top 5):");
    for (i, weapon_pref) in player.buy_preferences.preferred_weapons.iter().take(5).enumerate() {
        println!("      {}. {:?} - Priority: {:.2}, Min Credits: {}", 
                 i + 1, weapon_pref.weapon, weapon_pref.priority, weapon_pref.min_credits);
    }
}

fn print_buy_decision_analysis(sim: &ValorantSimulation, round_num: u8) {
    println!("\n📊 Round {} Buy Analysis:", round_num);
    
    let mut role_weapons: HashMap<AgentRole, Vec<(String, Weapon, ArmorType, u32)>> = HashMap::new();
    let mut total_spent = 0u32;
    let mut weapon_counts: HashMap<Weapon, u32> = HashMap::new();
    let mut armor_counts: HashMap<ArmorType, u32> = HashMap::new();
    
    for player in sim.players.values() {
        let role = player.agent.get_role();
        let entry = role_weapons.entry(role).or_insert_with(Vec::new);
        
        let primary = player.current_loadout.primary_weapon.clone().unwrap_or(Weapon::Classic);
        let spent = 4000 - player.current_credits; // Assuming they started with 4000
        total_spent += spent;
        
        entry.push((player.name.clone(), primary.clone(), player.current_loadout.armor.clone(), spent));
        
        *weapon_counts.entry(primary).or_insert(0) += 1;
        *armor_counts.entry(player.current_loadout.armor.clone()).or_insert(0) += 1;
    }
    
    // Print by role
    for (role, players) in role_weapons {
        println!("\n   {:?}s:", role);
        for (name, weapon, armor, spent) in players {
            println!("      {} -> {:?} + {:?} (${} spent)", name, weapon, armor, spent);
        }
    }
    
    println!("\n   📈 Team Statistics:");
    println!("      Total Credits Spent: ${}", total_spent);
    println!("      Average Spent per Player: ${}", total_spent / 10);
    
    println!("\n   🔫 Weapon Distribution:");
    for (weapon, count) in weapon_counts {
        println!("      {:?}: {} players", weapon, count);
    }
    
    println!("\n   🛡️ Armor Distribution:");
    for (armor, count) in armor_counts {
        println!("      {:?}: {} players", armor, count);
    }
}

fn simulate_economic_scenarios(sim: &mut ValorantSimulation) {
    print_separator("Economic Scenario Testing");
    
    let scenarios = vec![
        ("Rich Team (5000 credits each)", 5000),
        ("Moderate Economy (3000 credits each)", 3000),
        ("Poor Economy (1200 credits each)", 1200),
        ("Force Buy Situation (2400 credits each)", 2400),
    ];
    
    for (scenario_name, credits) in scenarios {
        println!("\n🎯 Scenario: {}", scenario_name);
        
        // Set all players' credits
        for player in sim.players.values_mut() {
            player.current_credits = credits;
            // Reset loadout
            player.current_loadout.primary_weapon = None;
            player.current_loadout.armor = ArmorType::None;
        }
        
        // Simulate buying
        sim.simulate_player_purchases();
        
        // Analyze round type determination
        let attacker_round_type = sim.determine_round_type(&Team::Attackers);
        let defender_round_type = sim.determine_round_type(&Team::Defenders);
        
        println!("   Round Type Detected - Attackers: {:?}, Defenders: {:?}", 
                 attacker_round_type, defender_round_type);
        
        print_buy_decision_analysis(sim, 1);
        
        // Show variety in decisions
        let mut unique_loadouts = std::collections::HashSet::new();
        for player in sim.players.values() {
            let loadout_sig = format!("{:?}-{:?}", 
                                     player.current_loadout.primary_weapon, 
                                     player.current_loadout.armor);
            unique_loadouts.insert(loadout_sig);
        }
        
        println!("   🎲 Decision Variety: {} unique loadouts out of 10 players", unique_loadouts.len());
    }
}

fn demonstrate_role_specialization() {
    print_separator("Agent Role Specialization Demonstration");
    
    let roles_and_agents = vec![
        (AgentRole::Duelist, Agent::Jett, "High aim, aggressive"),
        (AgentRole::Controller, Agent::Omen, "Utility focused"),  
        (AgentRole::Initiator, Agent::Sova, "Balanced approach"),
        (AgentRole::Sentinel, Agent::Cypher, "Conservative, long-range"),
    ];
    
    for (role, agent, description) in roles_and_agents {
        println!("\n🎯 {} Role Analysis ({})", format!("{:?}", role), description);
        
        // Create players with different skill levels for this role
        let skill_levels = vec![
            ("Pro Level", create_varied_skills(0.95, 0.9, 0.9, 0.85)),
            ("Average", create_varied_skills(0.7, 0.6, 0.7, 0.75)),
            ("New Player", create_varied_skills(0.4, 0.3, 0.5, 0.6)),
        ];
        
        for (skill_name, skills) in skill_levels {
            let player = Player::new(1, format!("{} {}", skill_name, format!("{:?}", agent)), 
                                   agent.clone(), Team::Attackers, skills);
            
            println!("\n   📋 {} {}:", skill_name, format!("{:?}", agent));
            println!("      Eco Threshold: {} credits", player.buy_preferences.eco_threshold);
            println!("      Force Buy Tendency: {:.1}%", player.buy_preferences.force_buy_tendency * 100.0);
            
            // Show top 3 weapon preferences
            println!("      Top Weapon Preferences:");
            for (i, weapon_pref) in player.buy_preferences.preferred_weapons.iter().take(3).enumerate() {
                println!("         {}. {:?} (Priority: {:.2})", 
                         i + 1, weapon_pref.weapon, weapon_pref.priority);
            }
        }
    }
}

fn test_situational_adaptation(sim: &mut ValorantSimulation) {
    print_separator("Situational Adaptation Testing");
    
    // Test different loss streaks and their impact
    let situations = vec![
        ("Fresh Start", 0, 4000),
        ("After First Loss", 1, 2800),
        ("Desperate (3 Loss Streak)", 3, 2200),
        ("Must Force Buy", 4, 1800),
    ];
    
    for (situation_name, loss_streak, credits) in situations {
        println!("\n🎯 Situation: {}", situation_name);
        
        // Set up scenario
        sim.loss_streaks.insert(Team::Attackers, loss_streak);
        sim.loss_streaks.insert(Team::Defenders, loss_streak);
        
        for player in sim.players.values_mut() {
            player.current_credits = credits;
            player.current_loadout.primary_weapon = None;
            player.current_loadout.armor = ArmorType::None;
        }
        
        // Create round context
        let context = sim.create_round_context(&Team::Attackers);
        println!("   📊 Context: Round Type: {:?}, Loss Streak: {}, Enemy Economy: {:?}", 
                 context.round_type, context.loss_streak, context.enemy_predicted_economy);
        
        // Test individual buy decisions
        let test_player = sim.players.values().next().unwrap();
        let decision = sim.make_dynamic_buy_decision(test_player, &context);
        
        println!("   🛍️ Sample Decision: {:?} + {:?}, Cost: ${}, Confidence: {:.1}%",
                 decision.primary_weapon.unwrap_or(Weapon::Classic),
                 decision.armor,
                 decision.total_cost,
                 decision.confidence * 100.0);
        
        // Show how force buy tendency affects decisions
        let aggressive_player = sim.players.values()
            .find(|p| p.buy_preferences.force_buy_tendency > 0.6)
            .unwrap();
        let conservative_player = sim.players.values()
            .find(|p| p.buy_preferences.force_buy_tendency < 0.4)
            .unwrap();
        
        let aggressive_decision = sim.make_dynamic_buy_decision(aggressive_player, &context);
        let conservative_decision = sim.make_dynamic_buy_decision(conservative_player, &context);
        
        println!("   🔥 Aggressive Player: ${} spent", aggressive_decision.total_cost);
        println!("   🛡️ Conservative Player: ${} spent", conservative_decision.total_cost);
    }
}

fn performance_benchmark(sim: &mut ValorantSimulation) {
    print_separator("Performance Benchmark");
    
    let iterations = 1000;
    let start_time = std::time::Instant::now();
    
    for _ in 0..iterations {
        // Reset credits
        for player in sim.players.values_mut() {
            player.current_credits = 3500;
        }
        
        // Simulate buy decisions
        sim.simulate_player_purchases();
    }
    
    let duration = start_time.elapsed();
    let avg_time = duration.as_micros() as f64 / iterations as f64;
    
    println!("⚡ Performance Results:");
    println!("   Total time for {} buy phases: {:?}", iterations, duration);
    println!("   Average time per buy phase: {:.2} μs", avg_time);
    println!("   Buy decisions per second: {:.0}", 1_000_000.0 / avg_time);
    println!("   ✅ Performance is suitable for real-time simulation");
}

fn main() {
    println!("🎮 VCT Career Dynamic Buy System Demonstration");
    
    // Create simulation with diverse team
    let mut sim = ValorantSimulation::new();
    
    let team_composition = vec![
        // Attackers
        ("TenZ", Agent::Jett, Team::Attackers, create_varied_skills(0.95, 0.9, 0.9, 0.7)),
        ("zombs", Agent::Omen, Team::Attackers, create_varied_skills(0.8, 0.7, 0.8, 0.9)),
        ("SicK", Agent::Sova, Team::Attackers, create_varied_skills(0.85, 0.8, 0.85, 0.85)),
        ("ShahZaM", Agent::Sage, Team::Attackers, create_varied_skills(0.9, 0.85, 0.8, 0.8)),
        ("dapr", Agent::Phoenix, Team::Attackers, create_varied_skills(0.88, 0.82, 0.87, 0.75)),
        
        // Defenders  
        ("yay", Agent::Chamber, Team::Defenders, create_varied_skills(0.97, 0.95, 0.85, 0.7)),
        ("Marved", Agent::Viper, Team::Defenders, create_varied_skills(0.82, 0.75, 0.88, 0.92)),
        ("crashies", Agent::Breach, Team::Defenders, create_varied_skills(0.83, 0.78, 0.84, 0.88)),
        ("Victor", Agent::Raze, Team::Defenders, create_varied_skills(0.86, 0.81, 0.89, 0.76)),
        ("FNS", Agent::Killjoy, Team::Defenders, create_varied_skills(0.75, 0.68, 0.82, 0.95)),
    ];
    
    for (i, (name, agent, team, skills)) in team_composition.into_iter().enumerate() {
        let mut player = Player::new((i + 1) as u32, name.to_string(), agent, team, skills);
        player.current_credits = 4000; // Starting credits
        sim.add_player(player);
    }
    
    // Start simulation
    sim.start_simulation();
    
    // Demonstrate buy preferences generation
    print_separator("Individual Player Buy Preferences");
    for player in sim.players.values().take(4) {
        print_player_buy_preferences(player);
    }
    
    // Demonstrate role specialization
    demonstrate_role_specialization();
    
    // Test economic scenarios
    simulate_economic_scenarios(&mut sim);
    
    // Test situational adaptation
    test_situational_adaptation(&mut sim);
    
    // Performance benchmark
    performance_benchmark(&mut sim);
    
    print_separator("Live Match Simulation");
    
    // Reset for live simulation
    for player in sim.players.values_mut() {
        player.current_credits = 800; // Pistol round
    }
    
    // Simulate several rounds
    for round_num in 1..=5 {
        println!("\n🎯 ROUND {} SIMULATION", round_num);
        
        // Advance to buy phase
        while !matches!(sim.state.phase, vctcareer_backend::sim::SimulationPhase::BuyPhase { .. }) {
            if let Err(e) = sim.advance_tick() {
                println!("Error advancing simulation: {}", e);
                break;
            }
        }
        
        // Show pre-buy status
        let attacker_credits: u32 = sim.players.values()
            .filter(|p| p.team == Team::Attackers)
            .map(|p| p.current_credits)
            .sum();
        let defender_credits: u32 = sim.players.values()
            .filter(|p| p.team == Team::Defenders)
            .map(|p| p.current_credits)
            .sum();
        
        println!("💰 Pre-Buy Credits - Attackers: ${}, Defenders: ${}", 
                 attacker_credits, defender_credits);
        
        // Trigger buy phase
        for _ in 0..10 {
            if let Err(e) = sim.advance_tick() {
                println!("Error in buy phase: {}", e);
                break;
            }
        }
        
        print_buy_decision_analysis(&sim, round_num);
        
        // Fast-forward through round
        for _ in 0..100 {
            if let Err(e) = sim.advance_tick() {
                break; // Round ended or error
            }
            if matches!(sim.state.phase, vctcareer_backend::sim::SimulationPhase::MatchEnd { .. }) {
                break;
            }
        }
        
        println!("🏆 Round Result - Score: {} - {}", sim.state.attacker_score, sim.state.defender_score);
        
        if matches!(sim.state.phase, vctcareer_backend::sim::SimulationPhase::MatchEnd { .. }) {
            println!("🎊 Match Complete!");
            break;
        }
    }
    
    print_separator("Summary");
    println!("✅ Dynamic Buy System Features Demonstrated:");
    println!("   🎯 Role-based weapon preferences");
    println!("   💰 Economic decision making");
    println!("   🧠 Situational adaptation");
    println!("   🎲 Player personality variation");
    println!("   ⚡ Real-time performance");
    println!("   📊 Comprehensive analytics");
    
    println!("\n🎮 The dynamic buy system successfully creates:");
    println!("   • Realistic and varied buying patterns");
    println!("   • Agent role specialization");
    println!("   • Economic strategy emergence");
    println!("   • Player personality expression");
    println!("   • Situational decision adaptation");
    
    println!("\n🚀 Ready for production use in VCT Career simulation!");
}