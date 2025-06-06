use std::collections::HashMap;
use rand::Rng;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Agent {
    Jett, Raze, Phoenix, Breach, Sova, Sage, Omen, Brimstone, Viper, Cypher, Killjoy, Skye, Yoru,
    Astra, Kayo, Chamber, Neon, Fade, Harbor, Gekko, Deadlock, Iso, Clove,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Weapon {
    Classic, Shorty, Frenzy, Ghost, Sheriff, Stinger, Spectre, Bucky, Judge, Bulldog, Guardian,
    Phantom, Vandal, Marshal, Operator, Ares, Odin, Knife,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Team {
    Attackers,
    Defenders,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BodyPart {
    Head,
    Body,
    Legs,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArmorType {
    None,
    Light,   // 25 armor, costs 400
    Heavy,   // 50 armor, costs 1000
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Penetration {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone)]
pub struct WeaponStats {
    pub price: u32,
    pub damage_head: (u32, u32, u32),    // no armor, light armor, heavy armor
    pub damage_body: (u32, u32, u32),
    pub damage_legs: (u32, u32, u32),
    pub fire_rate: f32,                  // rounds per second
    pub penetration: Penetration,
    pub magazine_size: u32,
    pub reload_time_ms: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerLoadout {
    pub primary_weapon: Option<Weapon>,
    pub secondary_weapon: Weapon,  // Always have Classic minimum
    pub armor: ArmorType,
    pub abilities_purchased: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Player {
    pub id: u32,
    pub name: String,
    pub agent: Agent,
    pub team: Team,
    pub current_health: u32,
    pub current_armor: u32,
    pub is_alive: bool,
    pub current_credits: u32,
    pub ultimate_points: u32,
    pub current_loadout: PlayerLoadout,

    pub aim_skill: f32,
    pub hs_skill: f32,
    pub movement_skill: f32,
    pub util_skill: f32,
}

impl Player {
    pub fn new(
        id: u32, name: String, agent: Agent, team: Team,
        aim: f32, hs: f32, movement: f32, util: f32,
    ) -> Self {
        Player {
            id,
            name,
            agent,
            team,
            current_health: 100,
            current_armor: 0,
            is_alive: true,
            current_credits: 0,
            ultimate_points: 0,
            current_loadout: PlayerLoadout {
                primary_weapon: None,
                secondary_weapon: Weapon::Classic,
                armor: ArmorType::None,
                abilities_purchased: Vec::new(),
            },
            aim_skill: aim.clamp(0.0, 1.0),
            hs_skill: hs.clamp(0.0, 1.0),
            movement_skill: movement.clamp(0.0, 1.0),
            util_skill: util.clamp(0.0, 1.0),
        }
    }

    pub fn reset_for_round(&mut self) {
        self.current_health = 100;
        // Set armor based on purchased armor type
        self.current_armor = match self.current_loadout.armor {
            ArmorType::None => 0,
            ArmorType::Light => 25,
            ArmorType::Heavy => 50,
        };
        self.is_alive = true;
    }

    pub fn take_damage(&mut self, amount: u32) {
        if !self.is_alive { return; }

        let total_health = self.current_health + self.current_armor;
        if amount >= total_health {
            self.current_health = 0;
            self.current_armor = 0;
            self.is_alive = false;
        } else if amount > self.current_armor {
            let remaining_damage = amount - self.current_armor;
            self.current_armor = 0;
            self.current_health = self.current_health.saturating_sub(remaining_damage);
        } else {
            self.current_armor = self.current_armor.saturating_sub(amount);
        }
    }

    pub fn survived_round(&self) -> bool {
        self.is_alive
    }
}

pub type Timestamp = u64;

#[derive(Debug, Clone)]
pub enum GameEvent {
    MatchStart { timestamp: Timestamp },
    MatchEnd {
        timestamp: Timestamp,
        winning_team: Team,
        score_attackers: u8,
        score_defenders: u8,
    },
    BuyPhaseStart {
        timestamp: Timestamp,
        round_number: u8,
    },
    BuyPhaseEnd {
        timestamp: Timestamp,
        round_number: u8,
    },
    RoundStart {
        timestamp: Timestamp,
        round_number: u8,
        attacker_credits_start: u32,
        defender_credits_start: u32,
    },
    RoundEnd {
        timestamp: Timestamp,
        round_number: u8,
        winning_team: Team,
        reason: RoundEndReason,
    },
    Kill {
        timestamp: Timestamp,
        killer_id: u32,
        victim_id: u32,
        weapon: Weapon,
        is_headshot: bool,
    },
    Damage {
        timestamp: Timestamp,
        attacker_id: u32,
        victim_id: u32,
        amount: u32,
        weapon: Weapon,
        is_headshot: bool,
    },
    SpikePlant {
        timestamp: Timestamp,
        planter_id: u32,
    },
    SpikeDefuse {
        timestamp: Timestamp,
        defuser_id: u32,
        successful: bool,
    },
    AbilityUsed {
        timestamp: Timestamp,
        player_id: u32,
        ability_name: String,
    },
    SideSwap {
        timestamp: Timestamp,
        round_number: u8,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum RoundEndReason {
    AllAttackersEliminated,
    AllDefendersEliminated,
    SpikeDetonated,
    SpikeDefused,
    TimeExpired,
}

pub struct ValorantSimulation {
    pub players: HashMap<u32, Player>,
    pub events: Vec<GameEvent>,
    pub current_timestamp: Timestamp,
    pub current_round: u8,
    pub attacker_score: u8,
    pub defender_score: u8,
    pub overtime_active: bool,
    pub loss_streaks: HashMap<Team, u8>,
    pub weapon_stats: HashMap<Weapon, WeaponStats>,
}

impl ValorantSimulation {
    pub fn new() -> Self {
        let mut weapon_stats = HashMap::new();
        
        // Initialize weapon stats based on Valorant data
        weapon_stats.insert(Weapon::Classic, WeaponStats {
            price: 0,
            damage_head: (78, 66, 26),
            damage_body: (26, 22, 22),
            damage_legs: (22, 18, 18),
            fire_rate: 6.75,
            penetration: Penetration::Low,
            magazine_size: 12,
            reload_time_ms: 2250,
        });
        
        weapon_stats.insert(Weapon::Ghost, WeaponStats {
            price: 500,
            damage_head: (105, 87, 30),
            damage_body: (30, 25, 25),
            damage_legs: (25, 21, 21),
            fire_rate: 6.75,
            penetration: Penetration::Medium,
            magazine_size: 15,
            reload_time_ms: 2500,
        });
        
        weapon_stats.insert(Weapon::Sheriff, WeaponStats {
            price: 800,
            damage_head: (159, 145, 55),
            damage_body: (55, 50, 46),
            damage_legs: (46, 42, 42),
            fire_rate: 4.0,
            penetration: Penetration::High,
            magazine_size: 6,
            reload_time_ms: 3000,
        });
        
        weapon_stats.insert(Weapon::Spectre, WeaponStats {
            price: 1600,
            damage_head: (78, 66, 26),
            damage_body: (26, 22, 22),
            damage_legs: (22, 18, 18),
            fire_rate: 13.33,
            penetration: Penetration::Medium,
            magazine_size: 30,
            reload_time_ms: 2250,
        });
        
        weapon_stats.insert(Weapon::Phantom, WeaponStats {
            price: 2900,
            damage_head: (156, 140, 124), // Close range
            damage_body: (39, 35, 31),
            damage_legs: (33, 29, 26),
            fire_rate: 11.0,
            penetration: Penetration::Medium,
            magazine_size: 30,
            reload_time_ms: 2500,
        });
        
        weapon_stats.insert(Weapon::Vandal, WeaponStats {
            price: 2900,
            damage_head: (160, 160, 160), // Always 160 regardless of armor
            damage_body: (40, 40, 40),
            damage_legs: (34, 34, 34),
            fire_rate: 9.75,
            penetration: Penetration::Medium,
            magazine_size: 25,
            reload_time_ms: 2500,
        });
        
        weapon_stats.insert(Weapon::Operator, WeaponStats {
            price: 4700,
            damage_head: (255, 255, 255),
            damage_body: (150, 150, 150),
            damage_legs: (120, 120, 120),
            fire_rate: 0.75,
            penetration: Penetration::High,
            magazine_size: 5,
            reload_time_ms: 3700,
        });

        ValorantSimulation {
            players: HashMap::new(),
            events: Vec::new(),
            current_timestamp: 0,
            current_round: 0,
            attacker_score: 0,
            defender_score: 0,
            overtime_active: false,
            loss_streaks: HashMap::new(),
            weapon_stats,
        }
    }

    pub fn add_player(&mut self, player: Player) {
        self.players.insert(player.id, player);
    }

    pub fn advance_time(&mut self, delta_ms: u64) {
        self.current_timestamp += delta_ms;
    }

    fn record_event(&mut self, event: GameEvent) {
        self.events.push(event);
    }

    pub fn get_alive_players_on_team(&self, team: &Team) -> Vec<&Player> {
        self.players.values()
            .filter(|p| p.team == *team && p.is_alive)
            .collect()
    }

    fn calculate_round_rewards(&mut self, winning_team: &Team, reason: &RoundEndReason, spike_planted: bool) {
        // Award credits based on Valorant economy system
        for player in self.players.values_mut() {
            let mut credits_earned = 0;
            
            if player.team == *winning_team {
                // Win reward
                credits_earned += 3000;
                
                // Reset loss streak for winning team
                self.loss_streaks.insert(player.team.clone(), 0);
            } else {
                // Loss reward with streak bonus
                let loss_streak = self.loss_streaks.get(&player.team).unwrap_or(&0);
                credits_earned += match loss_streak {
                    0 => 1900,      // First loss
                    1 => 2400,      // Second consecutive loss
                    _ => 2900,      // Third+ consecutive loss
                };
                
                // Update loss streak
                self.loss_streaks.insert(player.team.clone(), loss_streak + 1);
                
                // Survival bonus (if they survived a lost round)
                if player.survived_round() {
                    credits_earned = credits_earned.min(1000); // Cap at 1000 for survival
                }
            }
            
            // Spike plant bonus (300 credits per team member)
            if spike_planted && player.team == Team::Attackers {
                credits_earned += 300;
            }
            
            player.current_credits = (player.current_credits + credits_earned).min(9000);
        }
    }

    fn calculate_weapon_damage(&self, weapon: &Weapon, armor_type: &ArmorType, body_part: BodyPart, range_meters: f32) -> u32 {
        let stats = &self.weapon_stats[weapon];
        
        let base_damage = match body_part {
            BodyPart::Head => match armor_type {
                ArmorType::None => stats.damage_head.0,
                ArmorType::Light => stats.damage_head.1,
                ArmorType::Heavy => stats.damage_head.2,
            },
            BodyPart::Body => match armor_type {
                ArmorType::None => stats.damage_body.0,
                ArmorType::Light => stats.damage_body.1,
                ArmorType::Heavy => stats.damage_body.2,
            },
            BodyPart::Legs => match armor_type {
                ArmorType::None => stats.damage_legs.0,
                ArmorType::Light => stats.damage_legs.1,
                ArmorType::Heavy => stats.damage_legs.2,
            },
        };

        // Apply range penalties (simplified)
        let damage_multiplier = match weapon {
            Weapon::Phantom => {
                if range_meters <= 15.0 { 1.0 } 
                else if range_meters <= 30.0 { 0.85 } 
                else { 0.7 }
            },
            Weapon::Spectre | Weapon::Stinger => {
                if range_meters <= 20.0 { 1.0 } else { 0.75 }
            },
            _ => 1.0, // No damage falloff for most weapons
        };

        (base_damage as f32 * damage_multiplier) as u32
    }

    fn calculate_weapon_effectiveness(&self, weapon: &Weapon) -> f32 {
        match weapon {
            Weapon::Operator => 1.5,    // Massive aim advantage
            Weapon::Vandal => 1.2,      // High damage, good accuracy
            Weapon::Phantom => 1.15,    // Good balance
            Weapon::Guardian => 1.1,    // High damage, slower
            Weapon::Spectre => 0.9,     // Good for close range
            Weapon::Sheriff => 0.8,     // High damage pistol
            Weapon::Ghost => 0.6,       // Balanced pistol
            Weapon::Classic => 0.4,     // Basic weapon
            _ => 0.7,                   // Default effectiveness
        }
    }

    fn award_kill_bonus(&mut self, killer_id: u32) {
        if let Some(killer) = self.players.get_mut(&killer_id) {
            killer.current_credits = (killer.current_credits + 200).min(9000);
            killer.ultimate_points += 1; // TODO: Implement proper ult point system
        }
    }

    fn award_spike_plant_bonus(&mut self, planter_id: u32) {
        if let Some(planter) = self.players.get_mut(&planter_id) {
            planter.ultimate_points += 1; // TODO: Implement proper ult point system
        }
    }

    fn simulate_buy_phase(&mut self) {
        self.record_event(GameEvent::BuyPhaseStart {
            timestamp: self.current_timestamp,
            round_number: self.current_round,
        });
        
        // Simple AI buying logic
        for player in self.players.values_mut() {
            // Reset loadout if they died (don't carry over equipment)
            if !player.survived_round() {
                player.current_loadout = PlayerLoadout {
                    primary_weapon: None,
                    secondary_weapon: Weapon::Classic,
                    armor: ArmorType::None,
                    abilities_purchased: Vec::new(),
                };
            }
            
            // Basic buying strategy
            if player.current_credits >= 5700 { // Operator + Heavy armor
                player.current_loadout.primary_weapon = Some(Weapon::Operator);
                player.current_loadout.armor = ArmorType::Heavy;
                player.current_credits -= 5700;
            } else if player.current_credits >= 3900 { // Vandal + Heavy armor
                player.current_loadout.primary_weapon = Some(Weapon::Vandal);
                player.current_loadout.armor = ArmorType::Heavy;
                player.current_credits -= 3900;
            } else if player.current_credits >= 1600 { // SMG buy
                player.current_loadout.primary_weapon = Some(Weapon::Spectre);
                player.current_credits -= 1600;
                if player.current_credits >= 400 {
                    player.current_loadout.armor = ArmorType::Light;
                    player.current_credits -= 400;
                }
            } else if player.current_credits >= 800 { // Pistol upgrade
                player.current_loadout.secondary_weapon = Weapon::Sheriff;
                player.current_credits -= 800;
            }
        }
        
        self.advance_time(30000); // 30 second buy phase
        
        self.record_event(GameEvent::BuyPhaseEnd {
            timestamp: self.current_timestamp,
            round_number: self.current_round,
        });
    }

    pub fn run_simulation(&mut self) {
        self.record_event(GameEvent::MatchStart { timestamp: self.current_timestamp });

        const MAX_ROUNDS_REGULAR: u8 = 24;
        const WIN_SCORE_REGULAR: u8 = 13;
        const WIN_MARGIN_OVERTIME: u8 = 2;
        const SWITCH_SIDES_AFTER: u8 = 12;

        // Initialize loss streaks
        self.loss_streaks.insert(Team::Attackers, 0);
        self.loss_streaks.insert(Team::Defenders, 0);

        // Give starting credits
        for player in self.players.values_mut() {
            player.current_credits = 800;
        }

        let mut round_winner: Option<Team> = None;
        let mut round_reason: Option<RoundEndReason> = None;
        let mut spike_planted = false;

        'game_loop: loop {
            self.current_round += 1;
            self.advance_time(5000);

            // Handle side swaps
            if self.current_round == SWITCH_SIDES_AFTER + 1 {
                for player in self.players.values_mut() {
                    player.team = match player.team {
                        Team::Attackers => Team::Defenders,
                        Team::Defenders => Team::Attackers,
                    };
                    // Reset economy on side swap
                    player.current_credits = 800;
                    player.current_loadout = PlayerLoadout {
                        primary_weapon: None,
                        secondary_weapon: Weapon::Classic,
                        armor: ArmorType::None,
                        abilities_purchased: Vec::new(),
                    };
                }
                // Reset loss streaks on side swap
                self.loss_streaks.insert(Team::Attackers, 0);
                self.loss_streaks.insert(Team::Defenders, 0);
                
                self.record_event(GameEvent::SideSwap {
                    timestamp: self.current_timestamp,
                    round_number: self.current_round,
                });
            }

            // Buy phase simulation
            self.simulate_buy_phase();

            // Reset players for round
            for player in self.players.values_mut() {
                player.reset_for_round();
            }

            let mut spike_defused = false;

            self.record_event(GameEvent::RoundStart {
                timestamp: self.current_timestamp,
                round_number: self.current_round,
                attacker_credits_start: self.players.values().find(|p| p.team == Team::Attackers).map_or(0, |p| p.current_credits),
                defender_credits_start: self.players.values().find(|p| p.team == Team::Defenders).map_or(0, |p| p.current_credits),
            });
            let mut spike_timer_ms: i32 = 45_000;
            const ROUND_MAX_TIME_MS: u64 = 100_000;
            let round_start_timestamp = self.current_timestamp;

            let mut rng = rand::thread_rng();

            'round_loop: loop {
                self.advance_time(500);

                let alive_attackers_ids: Vec<u32> = self.get_alive_players_on_team(&Team::Attackers).into_iter().map(|p| p.id).collect();
                let alive_defenders_ids: Vec<u32> = self.get_alive_players_on_team(&Team::Defenders).into_iter().map(|p| p.id).collect();

                if alive_attackers_ids.is_empty() {
                    round_winner = Some(Team::Defenders);
                    round_reason = Some(RoundEndReason::AllAttackersEliminated);
                    break 'round_loop;
                }
                if alive_defenders_ids.is_empty() {
                    if spike_planted && !spike_defused {
                        round_winner = Some(Team::Attackers);
                        round_reason = Some(RoundEndReason::SpikeDetonated);
                    } else {
                        round_winner = Some(Team::Attackers);
                        round_reason = Some(RoundEndReason::AllDefendersEliminated);
                    }
                    break 'round_loop;
                }

                if !spike_planted {
                    if self.current_timestamp - round_start_timestamp > 30_000 && rng.random::<f32>() < 0.15 {
                        let planter_id = alive_attackers_ids[rng.random_range(0..alive_attackers_ids.len())];
                        self.record_event(GameEvent::SpikePlant { timestamp: self.current_timestamp, planter_id });
                        self.award_spike_plant_bonus(planter_id);
                        spike_planted = true;
                    }
                } else {
                    spike_timer_ms = spike_timer_ms.saturating_sub(500);
                    if spike_timer_ms <= 0 {
                        round_winner = Some(Team::Attackers);
                        round_reason = Some(RoundEndReason::SpikeDetonated);
                        break 'round_loop;
                    }

                    if !alive_defenders_ids.is_empty() && rng.random::<f32>() < 0.05 {
                        let defuser_id = alive_defenders_ids[rng.random_range(0..alive_defenders_ids.len())];
                        self.record_event(GameEvent::SpikeDefuse { timestamp: self.current_timestamp, defuser_id, successful: true });
                        // Award ult point for defuse
                        if let Some(defuser) = self.players.get_mut(&defuser_id) {
                            defuser.ultimate_points += 1;
                        }
                        spike_defused = true;
                        round_winner = Some(Team::Defenders);
                        round_reason = Some(RoundEndReason::SpikeDefused);
                        break 'round_loop;
                    }
                }

                // Combat simulation with weapon stats (only one engagement per tick)
                if !alive_attackers_ids.is_empty() && !alive_defenders_ids.is_empty() {
                    let attacker_id = alive_attackers_ids[rng.random_range(0..alive_attackers_ids.len())];
                    let defender_id = alive_defenders_ids[rng.random_range(0..alive_defenders_ids.len())];

                    // Double-check both players are still alive (could have died earlier this tick)
                    let attacker_still_alive = self.players.get(&attacker_id).map_or(false, |p| p.is_alive);
                    let defender_still_alive = self.players.get(&defender_id).map_or(false, |p| p.is_alive);
                    
                    if !attacker_still_alive || !defender_still_alive {
                        continue; // Skip this combat if either player is dead
                    }

                    let attacker_player_data = self.players.get(&attacker_id).unwrap().clone();
                    let defender_player_data = self.players.get(&defender_id).unwrap().clone();

                    // Use equipped weapon for combat effectiveness
                    let attacker_weapon = attacker_player_data.current_loadout.primary_weapon
                        .unwrap_or(attacker_player_data.current_loadout.secondary_weapon.clone());
                    let defender_weapon = defender_player_data.current_loadout.primary_weapon
                        .unwrap_or(defender_player_data.current_loadout.secondary_weapon.clone());

                    // Calculate weapon effectiveness multipliers
                    let attacker_weapon_effectiveness = self.calculate_weapon_effectiveness(&attacker_weapon);
                    let defender_weapon_effectiveness = self.calculate_weapon_effectiveness(&defender_weapon);

                    // Enhanced combat calculation with weapon stats
                    let attacker_base_skill = attacker_player_data.aim_skill * 0.7 + attacker_player_data.hs_skill * 0.3;
                    let defender_base_skill = defender_player_data.aim_skill * 0.7 + defender_player_data.hs_skill * 0.3;

                    let attacker_effective_skill = attacker_base_skill * attacker_weapon_effectiveness;
                    let defender_effective_skill = defender_base_skill * defender_weapon_effectiveness;

                    // Fire rate advantage
                    let attacker_fire_rate = self.weapon_stats[&attacker_weapon].fire_rate;
                    let defender_fire_rate = self.weapon_stats[&defender_weapon].fire_rate;
                    
                    let fire_rate_advantage = (attacker_fire_rate / defender_fire_rate).min(2.0).max(0.5);

                    let mut attacker_win_chance = 0.5 + (attacker_effective_skill - defender_effective_skill) * 0.3;
                    attacker_win_chance *= fire_rate_advantage;
                    attacker_win_chance = attacker_win_chance.clamp(0.1f32, 0.9f32);

                    // Determine hit location and headshot
                    let is_attacker_headshot = rng.random::<f32>() < attacker_player_data.hs_skill;
                    let is_defender_headshot = rng.random::<f32>() < defender_player_data.hs_skill;

                    let hit_body_part = if is_attacker_headshot || is_defender_headshot { 
                        BodyPart::Head 
                    } else if rng.random::<f32>() < 0.7 { 
                        BodyPart::Body 
                    } else { 
                        BodyPart::Legs 
                    };

                    // Simulate engagement range (10-50 meters)
                    let engagement_range = rng.gen_range(10.0..50.0);

                    if rng.random::<f32>() < attacker_win_chance {
                        // Attacker wins
                        let damage = self.calculate_weapon_damage(
                            &attacker_weapon, 
                            &defender_player_data.current_loadout.armor, 
                            hit_body_part, 
                            engagement_range
                        );
                        
                        if let Some(victim) = self.players.get_mut(&defender_id) {
                            victim.take_damage(damage);
                        }
                        
                        // Only record kill if both killer is alive and victim actually died
                        if let (Some(killer), Some(victim)) = (self.players.get(&attacker_id), self.players.get(&defender_id)) {
                            if killer.is_alive && !victim.is_alive {
                                self.record_event(GameEvent::Kill {
                                    timestamp: self.current_timestamp,
                                    killer_id: attacker_id,
                                    victim_id: defender_id,
                                    weapon: attacker_weapon,
                                    is_headshot: is_attacker_headshot,
                                });
                                self.award_kill_bonus(attacker_id);
                            }
                        }
                    } else {
                        // Defender wins
                        let damage = self.calculate_weapon_damage(
                            &defender_weapon, 
                            &attacker_player_data.current_loadout.armor, 
                            hit_body_part, 
                            engagement_range
                        );
                        
                        if let Some(victim) = self.players.get_mut(&attacker_id) {
                            victim.take_damage(damage);
                        }
                        
                        // Only record kill if both killer is alive and victim actually died
                        if let (Some(killer), Some(victim)) = (self.players.get(&defender_id), self.players.get(&attacker_id)) {
                            if killer.is_alive && !victim.is_alive {
                                self.record_event(GameEvent::Kill {
                                    timestamp: self.current_timestamp,
                                    killer_id: defender_id,
                                    victim_id: attacker_id,
                                    weapon: defender_weapon,
                                    is_headshot: is_defender_headshot,
                                });
                                self.award_kill_bonus(defender_id);
                            }
                        }
                    }
                }

                if !spike_planted && self.current_timestamp - round_start_timestamp >= ROUND_MAX_TIME_MS {
                    round_winner = Some(Team::Defenders);
                    round_reason = Some(RoundEndReason::TimeExpired);
                    break 'round_loop;
                }
            }

            let winner = round_winner.unwrap();
            let reason = round_reason.unwrap();
            
            // Reset variables for next round
            round_winner = None;
            round_reason = None;
            spike_planted = false;

            // Award round-end credits
            self.calculate_round_rewards(&winner, &reason, spike_planted);

            if winner == Team::Attackers {
                self.attacker_score += 1;
            } else {
                self.defender_score += 1;
            }

            self.record_event(GameEvent::RoundEnd {
                timestamp: self.current_timestamp,
                round_number: self.current_round,
                winning_team: winner.clone(),
                reason: reason.clone(),
            });

            let current_diff = (self.attacker_score as i16 - self.defender_score as i16).abs();

            // Check for match end conditions
            let match_ended = if self.overtime_active {
                if current_diff >= WIN_MARGIN_OVERTIME as i16 {
                    let winning_team = if self.attacker_score > self.defender_score {
                        Team::Attackers
                    } else {
                        Team::Defenders
                    };
                    self.record_event(GameEvent::MatchEnd {
                        timestamp: self.current_timestamp,
                        winning_team,
                        score_attackers: self.attacker_score,
                        score_defenders: self.defender_score,
                    });
                    true
                } else {
                    false
                }
            } else if self.attacker_score >= WIN_SCORE_REGULAR || self.defender_score >= WIN_SCORE_REGULAR {
                if current_diff >= 2 {
                    let winning_team = if self.attacker_score > self.defender_score {
                        Team::Attackers
                    } else {
                        Team::Defenders
                    };
                    self.record_event(GameEvent::MatchEnd {
                        timestamp: self.current_timestamp,
                        winning_team,
                        score_attackers: self.attacker_score,
                        score_defenders: self.defender_score,
                    });
                    true
                } else if self.attacker_score == MAX_ROUNDS_REGULAR / 2 + 1 && self.defender_score == MAX_ROUNDS_REGULAR / 2 + 1 {
                    self.overtime_active = true;
                    false
                } else {
                    false
                }
            } else if self.current_round >= MAX_ROUNDS_REGULAR && current_diff == 0 {
                self.overtime_active = true;
                false
            } else {
                false
            };

            if match_ended {
                break 'game_loop;
            }

            self.advance_time(1000);
        }
    }
}