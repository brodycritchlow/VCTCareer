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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Team {
    Attackers,
    Defenders,
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
            aim_skill: aim.clamp(0.0, 100.0),
            hs_skill: hs.clamp(0.0, 100.0),
            movement_skill: movement.clamp(0.0, 100.0),
            util_skill: util.clamp(0.0, 100.0),
        }
    }

    pub fn reset_for_round(&mut self) {
        self.current_health = 100;
        self.current_armor = 0;
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
}

impl ValorantSimulation {
    pub fn new() -> Self {
        ValorantSimulation {
            players: HashMap::new(),
            events: Vec::new(),
            current_timestamp: 0,
            current_round: 0,
            attacker_score: 0,
            defender_score: 0,
            overtime_active: false,
        }
    }

    pub fn add_player(&mut self, player: Player) {
        self.players.insert(player.id, player);
    }

    fn advance_time(&mut self, delta_ms: u64) {
        self.current_timestamp += delta_ms;
    }

    fn record_event(&mut self, event: GameEvent) {
        self.events.push(event);
    }

    fn get_alive_players_on_team(&self, team: &Team) -> Vec<&Player> {
        self.players.values()
            .filter(|p| p.team == *team && p.is_alive)
            .collect()
    }

    pub fn run_simulation(&mut self) {
        self.record_event(GameEvent::MatchStart { timestamp: self.current_timestamp });

        const MAX_ROUNDS_REGULAR: u8 = 24;
        const WIN_SCORE_REGULAR: u8 = 13;
        const WIN_MARGIN_OVERTIME: u8 = 2;

        loop {
            self.current_round += 1;
            self.advance_time(5000);

            for player in self.players.values_mut() {
                player.reset_for_round();
                player.current_credits += if self.current_round == 1 { 800 } else { 3000 };
            }

            self.record_event(GameEvent::RoundStart {
                timestamp: self.current_timestamp,
                round_number: self.current_round,
                attacker_credits_start: self.players.values().find(|p| p.team == Team::Attackers).map_or(0, |p| p.current_credits),
                defender_credits_start: self.players.values().find(|p| p.team == Team::Defenders).map_or(0, |p| p.current_credits),
            });

            let mut round_winner: Option<Team> = None;
            let mut round_reason: Option<RoundEndReason> = None;

            let mut spike_planted = false;
            let mut spike_defused = false;
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
                        spike_defused = true;
                        round_winner = Some(Team::Defenders);
                        round_reason = Some(RoundEndReason::SpikeDefused);
                        break 'round_loop;
                    }
                }

                if !alive_attackers_ids.is_empty() && !alive_defenders_ids.is_empty() {
                    let attacker_id = alive_attackers_ids[rng.random_range(0..alive_attackers_ids.len())];
                    let defender_id = alive_defenders_ids[rng.random_range(0..alive_defenders_ids.len())];

                    let attacker_player_data = self.players.get(&attacker_id).unwrap().clone();
                    let defender_player_data = self.players.get(&defender_id).unwrap().clone();

                    let attacker_offense_score = attacker_player_data.aim_skill * 0.7 + attacker_player_data.hs_skill * 0.3;
                    let defender_defense_score = defender_player_data.movement_skill;

                    let mut attacker_win_chance = 0.5 + (attacker_offense_score - defender_defense_score) * 0.4;
                    attacker_win_chance = attacker_win_chance.clamp(0.1, 0.9);

                    let is_attacker_headshot = rng.random::<f32>() < attacker_player_data.hs_skill;
                    let is_defender_headshot = rng.random::<f32>() < defender_player_data.hs_skill;

                    if rng.random::<f32>() < attacker_win_chance {
                        if let Some(victim) = self.players.get_mut(&defender_id) {
                            victim.take_damage(200);
                        }
                        self.record_event(GameEvent::Kill {
                            timestamp: self.current_timestamp,
                            killer_id: attacker_id,
                            victim_id: defender_id,
                            weapon: Weapon::Vandal,
                            is_headshot: is_attacker_headshot,
                        });
                    } else {
                        if let Some(victim) = self.players.get_mut(&attacker_id) {
                            victim.take_damage(200);
                        }
                        self.record_event(GameEvent::Kill {
                            timestamp: self.current_timestamp,
                            killer_id: defender_id,
                            victim_id: attacker_id,
                            weapon: Weapon::Vandal,
                            is_headshot: is_defender_headshot,
                        });
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

            if self.overtime_active {
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
                    break;
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
                    break;
                } else if self.attacker_score == MAX_ROUNDS_REGULAR / 2 + 1 && self.defender_score == MAX_ROUNDS_REGULAR / 2 + 1 {
                    self.overtime_active = true;
                }
            } else if self.current_round >= MAX_ROUNDS_REGULAR && current_diff == 0 {
                self.overtime_active = true;
            }

            self.advance_time(1000);
        }
    }
}

