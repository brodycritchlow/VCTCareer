use vctcareer_backend::sim::{
    Agent, Player, PlayerSkills, Team, ValorantSimulation, ArmorType, RoundType, 
    RoundContext, EconomyState, RoundEndReason
};

fn main() {
    println!("🔍 Debugging Dynamic Buy System");
    
    let mut sim = ValorantSimulation::new();
    
    // Create a simple test player with high credits
    let skills = PlayerSkills {
        aim: 0.8,
        hs: 0.7,
        movement: 0.75,
        util: 0.8,
    };
    
    let mut player = Player::new(1, "TestPlayer".to_string(), Agent::Jett, Team::Attackers, skills);
    player.current_credits = 8000; // Lots of credits
    
    println!("🎮 Player before buying:");
    println!("   Credits: {}", player.current_credits);
    println!("   Primary: {:?}", player.current_loadout.primary_weapon);
    println!("   Armor: {:?}", player.current_loadout.armor);
    
    // Show player preferences
    println!("\n🔧 Player preferences:");
    for (i, weapon_pref) in player.buy_preferences.preferred_weapons.iter().take(3).enumerate() {
        println!("   {}. {:?} - Priority: {:.2}, Min Credits: {}", 
                 i + 1, weapon_pref.weapon, weapon_pref.priority, weapon_pref.min_credits);
    }
    
    // Create a full buy context
    let context = RoundContext {
        round_type: RoundType::FullBuy,
        team_economy: 40000,
        enemy_predicted_economy: EconomyState::Strong,
        previous_round_result: None,
        loss_streak: 0,
    };
    
    println!("\n🎯 Round context:");
    println!("   Round Type: {:?}", context.round_type);
    println!("   Team Economy: {}", context.team_economy);
    
    // Test the buy decision
    let decision = sim.make_dynamic_buy_decision(&player, &context);
    
    println!("\n💡 Buy decision:");
    println!("   Primary weapon: {:?}", decision.primary_weapon);
    println!("   Secondary weapon: {:?}", decision.secondary_weapon);
    println!("   Armor: {:?}", decision.armor);
    println!("   Total cost: {}", decision.total_cost);
    println!("   Confidence: {:.2}", decision.confidence);
    
    // Add player to simulation and test full buy process
    sim.add_player(player);
    
    println!("\n🛒 Testing full buy process...");
    
    // Simulate buying
    sim.simulate_player_purchases();
    
    let player_after = sim.players.get(&1).unwrap();
    
    println!("\n🎮 Player after buying:");
    println!("   Credits: {} (spent: {})", 
             player_after.current_credits, 
             8000 - player_after.current_credits);
    println!("   Primary: {:?}", player_after.current_loadout.primary_weapon);
    println!("   Secondary: {:?}", player_after.current_loadout.secondary_weapon);
    println!("   Armor: {:?}", player_after.current_loadout.armor);
    
    // Test with different credit amounts
    println!("\n🧪 Testing different credit scenarios:");
    
    let credit_scenarios = vec![
        ("Rich", 9000),
        ("Moderate", 4000),  
        ("Low", 2000),
        ("Poor", 1000),
    ];
    
    for (scenario_name, credits) in credit_scenarios {
        let mut test_player = Player::new(2, format!("Test{}", scenario_name), Agent::Jett, Team::Attackers, 
                                         PlayerSkills { aim: 0.8, hs: 0.7, movement: 0.75, util: 0.8 });
        test_player.current_credits = credits;
        
        let decision = sim.make_dynamic_buy_decision(&test_player, &context);
        
        println!("   {} ({} credits): {:?} + {:?} = {} cost", 
                 scenario_name, credits,
                 decision.primary_weapon.unwrap_or(vctcareer_backend::sim::Weapon::Classic),
                 decision.armor,
                 decision.total_cost);
    }
    
    println!("\n✅ Debug complete!");
}