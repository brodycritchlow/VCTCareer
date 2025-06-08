use std::collections::HashMap;
use vctcareer_backend::{
    Agent, AgentRole, Player, PlayerSkills, Team, ValorantSimulation, 
    RoundType, RoundContext, EconomyState, RoundEndReason
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧠 VCT Career Neural Buy System Example");
    println!("========================================");
    
    // Initialize simulation with neural networks
    let mut simulation = ValorantSimulation::new();
    
    // Create diverse team compositions
    create_sample_teams(&mut simulation);
    
    // Enable neural learning for all players
    println!("\n🔧 Enabling neural networks for all players...");
    let errors = simulation.enable_neural_learning_for_all_players();
    if !errors.is_empty() {
        println!("⚠️  Some players failed to initialize neural networks:");
        for error in &errors {
            println!("   {}", error);
        }
    } else {
        println!("✅ All players successfully initialized with neural networks!");
    }
    
    // Also enable traditional ML for comparison
    simulation.enable_ml_for_all_players();
    
    // Simulate multiple rounds to generate training data
    println!("\n🎮 Simulating multiple rounds for neural network training...");
    
    for round in 1..=15 {
        println!("\n--- Round {} ---", round);
        
        // Create different round contexts for variety
        let context = create_round_context(round);
        println!("Round Type: {:?}, Economy: {}", context.round_type, context.team_economy);
        
        // Test neural buy decisions vs traditional decisions
        compare_buy_decisions(&mut simulation, &context, round);
        
        // Simulate the round performance and record results
        simulate_round_performance(&mut simulation, round);
        
        // Train neural networks on the collected data
        if round > 3 { // Start training after some data is collected
            let training_losses = simulation.train_neural_networks();
            print_training_progress(&training_losses, round);
        }
        
        // Show learning insights every 5 rounds
        if round % 5 == 0 {
            show_learning_insights(&simulation, round);
        }
    }
    
    // Final comparison and analysis
    println!("\n📊 Final Analysis");
    println!("=================");
    
    // Show final learning insights for each player
    for player_id in 1..=10 {
        if let Some(insights) = simulation.get_player_learning_insights(player_id) {
            println!("\n🧠 Player {} Neural Learning Summary:", player_id);
            println!("   Total rounds analyzed: {}", insights.total_rounds_analyzed);
            println!("   Current playstyle: {:?}", insights.current_playstyle);
            println!("   Recent avg impact: {:.2}", insights.recent_avg_impact);
            println!("   Learning trend: {}", insights.learning_trend);
            println!("   Confidence score: {:.2}", insights.confidence_score);
            if let Some(best_weapon) = &insights.most_successful_weapon {
                println!("   Most successful weapon: {:?}", best_weapon);
            }
        }
    }
    
    // Demonstrate prediction capabilities
    demonstrate_prediction_capabilities(&mut simulation)?;
    
    println!("\n🎉 Neural network example completed successfully!");
    println!("💡 The neural networks have learned from gameplay and can now make intelligent buy decisions!");
    
    Ok(())
}

fn create_sample_teams(simulation: &mut ValorantSimulation) {
    println!("👥 Creating sample teams with diverse roles...");
    
    // Team Attackers - Mixed skill levels
    let attackers = vec![
        (1, "Neo", Agent::Jett, PlayerSkills { aim: 0.9, hs: 0.8, movement: 0.9, util: 0.4 }),
        (2, "Sova_Main", Agent::Sova, PlayerSkills { aim: 0.7, hs: 0.6, movement: 0.6, util: 0.9 }),
        (3, "Smoke_King", Agent::Omen, PlayerSkills { aim: 0.6, hs: 0.5, movement: 0.5, util: 0.9 }),
        (4, "Site_Anchor", Agent::Cypher, PlayerSkills { aim: 0.8, hs: 0.7, movement: 0.4, util: 0.8 }),
        (5, "Entry_God", Agent::Phoenix, PlayerSkills { aim: 0.85, hs: 0.75, movement: 0.8, util: 0.6 }),
    ];
    
    // Team Defenders - Different skill distribution
    let defenders = vec![
        (6, "OpBot", Agent::Jett, PlayerSkills { aim: 0.95, hs: 0.9, movement: 0.7, util: 0.3 }),
        (7, "Info_Daddy", Agent::Breach, PlayerSkills { aim: 0.65, hs: 0.55, movement: 0.6, util: 0.95 }),
        (8, "Wall_Wizard", Agent::Sage, PlayerSkills { aim: 0.7, hs: 0.6, movement: 0.5, util: 0.85 }),
        (9, "Trap_Master", Agent::Killjoy, PlayerSkills { aim: 0.75, hs: 0.65, movement: 0.4, util: 0.9 }),
        (10, "Flex_Player", Agent::Skye, PlayerSkills { aim: 0.8, hs: 0.7, movement: 0.7, util: 0.8 }),
    ];
    
    // Add players to simulation
    for (id, name, agent, skills) in attackers {
        let player = Player::new(id, name.to_string(), agent, Team::Attackers, skills);
        simulation.add_player(player);
    }
    
    for (id, name, agent, skills) in defenders {
        let player = Player::new(id, name.to_string(), agent, Team::Defenders, skills);
        simulation.add_player(player);
    }
    
    // Set initial credits
    for player in simulation.get_players_mut().values_mut() {
        player.current_credits = 800; // Starting pistol round credits
    }
    
    println!("✅ Added 10 players (5 per team) with diverse roles and skills");
}

fn create_round_context(round: u8) -> RoundContext {
    // Create varied round contexts to train the neural network
    let (round_type, team_economy) = match round {
        1 => (RoundType::Pistol, 4000),
        2..=3 => (RoundType::AntiEco, 8000),
        4 => (RoundType::FullBuy, 25000),
        5..=6 => (RoundType::Eco, 6000),
        7 => (RoundType::ForceBuy, 15000),
        8..=10 => (RoundType::FullBuy, 30000),
        11 => (RoundType::Eco, 8000),
        12 => (RoundType::ForceBuy, 18000),
        _ => (RoundType::FullBuy, 28000),
    };
    
    RoundContext {
        round_type,
        team_economy,
        enemy_predicted_economy: if team_economy > 20000 { 
            EconomyState::Strong 
        } else if team_economy > 15000 { 
            EconomyState::Moderate 
        } else { 
            EconomyState::Poor 
        },
        previous_round_result: Some(RoundEndReason::AllAttackersEliminated),
        loss_streak: if round > 5 && round % 4 == 0 { 2 } else { 0 },
    }
}

fn compare_buy_decisions(simulation: &mut ValorantSimulation, context: &RoundContext, _round: u8) {
    println!("🔄 Comparing neural vs traditional buy decisions:");
    
    // Update player credits based on round type
    for player in simulation.get_players_mut().values_mut() {
        player.current_credits = calculate_credits_for_round(context, player.agent.get_role());
    }
    
    // Compare decisions for a few key players
    let sample_players = vec![1, 3, 6, 8]; // Different roles
    
    for &player_id in &sample_players {
        if let Ok(neural_decision) = simulation.make_neural_buy_decision(player_id, context) {
            // Get player for traditional decision
            let player = simulation.get_players().get(&player_id).unwrap();
            let traditional_decision = simulation.make_dynamic_buy_decision(player, context);
            
            println!("  Player {} ({}): Neural vs Traditional", 
                player_id, 
                player.agent.get_role().to_string()
            );
            
            // Compare weapon choices
            let neural_weapon = neural_decision.primary_weapon
                .map(|w| format!("{:?}", w))
                .unwrap_or_else(|| "None".to_string());
            let traditional_weapon = traditional_decision.primary_weapon
                .map(|w| format!("{:?}", w))
                .unwrap_or_else(|| "None".to_string());
            
            println!("    Weapon: {} vs {}", neural_weapon, traditional_weapon);
            println!("    Cost: ${} vs ${}", neural_decision.total_cost, traditional_decision.total_cost);
            println!("    Confidence: {:.2} vs {:.2}", neural_decision.confidence, traditional_decision.confidence);
            
            // Show if neural network is learning differently
            if neural_weapon != traditional_weapon {
                println!("    🧠 Neural network chose differently!");
            }
        }
    }
}

fn simulate_round_performance(simulation: &mut ValorantSimulation, round: u8) {
    println!("⚔️  Simulating round {} performance...", round);
    
    // Simulate some realistic round outcomes
    let mut round_outcomes = HashMap::new();
    
    for player_id in 1..=10 {
        if let Some(player) = simulation.get_players().get(&player_id) {
            // Simulate performance based on player skills and weapon choice
            let base_performance = (player.skills.aim + player.skills.movement) / 2.0;
            
            // Add some randomness and weapon influence
            let performance_modifier = rand::random::<f32>() * 0.4 - 0.2; // -0.2 to +0.2
            let final_performance = (base_performance + performance_modifier).clamp(0.0, 1.0);
            
            // Simulate kills/deaths based on performance
            let kills = if final_performance > 0.8 { 2 } else if final_performance > 0.6 { 1 } else { 0 };
            let deaths = if final_performance < 0.3 { 1 } else { 0 };
            
            round_outcomes.insert(player_id, (kills, deaths, final_performance));
        }
    }
    
    // Record the round performance for neural network training
    simulation.record_round_performance(round);
    
    println!("   Recorded performance data for neural network training");
}

fn calculate_credits_for_round(context: &RoundContext, role: AgentRole) -> u32 {
    let base_credits = match context.round_type {
        RoundType::Pistol => 800,
        RoundType::Eco => 2000 + rand::random::<u32>() % 1000,
        RoundType::AntiEco => 4000 + rand::random::<u32>() % 2000,
        RoundType::ForceBuy => 3500 + rand::random::<u32>() % 1500,
        RoundType::FullBuy => 6000 + rand::random::<u32>() % 3000,
    };
    
    // Role-based credit variation
    let role_modifier = match role {
        AgentRole::Duelist => 1.1,   // Slightly more credits for entry fraggers
        AgentRole::Controller => 1.0, // Standard credits
        AgentRole::Initiator => 1.05, // Slightly more for utility
        AgentRole::Sentinel => 0.95,  // Slightly less, more conservative
    };
    
    (base_credits as f32 * role_modifier) as u32
}

fn print_training_progress(training_losses: &HashMap<u32, f32>, round: u8) {
    println!("🎯 Neural network training progress (Round {}):", round);
    
    let mut players: Vec<_> = training_losses.iter().collect();
    players.sort_by_key(|(id, _)| *id);
    
    for (player_id, loss) in &players {
        if loss.is_finite() && **loss > 0.0 {
            let status = if **loss < 0.1 {
                "🟢 Converging"
            } else if **loss < 0.3 {
                "🟡 Learning"
            } else {
                "🔴 Training"
            };
            println!("   Player {}: Loss = {:.4} {}", *player_id, **loss, status);
        }
    }
}

fn show_learning_insights(simulation: &ValorantSimulation, round: u8) {
    println!("\n📈 Learning Insights After Round {}:", round);
    
    let mut improving_count = 0;
    let mut stable_count = 0;
    let mut learning_count = 0;
    
    for player_id in 1..=10 {
        if let Some(insights) = simulation.get_player_learning_insights(player_id) {
            match insights.learning_trend.as_str() {
                "Improving" => improving_count += 1,
                "Stable" => stable_count += 1,
                _ => learning_count += 1,
            }
            
            if player_id <= 3 { // Show details for first 3 players
                println!("   Player {}: {} trend, {:.2} avg impact", 
                    player_id, insights.learning_trend, insights.recent_avg_impact);
            }
        }
    }
    
    println!("   Summary: {} improving, {} stable, {} still learning", 
        improving_count, stable_count, learning_count);
}

fn demonstrate_prediction_capabilities(simulation: &mut ValorantSimulation) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔮 Demonstrating Neural Network Prediction Capabilities");
    println!("=====================================================");
    
    // Create a hypothetical scenario
    let scenarios = vec![
        ("Pistol Round", RoundContext {
            round_type: RoundType::Pistol,
            team_economy: 4000,
            enemy_predicted_economy: EconomyState::Poor,
            previous_round_result: None,
            loss_streak: 0,
        }),
        ("Economy Round", RoundContext {
            round_type: RoundType::Eco,
            team_economy: 8000,
            enemy_predicted_economy: EconomyState::Strong,
            previous_round_result: Some(RoundEndReason::AllAttackersEliminated),
            loss_streak: 2,
        }),
        ("Full Buy Round", RoundContext {
            round_type: RoundType::FullBuy,
            team_economy: 35000,
            enemy_predicted_economy: EconomyState::Strong,
            previous_round_result: Some(RoundEndReason::SpikeDefused),
            loss_streak: 0,
        }),
    ];
    
    for (scenario_name, context) in scenarios {
        println!("\n🎪 Scenario: {}", scenario_name);
        println!("   Context: {:?}, Economy: {}", context.round_type, context.team_economy);
        
        // Test different player types
        let test_players = vec![
            (1, "Duelist"),
            (3, "Controller"), 
            (8, "Sentinel"),
        ];
        
        for (player_id, role_name) in test_players {
            match simulation.make_neural_buy_decision(player_id, &context) {
                Ok(decision) => {
                    println!("   {} prediction:", role_name);
                    if let Some(weapon) = &decision.primary_weapon {
                        println!("     Weapon: {:?}", weapon);
                    } else {
                        println!("     Weapon: Save/Eco");
                    }
                    println!("     Armor: {:?}", decision.armor);
                    println!("     Utility Budget: ${}", decision.abilities_budget);
                    println!("     Total Cost: ${}", decision.total_cost);
                    println!("     Confidence: {:.2}", decision.confidence);
                }
                Err(e) => {
                    println!("   {} prediction failed: {}", role_name, e);
                }
            }
        }
    }
    
    Ok(())
}

// Helper trait for role string conversion
trait RoleString {
    fn to_string(&self) -> String;
}

impl RoleString for AgentRole {
    fn to_string(&self) -> String {
        match self {
            AgentRole::Duelist => "Duelist".to_string(),
            AgentRole::Controller => "Controller".to_string(),
            AgentRole::Initiator => "Initiator".to_string(),
            AgentRole::Sentinel => "Sentinel".to_string(),
        }
    }
}