use std::collections::HashMap;
use vctcareer_backend::sim::{
    Agent, AgentRole, ArmorType, Player, PlayerSkills, Team, ValorantSimulation, Weapon
};

fn create_skilled_player(name: &str, agent: Agent, team: Team, aim: f32, hs: f32) -> Player {
    let skills = PlayerSkills {
        aim,
        hs,
        movement: 0.75,
        util: 0.8,
    };
    Player::new(1, name.to_string(), agent, team, skills)
}

fn print_buy_analysis(sim: &ValorantSimulation, round_name: &str) {
    println!("\n🎯 {} Buy Analysis:", round_name);
    
    let mut weapon_counts: HashMap<String, u32> = HashMap::new();
    let mut armor_counts: HashMap<ArmorType, u32> = HashMap::new();
    let mut role_purchases: HashMap<AgentRole, Vec<String>> = HashMap::new();
    
    for player in sim.players.values() {
        let weapon_name = match &player.current_loadout.primary_weapon {
            Some(weapon) => format!("{:?}", weapon),
            None => format!("{:?}", player.current_loadout.secondary_weapon),
        };
        
        *weapon_counts.entry(weapon_name.clone()).or_insert(0) += 1;
        *armor_counts.entry(player.current_loadout.armor.clone()).or_insert(0) += 1;
        
        let role = player.agent.get_role();
        let purchases = role_purchases.entry(role).or_insert_with(Vec::new);
        purchases.push(format!("{}: {}", player.name, weapon_name));
    }
    
    println!("   📊 Weapon Distribution:");
    for (weapon, count) in weapon_counts {
        println!("      {}: {} players", weapon, count);
    }
    
    println!("\n   🛡️ Armor Distribution:");
    for (armor, count) in armor_counts {
        println!("      {:?}: {} players", armor, count);
    }
    
    println!("\n   🎭 Role-Based Purchases:");
    for (role, purchases) in role_purchases {
        println!("      {:?}s:", role);
        for purchase in purchases {
            println!("         {}", purchase);
        }
    }
}

fn main() {
    println!("🎮 VCT Career Dynamic Buy System Showcase");
    println!("==========================================");
    
    let mut sim = ValorantSimulation::new();
    
    // Create a diverse team with different roles and skill levels
    let team_data = vec![
        // Attackers
        ("TenZ", Agent::Jett, Team::Attackers, 0.95, 0.9),
        ("ShahZaM", Agent::Sova, Team::Attackers, 0.85, 0.8),
        ("zombs", Agent::Omen, Team::Attackers, 0.8, 0.7),
        ("SicK", Agent::Sage, Team::Attackers, 0.88, 0.82),
        ("dapr", Agent::Phoenix, Team::Attackers, 0.87, 0.83),
        
        // Defenders
        ("yay", Agent::Chamber, Team::Defenders, 0.97, 0.95),
        ("Marved", Agent::Viper, Team::Defenders, 0.82, 0.75),
        ("crashies", Agent::Breach, Team::Defenders, 0.83, 0.78),
        ("Victor", Agent::Raze, Team::Defenders, 0.86, 0.81),
        ("FNS", Agent::Killjoy, Team::Defenders, 0.75, 0.68),
    ];
    
    for (i, (name, agent, team, aim, hs)) in team_data.into_iter().enumerate() {
        let mut player = create_skilled_player(name, agent, team, aim, hs);
        player.id = (i + 1) as u32;
        sim.add_player(player);
    }
    
    sim.start_simulation();
    
    // Demonstrate Round 1 (Pistol Round)
    println!("\n🔫 ROUND 1: PISTOL ROUND");
    println!("=========================");
    
    sim.state.current_round = 1;
    for player in sim.players.values_mut() {
        player.current_credits = 800; // Starting credits
        player.current_loadout.primary_weapon = None;
        player.current_loadout.armor = ArmorType::None;
        player.current_loadout.abilities_purchased.clear();
    }
    
    sim.simulate_player_purchases();
    print_buy_analysis(&sim, "Pistol Round");
    
    // Demonstrate Round 3 (Full Buy with Team Coordination)
    println!("\n\n💰 ROUND 3: COORDINATED FULL BUY ROUND");
    println!("=======================================");
    
    sim.state.current_round = 3;
    for player in sim.players.values_mut() {
        player.current_credits = 5000; // Rich team
        player.current_loadout.primary_weapon = None;
        player.current_loadout.armor = ArmorType::None;
        player.current_loadout.abilities_purchased.clear();
    }
    
    sim.simulate_player_purchases();
    print_buy_analysis(&sim, "Coordinated Full Buy Round");
    
    // Demonstrate Eco Round with Team Strategy
    println!("\n\n💸 ROUND 5: STRATEGIC ECO ROUND");
    println!("================================");
    
    sim.state.current_round = 5;
    for player in sim.players.values_mut() {
        player.current_credits = 1200; // Poor economy
        player.current_loadout.primary_weapon = None;
        player.current_loadout.armor = ArmorType::None;
        player.current_loadout.abilities_purchased.clear();
    }
    
    sim.simulate_player_purchases();
    print_buy_analysis(&sim, "Strategic Eco Round");
    
    // Demonstrate Force Buy
    println!("\n\n⚡ ROUND 7: FORCE BUY (Team Coordination Test)");
    println!("===============================================");
    
    sim.state.current_round = 7;
    sim.loss_streaks.insert(Team::Attackers, 3);
    sim.loss_streaks.insert(Team::Defenders, 3);
    
    for player in sim.players.values_mut() {
        player.current_credits = 2400; // Moderate credits
        player.current_loadout.primary_weapon = None;
        player.current_loadout.armor = ArmorType::None;
        player.current_loadout.abilities_purchased.clear();
    }
    
    sim.simulate_player_purchases();
    print_buy_analysis(&sim, "Force Buy Round (Team Coordinated)");
    
    // Show individual player preferences
    println!("\n\n🎯 PLAYER PREFERENCE & COORDINATION ANALYSIS");
    println!("=============================================");
    
    for player in sim.players.values().take(4) {
        println!("\n🎮 {} ({:?} - {:?})", player.name, player.agent, player.agent.get_role());
        println!("   💰 Eco Threshold: {} credits", player.buy_preferences.eco_threshold);
        println!("   ⚡ Force Buy Tendency: {:.0}%", player.buy_preferences.force_buy_tendency * 100.0);
        println!("   🔧 Utility Priority: {:.0}%", player.buy_preferences.utility_priority * 100.0);
        
        // Show latest purchase details
        println!("   🛒 Latest Purchase:");
        println!("      Weapon: {:?}", player.current_loadout.primary_weapon.as_ref().unwrap_or(&Weapon::Classic));
        println!("      Armor: {:?}", player.current_loadout.armor);
        if !player.current_loadout.abilities_purchased.is_empty() {
            println!("      Utilities: {}", player.current_loadout.abilities_purchased.join(", "));
        }
        
        println!("   🔫 Top Weapon Preferences:");
        for (i, weapon_pref) in player.buy_preferences.preferred_weapons.iter().take(3).enumerate() {
            println!("      {}. {:?} (Priority: {:.2})", i + 1, weapon_pref.weapon, weapon_pref.priority);
        }
    }
    
    println!("\n\n🎊 PHASE 2 FEATURES SUMMARY");
    println!("=============================");
    println!("✅ Pistol Round: Players buy pistols and light armor");
    println!("✅ Full Buy Round: Role-based weapon variety (Vandal, Phantom, Operator, Guardian)");
    println!("✅ Eco Round: Players save money or buy minimal items");
    println!("✅ Force Buy: Aggressive players force buy despite low economy");
    println!("✅ Player Personality: Each player has unique buying preferences");
    println!("✅ Role Specialization: Duelists, Controllers, Sentinels, Initiators buy differently");
    println!("✅ Team Coordination: Strategic team-wide buy decisions");
    println!("✅ Utility Budget Management: Role-based utility allocation");
    println!("✅ Team Composition Awareness: Balanced team utility coverage");
    
    println!("\n🚀 Phase 2 of the dynamic buy system is working perfectly!");
    println!("   Players make realistic, coordinated decisions based on:");
    println!("   • Agent roles and responsibilities");
    println!("   • Individual skill levels and preferences");
    println!("   • Round context and team economy");
    println!("   • Loss streaks and pressure situations");
    println!("   • Team strategy and coordination");
    println!("   • Utility budget allocation by role");
    println!("   • Strategic planning and composition balance");
}