use VCTCareerBackend::sim::{Agent, Player, Team, ValorantSimulation};

fn colorize_player_id(id: u32) -> String {
    if id <= 5 {
        format!("\x1b[34m{}\x1b[0m", id) // Blue for attackers
    } else {
        format!("\x1b[31m{}\x1b[0m", id) // Red for defenders
    }
}

fn main() {
    let mut sim = ValorantSimulation::new();

    // Attackers with varied skills
    sim.add_player(Player::new(
        1,
        "Jett A".to_string(),
        Agent::Jett,
        Team::Attackers,
        0.8,
        0.7,
        0.6,
        0.75,
    ));
    sim.add_player(Player::new(
        2,
        "Phoenix A".to_string(),
        Agent::Phoenix,
        Team::Attackers,
        0.65,
        0.5,
        0.7,
        0.8,
    ));
    sim.add_player(Player::new(
        3,
        "Sova A".to_string(),
        Agent::Sova,
        Team::Attackers,
        0.7,
        0.6,
        0.65,
        0.9,
    ));
    sim.add_player(Player::new(
        4,
        "Sage A".to_string(),
        Agent::Sage,
        Team::Attackers,
        0.5,
        0.4,
        0.5,
        0.85,
    ));
    sim.add_player(Player::new(
        5,
        "Viper A".to_string(),
        Agent::Viper,
        Team::Attackers,
        0.75,
        0.65,
        0.55,
        0.8,
    ));

    // Defenders with varied skills
    sim.add_player(Player::new(
        6,
        "Omen D".to_string(),
        Agent::Omen,
        Team::Defenders,
        0.7,
        0.6,
        0.75,
        0.8,
    ));
    sim.add_player(Player::new(
        7,
        "Killjoy D".to_string(),
        Agent::Killjoy,
        Team::Defenders,
        0.6,
        0.55,
        0.6,
        0.9,
    ));
    sim.add_player(Player::new(
        8,
        "Cypher D".to_string(),
        Agent::Cypher,
        Team::Defenders,
        0.65,
        0.5,
        0.7,
        0.85,
    ));
    sim.add_player(Player::new(
        9,
        "Raze D".to_string(),
        Agent::Raze,
        Team::Defenders,
        0.85,
        0.75,
        0.65,
        0.7,
    ));
    sim.add_player(Player::new(
        10,
        "Breach D".to_string(),
        Agent::Breach,
        Team::Defenders,
        0.7,
        0.6,
        0.7,
        0.75,
    ));

    let _ = sim.run_simulation_to_completion();

    for event in sim.events {
        match event {
            VCTCareerBackend::sim::GameEvent::MatchStart { timestamp } => {
                println!("[{}] Match Start", timestamp);
            }
            VCTCareerBackend::sim::GameEvent::MatchEnd {
                timestamp,
                winning_team,
                score_attackers,
                score_defenders,
            } => {
                println!(
                    "[{}] Match End - Winner: {:?} | Final Score: {}-{}",
                    timestamp, winning_team, score_attackers, score_defenders
                );
            }
            VCTCareerBackend::sim::GameEvent::BuyPhaseStart {
                timestamp,
                round_number,
            } => {
                println!("[{}] Round {} Buy Phase Start", timestamp, round_number);
            }
            VCTCareerBackend::sim::GameEvent::BuyPhaseEnd {
                timestamp,
                round_number,
            } => {
                println!("[{}] Round {} Buy Phase End", timestamp, round_number);
            }
            VCTCareerBackend::sim::GameEvent::RoundStart {
                timestamp,
                round_number,
                attacker_credits_start,
                defender_credits_start,
            } => {
                println!(
                    "[{}] Round {} Start | Attacker Credits: {} | Defender Credits: {}",
                    timestamp, round_number, attacker_credits_start, defender_credits_start
                );
            }
            VCTCareerBackend::sim::GameEvent::RoundEnd {
                timestamp,
                round_number,
                winning_team,
                reason,
            } => {
                println!(
                    "[{}] Round {} End - Winner: {:?} | Reason: {:?}",
                    timestamp, round_number, winning_team, reason
                );
            }
            VCTCareerBackend::sim::GameEvent::Kill {
                timestamp,
                killer_id,
                victim_id,
                weapon,
                is_headshot,
            } => {
                println!(
                    "[{}] Kill: Player {} -> Player {} with {:?}{}",
                    timestamp,
                    colorize_player_id(killer_id),
                    colorize_player_id(victim_id),
                    weapon,
                    if is_headshot { " (HS)" } else { "" }
                );
            }
            VCTCareerBackend::sim::GameEvent::Damage {
                timestamp,
                attacker_id,
                victim_id,
                amount,
                weapon,
                is_headshot,
            } => {
                println!(
                    "[{}] Damage: Player {} -> Player {} | {} dmg with {:?}{}",
                    timestamp,
                    colorize_player_id(attacker_id),
                    colorize_player_id(victim_id),
                    amount,
                    weapon,
                    if is_headshot { " (HS)" } else { "" }
                );
            }
            VCTCareerBackend::sim::GameEvent::SpikePlant {
                timestamp,
                planter_id,
            } => {
                println!(
                    "[{}] Spike Planted by Player {}",
                    timestamp,
                    colorize_player_id(planter_id)
                );
            }
            VCTCareerBackend::sim::GameEvent::SpikeDefuse {
                timestamp,
                defuser_id,
                successful,
            } => {
                println!(
                    "[{}] Spike Defuse by Player {} | Successful: {}",
                    timestamp,
                    colorize_player_id(defuser_id),
                    successful
                );
            }
            VCTCareerBackend::sim::GameEvent::AbilityUsed {
                timestamp,
                player_id,
                ability_name,
            } => {
                println!(
                    "[{}] Ability Used: Player {} -> {}",
                    timestamp,
                    colorize_player_id(player_id),
                    ability_name
                );
            }
            VCTCareerBackend::sim::GameEvent::SideSwap {
                timestamp,
                round_number,
            } => {
                println!(
                    "[{}] Side Swap at Round {} - Teams switching sides",
                    timestamp, round_number
                );
            }
        }
    }
}
