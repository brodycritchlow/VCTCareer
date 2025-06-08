use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;
use uuid::Uuid;

// Candle ML imports
use candle_core::{Device, Result as CandleResult, Tensor, DType};
use candle_nn::{Module, VarBuilder, VarMap, linear, Linear, ops};

#[derive(Debug, Clone)]
struct PlayerRoundStats {
    kills: u8,
    deaths: u8,
    assists: u8,
    survival_rate: f32,
    avg_damage: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub enum Agent {
    Jett,
    Raze,
    Phoenix,
    Breach,
    Sova,
    Sage,
    Omen,
    Brimstone,
    Viper,
    Cypher,
    Killjoy,
    Skye,
    Yoru,
    Astra,
    Kayo,
    Chamber,
    Neon,
    Fade,
    Harbor,
    Gekko,
    Deadlock,
    Iso,
    Clove,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub enum AgentRole {
    Duelist,
    Initiator,
    Controller,
    Sentinel,
}

impl Agent {
    pub fn get_role(&self) -> AgentRole {
        match self {
            Agent::Jett | Agent::Raze | Agent::Phoenix | Agent::Yoru | Agent::Neon | Agent::Iso => AgentRole::Duelist,
            Agent::Breach | Agent::Sova | Agent::Skye | Agent::Kayo | Agent::Fade | Agent::Gekko => AgentRole::Initiator,
            Agent::Omen | Agent::Brimstone | Agent::Viper | Agent::Astra | Agent::Harbor | Agent::Clove => AgentRole::Controller,
            Agent::Sage | Agent::Cypher | Agent::Killjoy | Agent::Chamber | Agent::Deadlock => AgentRole::Sentinel,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub enum Weapon {
    Classic,
    Shorty,
    Frenzy,
    Ghost,
    Sheriff,
    Stinger,
    Spectre,
    Bucky,
    Judge,
    Bulldog,
    Guardian,
    Phantom,
    Vandal,
    Marshal,
    Operator,
    Ares,
    Odin,
    Knife,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub enum Team {
    Attackers,
    Defenders,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BodyPart {
    Head,
    Body,
    Legs,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub enum ArmorType {
    None,
    Light, // 25 armor, costs 400
    Heavy, // 50 armor, costs 1000
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Penetration {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum RoundType {
    Pistol,
    Eco,
    AntiEco,
    FullBuy,
    ForceBuy,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum EconomyState {
    Poor,     // < 2000 avg credits
    Moderate, // 2000-4000 avg credits
    Strong,   // > 4000 avg credits
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct WeaponPriority {
    pub weapon: Weapon,
    pub priority: f32,
    pub min_credits: u32,
    pub situational_modifiers: HashMap<String, f32>, // RoundType as string for HashMap
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct BuyPreferences {
    pub preferred_weapons: Vec<WeaponPriority>,
    pub eco_threshold: u32,
    pub force_buy_tendency: f32,
    pub utility_priority: f32,
    pub armor_priority: f32,
    pub role_weapon_preferences: HashMap<String, Vec<Weapon>>, // AgentRole as string for HashMap
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct RoundContext {
    pub round_type: RoundType,
    pub team_economy: u32,
    pub enemy_predicted_economy: EconomyState,
    pub previous_round_result: Option<RoundEndReason>,
    pub loss_streak: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct BuyDecision {
    pub primary_weapon: Option<Weapon>,
    pub secondary_weapon: Weapon,
    pub armor: ArmorType,
    pub abilities_budget: u32,
    pub total_cost: u32,
    pub confidence: f32,
    pub coordination_priority: f32, // How important this player's buy is for team success
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TeamBuyStrategy {
    pub strategy_type: TeamStrategyType,
    pub priority_roles: Vec<AgentRole>, // Which roles get priority in buying
    pub utility_budget: u32,
    pub minimum_rifles: u8,
    pub allow_eco_frags: bool, // Allow some players to buy while others save
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum TeamStrategyType {
    FullSave,    // Everyone saves for next round
    EcoFrag,     // Some players buy minimal weapons to get frags
    HalfBuy,     // Buy weapons but minimal utility/armor
    FullBuy,     // Everyone buys optimal loadouts
    ForceBuy,    // Buy everything despite poor economy
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UtilityBudget {
    pub smokes_budget: u32,
    pub flashes_budget: u32,
    pub info_budget: u32,
    pub healing_budget: u32,
    pub total_utility_spend: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TeamComposition {
    pub has_smoker: bool,
    pub has_igl: bool, // In-game leader (typically Controller/Sentinel)
    pub has_entry_fragger: bool,
    pub has_support: bool,
    pub rifle_players: u8,
    pub operator_players: u8,
}

// Phase 3: Machine Learning Integration Data Structures

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct BuyPerformanceMetrics {
    #[schema(value_type = String)]
    pub decision_id: Uuid,
    pub player_id: u32,
    pub round_number: u8,
    pub buy_decision: BuyDecision,
    pub round_context: RoundContext,
    pub performance_outcome: Option<RoundPerformanceOutcome>,
    pub timestamp: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct RoundPerformanceOutcome {
    pub round_won: bool,
    pub kills: u8,
    pub deaths: u8,
    pub assists: u8,
    pub damage_dealt: u32,
    pub damage_taken: u32,
    pub clutch_situation: bool,
    pub first_kill: bool,
    pub multi_kill: bool,
    pub round_impact_score: f32, // 0.0 to 1.0 rating of round contribution
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct LearningWeights {
    pub weapon_success_rate: f32,
    pub economic_efficiency: f32,
    pub team_synergy_bonus: f32,
    pub situational_adaptation: f32,
    pub recent_performance_bias: f32, // How much to weight recent vs historical data
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct AdaptiveBuyPreferences {
    pub base_preferences: BuyPreferences,
    pub learned_adjustments: HashMap<String, f32>, // Contextual adjustments to base preferences
    pub success_rates: HashMap<String, WeaponSuccessRate>, // Per-weapon success tracking
    pub learning_weights: LearningWeights,
    pub confidence_threshold: f32, // Minimum confidence to deviate from base preferences
    pub adaptation_rate: f32, // How quickly to adapt to new data (0.0 to 1.0)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct WeaponSuccessRate {
    pub weapon: Weapon,
    pub total_rounds: u32,
    pub successful_rounds: u32, // Rounds where the weapon choice led to positive outcomes
    pub average_impact: f32,
    pub context_success: HashMap<String, ContextualSuccess>, // Success by round type, economy, etc.
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct ContextualSuccess {
    pub context_key: String, // e.g., "FullBuy_Strong_Economy", "Eco_Poor_Economy"
    pub success_count: u32,
    pub total_count: u32,
    pub average_performance: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct PlayerLearningProfile {
    pub player_id: u32,
    pub adaptive_preferences: AdaptiveBuyPreferences,
    pub playstyle_pattern: PlaystylePattern,
    pub meta_adaptation: MetaAdaptation,
    pub performance_history: Vec<BuyPerformanceMetrics>,
    pub last_updated: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum PlaystylePattern {
    Aggressive,      // Prefers high-impact weapons, higher force buy tendency
    Conservative,    // Saves more, prefers reliable weapons
    Adaptive,        // Changes style based on game state
    Supportive,      // Prioritizes team utility over individual fragging
    Experimental,    // Tries different weapons/strategies more often
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct MetaAdaptation {
    pub current_meta_score: f32, // How well player's style fits current meta
    pub meta_trends: Vec<MetaTrend>,
    pub adaptation_suggestions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct MetaTrend {
    pub trend_type: String, // e.g., "OperatorMeta", "EcoRush", "UtilityFocus"
    pub strength: f32, // How strong this trend is (0.0 to 1.0)
    pub player_alignment: f32, // How well player aligns with this trend
    pub impact_on_success: f32, // Measured impact on win rate
}

#[derive(Debug, Clone)]
pub struct WeaponStats {
    pub price: u32,
    pub damage_head: (u32, u32, u32), // no armor, light armor, heavy armor
    pub damage_body: (u32, u32, u32),
    pub damage_legs: (u32, u32, u32),
    pub fire_rate: f32, // rounds per second
    pub penetration: Penetration,
    pub magazine_size: u32,
    pub reload_time_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerLoadout {
    pub primary_weapon: Option<Weapon>,
    pub secondary_weapon: Weapon, // Always have Classic minimum
    pub armor: ArmorType,
    pub abilities_purchased: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerSkills {
    pub aim: f32,
    pub hs: f32,
    pub movement: f32,
    pub util: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PlayerLearningInsights {
    pub player_id: u32,
    pub total_rounds_analyzed: u32,
    pub current_playstyle: PlaystylePattern,
    pub recent_avg_impact: f32,
    pub most_successful_weapon: Option<Weapon>,
    pub adaptation_rate: f32,
    pub learning_trend: String, // "Improving", "Declining", "Stable", "Learning"
    pub confidence_score: f32,
}

// Phase 3.5: Candle Neural Network Integration

#[derive(Debug, Clone)]
pub struct BuyDecisionNetwork {
    pub input_layer: Linear,
    pub hidden_layer1: Linear,
    pub hidden_layer2: Linear,
    pub output_layer: Linear,
    pub device: Device,
}

impl BuyDecisionNetwork {
    const INPUT_SIZE: usize = 32; // Player features + game state + context
    const HIDDEN_SIZE1: usize = 64;
    const HIDDEN_SIZE2: usize = 32;
    const OUTPUT_SIZE: usize = 15; // Weapon confidences + eco decision + armor choices

    pub fn new(vs: VarBuilder) -> CandleResult<Self> {
        let input_layer = linear(Self::INPUT_SIZE, Self::HIDDEN_SIZE1, vs.pp("input"))?;
        let hidden_layer1 = linear(Self::HIDDEN_SIZE1, Self::HIDDEN_SIZE2, vs.pp("hidden1"))?;
        let hidden_layer2 = linear(Self::HIDDEN_SIZE2, Self::HIDDEN_SIZE2, vs.pp("hidden2"))?;
        let output_layer = linear(Self::HIDDEN_SIZE2, Self::OUTPUT_SIZE, vs.pp("output"))?;
        
        Ok(Self {
            input_layer,
            hidden_layer1,
            hidden_layer2,
            output_layer,
            device: vs.device().clone(),
        })
    }

    pub fn forward(&self, input: &Tensor) -> CandleResult<Tensor> {
        let x = self.input_layer.forward(input)?;
        let x = x.relu()?;
        
        let x = self.hidden_layer1.forward(&x)?;
        let x = x.relu()?;
        
        let x = self.hidden_layer2.forward(&x)?;
        let x = x.relu()?;
        
        let output = self.output_layer.forward(&x)?;
        ops::sigmoid(&output) // Output probabilities/confidences
    }
}

#[derive(Debug, Clone)]
pub struct GameStateFeatures {
    // Player features (10 dimensions)
    pub player_credits: f32,
    pub player_kills: f32,
    pub player_deaths: f32,
    pub player_assists: f32,
    pub player_survival_rate: f32,
    pub player_avg_damage: f32,
    pub player_headshot_rate: f32,
    pub player_aim_skill: f32,
    pub player_utility_skill: f32,
    pub player_movement_skill: f32,
    
    // Game context features (12 dimensions)
    pub round_number: f32,
    pub team_score: f32,
    pub enemy_score: f32,
    pub team_economy: f32,
    pub enemy_economy: f32,
    pub loss_streak: f32,
    pub round_type_pistol: f32,
    pub round_type_eco: f32,
    pub round_type_fullbuy: f32,
    pub round_type_forcebuy: f32,
    pub round_type_antieco: f32,
    pub map_side: f32, // 0.0 for attacker, 1.0 for defender
    
    // Role features (4 dimensions)
    pub is_duelist: f32,
    pub is_controller: f32,
    pub is_initiator: f32,
    pub is_sentinel: f32,
    
    // Team coordination features (6 dimensions)
    pub team_rifles_needed: f32,
    pub team_utility_budget: f32,
    pub coordination_priority: f32,
    pub team_strategy_eco: f32,
    pub team_strategy_force: f32,
    pub team_strategy_full: f32,
}

impl GameStateFeatures {
    pub fn to_tensor(&self, device: &Device) -> CandleResult<Tensor> {
        let features = vec![
            // Player features
            self.player_credits,
            self.player_kills,
            self.player_deaths,
            self.player_assists,
            self.player_survival_rate,
            self.player_avg_damage,
            self.player_headshot_rate,
            self.player_aim_skill,
            self.player_utility_skill,
            self.player_movement_skill,
            
            // Game context features
            self.round_number,
            self.team_score,
            self.enemy_score,
            self.team_economy,
            self.enemy_economy,
            self.loss_streak,
            self.round_type_pistol,
            self.round_type_eco,
            self.round_type_fullbuy,
            self.round_type_forcebuy,
            self.round_type_antieco,
            self.map_side,
            
            // Role features
            self.is_duelist,
            self.is_controller,
            self.is_initiator,
            self.is_sentinel,
            
            // Team coordination features
            self.team_rifles_needed,
            self.team_utility_budget,
            self.coordination_priority,
            self.team_strategy_eco,
            self.team_strategy_force,
            self.team_strategy_full,
        ];
        
        Tensor::from_vec(features, (1, BuyDecisionNetwork::INPUT_SIZE), device)
    }
}

#[derive(Clone)]
pub struct NeuralBuyPredictor {
    pub network: BuyDecisionNetwork,
    pub var_map: VarMap,
    pub device: Device,
    pub training_data: Vec<(GameStateFeatures, BuyDecisionTarget)>,
    pub learning_rate: f64,
}

#[derive(Debug, Clone)]
pub struct BuyDecisionTarget {
    // Target outputs (15 dimensions matching network output)
    pub weapon_confidences: Vec<f32>, // 10 weapons
    pub should_eco: f32,
    pub should_force: f32,
    pub armor_priority: f32,
    pub utility_priority: f32,
    pub overall_confidence: f32,
}

impl NeuralBuyPredictor {
    pub fn new() -> CandleResult<Self> {
        let device = Device::Cpu; // Can be changed to GPU if available
        let var_map = VarMap::new();
        let vs = VarBuilder::from_varmap(&var_map, DType::F32, &device);
        
        let network = BuyDecisionNetwork::new(vs)?;
        
        Ok(Self {
            network,
            var_map,
            device,
            training_data: Vec::new(),
            learning_rate: 0.001,
        })
    }

    pub fn predict(&self, features: &GameStateFeatures) -> CandleResult<BuyDecisionTarget> {
        let input = features.to_tensor(&self.device)?;
        let output = self.network.forward(&input)?;
        
        let output_vec = output.flatten_all()?.to_vec1::<f32>()?;
        
        Ok(BuyDecisionTarget {
            weapon_confidences: output_vec[0..10].to_vec(),
            should_eco: output_vec[10],
            should_force: output_vec[11],
            armor_priority: output_vec[12],
            utility_priority: output_vec[13],
            overall_confidence: output_vec[14],
        })
    }

    pub fn add_training_example(&mut self, features: GameStateFeatures, target: BuyDecisionTarget) {
        self.training_data.push((features, target));
        
        // Keep only recent training data (last 1000 examples)
        if self.training_data.len() > 1000 {
            self.training_data.remove(0);
        }
    }

    pub fn train_step(&mut self) -> CandleResult<f32> {
        if self.training_data.len() < 10 {
            return Ok(0.0); // Not enough data to train
        }

        // Sample a batch of training data
        let batch_size = (self.training_data.len().min(32)) as usize;
        let mut batch_features = Vec::new();
        let mut batch_targets = Vec::new();
        
        for _i in 0..batch_size {
            let idx = rand::Rng::random_range(&mut rand::rng(), 0..self.training_data.len());
            let (features, target) = &self.training_data[idx];
            batch_features.push(features.clone());
            batch_targets.push(target.clone());
        }

        // Convert to tensors
        let mut input_data = Vec::new();
        let mut target_data = Vec::new();
        
        for (features, target) in batch_features.iter().zip(batch_targets.iter()) {
            let input_tensor = features.to_tensor(&self.device)?;
            input_data.push(input_tensor.flatten_all()?.to_vec1::<f32>()?);
            
            let mut target_vec = target.weapon_confidences.clone();
            target_vec.extend_from_slice(&[
                target.should_eco,
                target.should_force,
                target.armor_priority,
                target.utility_priority,
                target.overall_confidence,
            ]);
            target_data.push(target_vec);
        }

        // Create batch tensors
        let input_batch = Tensor::from_vec(
            input_data.into_iter().flatten().collect::<Vec<f32>>(),
            (batch_size, BuyDecisionNetwork::INPUT_SIZE),
            &self.device,
        )?;
        
        let target_batch = Tensor::from_vec(
            target_data.into_iter().flatten().collect::<Vec<f32>>(),
            (batch_size, BuyDecisionNetwork::OUTPUT_SIZE),
            &self.device,
        )?;

        // Forward pass
        let predicted = self.network.forward(&input_batch)?;
        
        // Calculate MSE loss
        let loss = predicted.sub(&target_batch)?.sqr()?.mean_all()?;
        let loss_value = loss.to_scalar::<f32>()?;

        // TODO: Implement backward pass and weight updates
        // This would require gradient computation which is more complex in Candle
        // For now, we'll return the loss for monitoring
        
        Ok(loss_value)
    }

    pub fn get_learning_insights(&self) -> Option<PlayerLearningInsights> {
        if self.training_data.is_empty() {
            return None;
        }

        // Calculate training progress metrics
        let total_rounds_analyzed = self.training_data.len() as u32;
        
        // Analyze recent performance (last 20 training examples)
        let recent_data = self.training_data
            .iter()
            .rev()
            .take(20)
            .collect::<Vec<_>>();
        
        let recent_avg_impact = if recent_data.is_empty() {
            0.0
        } else {
            recent_data
                .iter()
                .map(|(_, target)| target.overall_confidence)
                .sum::<f32>() / recent_data.len() as f32
        };

        // Determine playstyle based on weapon preferences in training data
        let mut weapon_usage = std::collections::HashMap::new();
        for (_, target) in &self.training_data {
            for (i, confidence) in target.weapon_confidences.iter().enumerate() {
                if *confidence > 0.5 {
                    *weapon_usage.entry(i).or_insert(0) += 1;
                }
            }
        }

        // Map most used weapon index to actual weapon
        let most_successful_weapon = weapon_usage
            .iter()
            .max_by_key(|(_, &count)| count)
            .and_then(|(&weapon_idx, _)| {
                match weapon_idx {
                    0 => Some(Weapon::Classic),
                    1 => Some(Weapon::Ghost),
                    2 => Some(Weapon::Sheriff),
                    3 => Some(Weapon::Spectre),
                    4 => Some(Weapon::Bulldog),
                    5 => Some(Weapon::Guardian),
                    6 => Some(Weapon::Phantom),
                    7 => Some(Weapon::Vandal),
                    8 => Some(Weapon::Marshal),
                    9 => Some(Weapon::Operator),
                    _ => None,
                }
            });

        // Determine playstyle based on weapon preferences and behavior patterns
        let current_playstyle = if weapon_usage.get(&9).unwrap_or(&0) > &(total_rounds_analyzed / 3) {
            // Frequent Operator usage
            PlaystylePattern::Conservative
        } else if weapon_usage.get(&6).unwrap_or(&0) + weapon_usage.get(&7).unwrap_or(&0) > (total_rounds_analyzed / 2) {
            // Frequent rifle usage
            PlaystylePattern::Aggressive
        } else {
            // Mixed weapon usage
            PlaystylePattern::Adaptive
        };

        // Calculate learning trend based on recent vs earlier performance
        let learning_trend = if total_rounds_analyzed >= 40 {
            let early_avg = self.training_data
                .iter()
                .take(20)
                .map(|(_, target)| target.overall_confidence)
                .sum::<f32>() / 20.0;
            
            let improvement = recent_avg_impact - early_avg;
            if improvement > 0.1 {
                "Improving".to_string()
            } else if improvement < -0.1 {
                "Declining".to_string()
            } else {
                "Stable".to_string()
            }
        } else {
            "Learning".to_string()
        };

        Some(PlayerLearningInsights {
            player_id: 0, // Will be set by the caller
            total_rounds_analyzed,
            current_playstyle,
            recent_avg_impact,
            most_successful_weapon,
            adaptation_rate: self.learning_rate as f32,
            learning_trend,
            confidence_score: recent_avg_impact,
        })
    }
}

#[derive(Clone, Serialize, Deserialize)]
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

    pub skills: PlayerSkills,
    pub buy_preferences: BuyPreferences,
    // Phase 3: ML Integration
    pub learning_profile: Option<PlayerLearningProfile>,
    // Phase 3.5: Neural Network Integration
    #[serde(skip)] // Don't serialize the neural network
    pub neural_predictor: Option<NeuralBuyPredictor>,
}

impl Player {
    pub fn new(id: u32, name: String, agent: Agent, team: Team, skills: PlayerSkills) -> Self {
        let buy_preferences = Self::generate_buy_preferences_for_agent(&agent, &skills);
        
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
            skills,
            buy_preferences,
            learning_profile: None, // Initialize without ML profile, will be created when ML is enabled
            neural_predictor: None, // Initialize without neural network, will be created when enabled
        }
    }

    fn generate_buy_preferences_for_agent(agent: &Agent, skills: &PlayerSkills) -> BuyPreferences {
        let role = agent.get_role();
        let mut preferred_weapons = Vec::new();
        
        // Generate weapon preferences based on agent role and player skills
        match role {
            AgentRole::Duelist => {
                // Duelists prefer rifles and high-damage weapons
                preferred_weapons.push(WeaponPriority {
                    weapon: Weapon::Vandal,
                    priority: 0.9 + skills.aim * 0.1,
                    min_credits: 2900,
                    situational_modifiers: HashMap::new(),
                });
                preferred_weapons.push(WeaponPriority {
                    weapon: Weapon::Phantom,
                    priority: 0.85 + skills.aim * 0.1,
                    min_credits: 2900,
                    situational_modifiers: HashMap::new(),
                });
                preferred_weapons.push(WeaponPriority {
                    weapon: Weapon::Operator,
                    priority: 0.6 + skills.aim * 0.3,
                    min_credits: 4700,
                    situational_modifiers: HashMap::new(),
                });
                preferred_weapons.push(WeaponPriority {
                    weapon: Weapon::Spectre,
                    priority: 0.7,
                    min_credits: 1600,
                    situational_modifiers: HashMap::new(),
                });
            },
            AgentRole::Controller => {
                // Controllers balance utility and weapons
                preferred_weapons.push(WeaponPriority {
                    weapon: Weapon::Phantom,
                    priority: 0.8,
                    min_credits: 2900,
                    situational_modifiers: HashMap::new(),
                });
                preferred_weapons.push(WeaponPriority {
                    weapon: Weapon::Vandal,
                    priority: 0.75,
                    min_credits: 2900,
                    situational_modifiers: HashMap::new(),
                });
                preferred_weapons.push(WeaponPriority {
                    weapon: Weapon::Guardian,
                    priority: 0.6,
                    min_credits: 2250,
                    situational_modifiers: HashMap::new(),
                });
            },
            AgentRole::Initiator => {
                // Initiators prefer versatile weapons
                preferred_weapons.push(WeaponPriority {
                    weapon: Weapon::Phantom,
                    priority: 0.85,
                    min_credits: 2900,
                    situational_modifiers: HashMap::new(),
                });
                preferred_weapons.push(WeaponPriority {
                    weapon: Weapon::Vandal,
                    priority: 0.8,
                    min_credits: 2900,
                    situational_modifiers: HashMap::new(),
                });
                preferred_weapons.push(WeaponPriority {
                    weapon: Weapon::Bulldog,
                    priority: 0.65,
                    min_credits: 2050,
                    situational_modifiers: HashMap::new(),
                });
            },
            AgentRole::Sentinel => {
                // Sentinels prefer defensive weapons and save economy
                preferred_weapons.push(WeaponPriority {
                    weapon: Weapon::Operator,
                    priority: 0.7 + skills.aim * 0.2,
                    min_credits: 4700,
                    situational_modifiers: HashMap::new(),
                });
                preferred_weapons.push(WeaponPriority {
                    weapon: Weapon::Guardian,
                    priority: 0.75,
                    min_credits: 2250,
                    situational_modifiers: HashMap::new(),
                });
                preferred_weapons.push(WeaponPriority {
                    weapon: Weapon::Vandal,
                    priority: 0.7,
                    min_credits: 2900,
                    situational_modifiers: HashMap::new(),
                });
            },
        }

        // Add secondary weapon preferences
        preferred_weapons.push(WeaponPriority {
            weapon: Weapon::Sheriff,
            priority: 0.6 + skills.aim * 0.2,
            min_credits: 800,
            situational_modifiers: HashMap::new(),
        });
        preferred_weapons.push(WeaponPriority {
            weapon: Weapon::Ghost,
            priority: 0.5,
            min_credits: 500,
            situational_modifiers: HashMap::new(),
        });

        BuyPreferences {
            preferred_weapons,
            eco_threshold: match role {
                AgentRole::Duelist => 2000,     // Aggressive, lower eco threshold
                AgentRole::Controller => 2500,  // Moderate eco threshold
                AgentRole::Initiator => 2200,   // Moderate eco threshold  
                AgentRole::Sentinel => 3000,    // Conservative, higher eco threshold
            },
            force_buy_tendency: match role {
                AgentRole::Duelist => 0.7,      // High force buy tendency
                AgentRole::Controller => 0.4,   // Low force buy tendency
                AgentRole::Initiator => 0.5,    // Moderate force buy tendency
                AgentRole::Sentinel => 0.3,     // Very low force buy tendency
            },
            utility_priority: match role {
                AgentRole::Controller => 0.8,   // High utility priority
                AgentRole::Initiator => 0.7,    // High utility priority
                AgentRole::Sentinel => 0.6,     // Moderate utility priority
                AgentRole::Duelist => 0.3,      // Low utility priority
            },
            armor_priority: 0.8, // Generally high armor priority for all roles
            role_weapon_preferences: HashMap::new(), // Will be populated later if needed
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
        if !self.is_alive {
            return;
        }

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

    // Phase 3: Machine Learning Methods

    pub fn enable_neural_learning(&mut self) -> Result<(), String> {
        match NeuralBuyPredictor::new() {
            Ok(predictor) => {
                self.neural_predictor = Some(predictor);
                Ok(())
            }
            Err(e) => Err(format!("Failed to initialize neural network: {:?}", e))
        }
    }

    pub fn enable_machine_learning(&mut self, current_timestamp: Timestamp) {
        if self.learning_profile.is_none() {
            self.learning_profile = Some(Self::create_initial_learning_profile(
                self.id,
                &self.buy_preferences,
                &self.agent,
                current_timestamp,
            ));
        }
    }

    fn create_initial_learning_profile(
        player_id: u32,
        base_preferences: &BuyPreferences,
        agent: &Agent,
        timestamp: Timestamp,
    ) -> PlayerLearningProfile {
        let role = agent.get_role();
        
        // Initialize adaptive preferences based on role
        let adaptive_preferences = AdaptiveBuyPreferences {
            base_preferences: base_preferences.clone(),
            learned_adjustments: HashMap::new(),
            success_rates: HashMap::new(),
            learning_weights: LearningWeights {
                weapon_success_rate: 0.4,
                economic_efficiency: 0.25,
                team_synergy_bonus: 0.2,
                situational_adaptation: 0.1,
                recent_performance_bias: 0.05,
            },
            confidence_threshold: 0.7,
            adaptation_rate: match role {
                AgentRole::Duelist => 0.3,      // Aggressive learners
                AgentRole::Controller => 0.15,  // Conservative learners
                AgentRole::Initiator => 0.25,   // Moderate learners
                AgentRole::Sentinel => 0.2,     // Deliberate learners
            },
        };

        // Determine initial playstyle pattern based on role and base preferences
        let playstyle_pattern = match role {
            AgentRole::Duelist => PlaystylePattern::Aggressive,
            AgentRole::Controller => PlaystylePattern::Supportive,
            AgentRole::Initiator => PlaystylePattern::Adaptive,
            AgentRole::Sentinel => PlaystylePattern::Conservative,
        };

        let meta_adaptation = MetaAdaptation {
            current_meta_score: 0.5, // Neutral starting score
            meta_trends: Vec::new(),
            adaptation_suggestions: Vec::new(),
        };

        PlayerLearningProfile {
            player_id,
            adaptive_preferences,
            playstyle_pattern,
            meta_adaptation,
            performance_history: Vec::new(),
            last_updated: timestamp,
        }
    }

    pub fn record_buy_performance(&mut self, 
        decision_id: Uuid,
        buy_decision: BuyDecision,
        round_context: RoundContext,
        performance_outcome: Option<RoundPerformanceOutcome>,
        timestamp: Timestamp,
    ) {
        if let Some(ref mut profile) = self.learning_profile {
            let metrics = BuyPerformanceMetrics {
                decision_id,
                player_id: self.id,
                round_number: round_context.round_type as u8, // Approximation
                buy_decision,
                round_context: round_context.clone(),
                performance_outcome,
                timestamp,
            };

            profile.performance_history.push(metrics);
            profile.last_updated = timestamp;

            // Keep only recent history (last 100 rounds) for performance
            if profile.performance_history.len() > 100 {
                profile.performance_history.remove(0);
            }
        }
    }

    pub fn update_weapon_success_rates(&mut self) {
        if let Some(ref mut profile) = self.learning_profile {
            // Clear existing success rates
            profile.adaptive_preferences.success_rates.clear();

            // Group performance by weapon
            let mut weapon_data: HashMap<Weapon, Vec<&BuyPerformanceMetrics>> = HashMap::new();
            
            for metrics in &profile.performance_history {
                if let Some(weapon) = &metrics.buy_decision.primary_weapon {
                    weapon_data.entry(weapon.clone()).or_insert_with(Vec::new).push(metrics);
                }
            }

            // Calculate success rates for each weapon
            for (weapon, metrics_list) in weapon_data {
                let total_rounds = metrics_list.len() as u32;
                let successful_rounds = metrics_list.iter()
                    .filter(|m| {
                        if let Some(outcome) = &m.performance_outcome {
                            outcome.round_impact_score > 0.5 // Consider 0.5+ as successful
                        } else {
                            false
                        }
                    })
                    .count() as u32;

                let average_impact = metrics_list.iter()
                    .filter_map(|m| m.performance_outcome.as_ref())
                    .map(|o| o.round_impact_score)
                    .sum::<f32>() / total_rounds as f32;

                // Build contextual success data
                let mut context_success = HashMap::new();
                Self::build_contextual_success_data_static(&metrics_list, &mut context_success);

                let success_rate = WeaponSuccessRate {
                    weapon: weapon.clone(),
                    total_rounds,
                    successful_rounds,
                    average_impact,
                    context_success,
                };

                profile.adaptive_preferences.success_rates.insert(
                    format!("{:?}", weapon), 
                    success_rate
                );
            }
        }
    }

    fn build_contextual_success_data_static(
        metrics_list: &[&BuyPerformanceMetrics],
        context_success: &mut HashMap<String, ContextualSuccess>,
    ) {
        for metrics in metrics_list {
            let context_key = format!(
                "{:?}_{:?}", 
                metrics.round_context.round_type,
                metrics.round_context.enemy_predicted_economy
            );

            let entry = context_success.entry(context_key.clone()).or_insert(ContextualSuccess {
                context_key: context_key.clone(),
                success_count: 0,
                total_count: 0,
                average_performance: 0.0,
            });

            entry.total_count += 1;
            if let Some(outcome) = &metrics.performance_outcome {
                if outcome.round_impact_score > 0.5 {
                    entry.success_count += 1;
                }
                entry.average_performance = 
                    ((entry.average_performance * (entry.total_count - 1) as f32) + outcome.round_impact_score) 
                    / entry.total_count as f32;
            }
        }
    }

    pub fn adapt_buy_preferences(&mut self) {
        if let Some(ref mut profile) = self.learning_profile {
            // Only adapt if we have sufficient data
            if profile.performance_history.len() < 10 {
                return;
            }

            let adaptation_rate = profile.adaptive_preferences.adaptation_rate;
            
            // Analyze recent performance trends
            let recent_performance: Vec<&BuyPerformanceMetrics> = profile.performance_history
                .iter()
                .rev()
                .take(20) // Last 20 rounds
                .collect();

            // Update learned adjustments based on performance
            for weapon_name in profile.adaptive_preferences.success_rates.keys() {
                if let Some(success_rate) = profile.adaptive_preferences.success_rates.get(weapon_name) {
                    let success_ratio = success_rate.successful_rounds as f32 / success_rate.total_rounds as f32;
                    
                    // Adjust weapon priority based on success rate
                    let adjustment = if success_ratio > 0.7 {
                        adaptation_rate * 0.1 // Increase priority for successful weapons
                    } else if success_ratio < 0.3 {
                        -adaptation_rate * 0.1 // Decrease priority for unsuccessful weapons
                    } else {
                        0.0 // No change for neutral performance
                    };

                    profile.adaptive_preferences.learned_adjustments.insert(
                        weapon_name.clone(),
                        adjustment,
                    );
                }
            }

            // Update playstyle pattern based on recent performance
            Self::update_playstyle_pattern_static(&mut profile.playstyle_pattern, &recent_performance);
        }
    }

    fn update_playstyle_pattern_static(
        playstyle_pattern: &mut PlaystylePattern,
        recent_performance: &[&BuyPerformanceMetrics]
    ) {
        // Analyze patterns in recent buy decisions
        let aggressive_decisions = recent_performance.iter()
            .filter(|m| {
                matches!(m.buy_decision.primary_weapon, 
                         Some(Weapon::Operator) | Some(Weapon::Vandal) | Some(Weapon::Phantom))
            })
            .count();

        let conservative_decisions = recent_performance.iter()
            .filter(|m| m.buy_decision.total_cost < 2000)
            .count();

        let total_decisions = recent_performance.len();

        if aggressive_decisions as f32 / total_decisions as f32 > 0.7 {
            *playstyle_pattern = PlaystylePattern::Aggressive;
        } else if conservative_decisions as f32 / total_decisions as f32 > 0.6 {
            *playstyle_pattern = PlaystylePattern::Conservative;
        } else {
            *playstyle_pattern = PlaystylePattern::Adaptive;
        }
    }
}

pub type Timestamp = u64;

// Helper functions for UUID serialization in schemas
fn serialize_uuid<S>(uuid: &Uuid, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&uuid.to_string())
}

fn deserialize_uuid<'de, D>(deserializer: D) -> Result<Uuid, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Uuid::parse_str(&s).map_err(serde::de::Error::custom)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum SimulationMode {
    Paused,
    Playing,
    FastForward,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum SimulationPhase {
    NotStarted,
    BuyPhase {
        round_number: u8,
    },
    RoundActive {
        round_number: u8,
        spike_planted: bool,
    },
    RoundEnd {
        round_number: u8,
        winner: Team,
    },
    MatchEnd {
        winner: Team,
        final_score: (u8, u8),
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SimulationState {
    #[serde(
        serialize_with = "serialize_uuid",
        deserialize_with = "deserialize_uuid"
    )]
    #[schema(value_type = String)]
    pub id: Uuid,
    pub mode: SimulationMode,
    pub phase: SimulationPhase,
    pub playback_speed: f32,
    pub current_timestamp: Timestamp,
    pub current_round: u8,
    pub attacker_score: u8,
    pub defender_score: u8,
    pub overtime_active: bool,
    pub tick_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EventFilter {
    pub event_types: Option<Vec<String>>,
    pub player_ids: Option<Vec<u32>>,
    pub round_numbers: Option<Vec<u8>>,
    pub start_timestamp: Option<Timestamp>,
    pub end_timestamp: Option<Timestamp>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PlayerStats {
    pub player_id: u32,
    pub kills: u32,
    pub deaths: u32,
    pub assists: u32,
    pub damage_dealt: u32,
    pub headshot_percentage: f32,
    pub credits: u32,
    pub ultimate_points: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum GameEvent {
    MatchStart {
        timestamp: Timestamp,
    },
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum RoundEndReason {
    AllAttackersEliminated,
    AllDefendersEliminated,
    SpikeDetonated,
    SpikeDefused,
    TimeExpired,
}

impl GameEvent {
    pub fn timestamp(&self) -> Timestamp {
        match self {
            GameEvent::MatchStart { timestamp } => *timestamp,
            GameEvent::MatchEnd { timestamp, .. } => *timestamp,
            GameEvent::BuyPhaseStart { timestamp, .. } => *timestamp,
            GameEvent::BuyPhaseEnd { timestamp, .. } => *timestamp,
            GameEvent::RoundStart { timestamp, .. } => *timestamp,
            GameEvent::RoundEnd { timestamp, .. } => *timestamp,
            GameEvent::Kill { timestamp, .. } => *timestamp,
            GameEvent::Damage { timestamp, .. } => *timestamp,
            GameEvent::SpikePlant { timestamp, .. } => *timestamp,
            GameEvent::SpikeDefuse { timestamp, .. } => *timestamp,
            GameEvent::AbilityUsed { timestamp, .. } => *timestamp,
            GameEvent::SideSwap { timestamp, .. } => *timestamp,
        }
    }
}

pub struct ValorantSimulation {
    pub state: SimulationState,
    pub players: HashMap<u32, Player>,
    pub events: Vec<GameEvent>,
    pub loss_streaks: HashMap<Team, u8>,
    pub weapon_stats: HashMap<Weapon, WeaponStats>,

    // New fields for modular control
    pub checkpoints: HashMap<u64, SimulationCheckpoint>,
    pub round_timer_ms: i32,
    pub spike_timer_ms: i32,
    pub spike_planted: bool,
    pub spike_defused: bool,
    pub round_start_timestamp: Timestamp,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SimulationCheckpoint {
    pub state: SimulationState,
    pub players: HashMap<u32, Player>,
    pub events: Vec<GameEvent>,
    pub loss_streaks: HashMap<Team, u8>,
}

#[allow(clippy::new_without_default)]
impl ValorantSimulation {
    pub fn new() -> Self {
        let mut weapon_stats = HashMap::new();

        // Initialize weapon stats based on Valorant data
        weapon_stats.insert(
            Weapon::Classic,
            WeaponStats {
                price: 0,
                damage_head: (78, 66, 26),
                damage_body: (26, 22, 22),
                damage_legs: (22, 18, 18),
                fire_rate: 6.75,
                penetration: Penetration::Low,
                magazine_size: 12,
                reload_time_ms: 2250,
            },
        );

        weapon_stats.insert(
            Weapon::Ghost,
            WeaponStats {
                price: 500,
                damage_head: (105, 87, 30),
                damage_body: (30, 25, 25),
                damage_legs: (25, 21, 21),
                fire_rate: 6.75,
                penetration: Penetration::Medium,
                magazine_size: 15,
                reload_time_ms: 2500,
            },
        );

        weapon_stats.insert(
            Weapon::Sheriff,
            WeaponStats {
                price: 800,
                damage_head: (159, 145, 55),
                damage_body: (55, 50, 46),
                damage_legs: (46, 42, 42),
                fire_rate: 4.0,
                penetration: Penetration::High,
                magazine_size: 6,
                reload_time_ms: 3000,
            },
        );

        weapon_stats.insert(
            Weapon::Spectre,
            WeaponStats {
                price: 1600,
                damage_head: (78, 66, 26),
                damage_body: (26, 22, 22),
                damage_legs: (22, 18, 18),
                fire_rate: 13.33,
                penetration: Penetration::Medium,
                magazine_size: 30,
                reload_time_ms: 2250,
            },
        );

        weapon_stats.insert(
            Weapon::Phantom,
            WeaponStats {
                price: 2900,
                damage_head: (156, 140, 124), // Close range
                damage_body: (39, 35, 31),
                damage_legs: (33, 29, 26),
                fire_rate: 11.0,
                penetration: Penetration::Medium,
                magazine_size: 30,
                reload_time_ms: 2500,
            },
        );

        weapon_stats.insert(
            Weapon::Vandal,
            WeaponStats {
                price: 2900,
                damage_head: (160, 160, 160), // Always 160 regardless of armor
                damage_body: (40, 40, 40),
                damage_legs: (34, 34, 34),
                fire_rate: 9.75,
                penetration: Penetration::Medium,
                magazine_size: 25,
                reload_time_ms: 2500,
            },
        );

        weapon_stats.insert(
            Weapon::Operator,
            WeaponStats {
                price: 4700,
                damage_head: (255, 255, 255),
                damage_body: (150, 150, 150),
                damage_legs: (120, 120, 120),
                fire_rate: 0.75,
                penetration: Penetration::High,
                magazine_size: 5,
                reload_time_ms: 3700,
            },
        );

        weapon_stats.insert(
            Weapon::Guardian,
            WeaponStats {
                price: 2250,
                damage_head: (195, 180, 165),
                damage_body: (65, 60, 55),
                damage_legs: (48, 45, 41),
                fire_rate: 4.75,
                penetration: Penetration::High,
                magazine_size: 12,
                reload_time_ms: 2500,
            },
        );

        weapon_stats.insert(
            Weapon::Bulldog,
            WeaponStats {
                price: 2050,
                damage_head: (116, 100, 84),
                damage_body: (35, 30, 25),
                damage_legs: (26, 22, 18),
                fire_rate: 9.15,
                penetration: Penetration::Medium,
                magazine_size: 24,
                reload_time_ms: 2500,
            },
        );

        let simulation_id = Uuid::new_v4();

        ValorantSimulation {
            state: SimulationState {
                id: simulation_id,
                mode: SimulationMode::Playing,
                phase: SimulationPhase::NotStarted,
                playback_speed: 1.0,
                current_timestamp: 0,
                current_round: 0,
                attacker_score: 0,
                defender_score: 0,
                overtime_active: false,
                tick_count: 0,
            },
            players: HashMap::new(),
            events: Vec::new(),
            loss_streaks: HashMap::new(),
            weapon_stats,
            checkpoints: HashMap::new(),
            round_timer_ms: 100_000,
            spike_timer_ms: 45_000,
            spike_planted: false,
            spike_defused: false,
            round_start_timestamp: 0,
        }
    }

    pub fn add_player(&mut self, player: Player) {
        self.players.insert(player.id, player);
    }

    pub fn advance_time(&mut self, delta_ms: u64) {
        let adjusted_delta = (delta_ms as f32 * self.state.playback_speed) as u64;
        self.state.current_timestamp += adjusted_delta;
        self.state.tick_count += 1;
    }

    // New modular control methods
    pub fn start_simulation(&mut self) {
        if matches!(self.state.phase, SimulationPhase::NotStarted) {
            self.state.mode = SimulationMode::Playing;
            self.state.phase = SimulationPhase::BuyPhase { round_number: 1 };
            self.state.current_round = 1;

            // Initialize players with starting credits
            for player in self.players.values_mut() {
                player.current_credits = 800;
            }

            // Initialize loss streaks
            self.loss_streaks.insert(Team::Attackers, 0);
            self.loss_streaks.insert(Team::Defenders, 0);

            self.record_event(GameEvent::MatchStart {
                timestamp: self.state.current_timestamp,
            });
        }
    }

    pub fn pause_simulation(&mut self) {
        self.state.mode = SimulationMode::Paused;
    }

    pub fn resume_simulation(&mut self) {
        if matches!(self.state.mode, SimulationMode::Paused) {
            self.state.mode = SimulationMode::Playing;
        }
    }

    pub fn set_playback_speed(&mut self, speed: f32) {
        self.state.playback_speed = speed.clamp(0.1, 5.0);
        if speed > 1.0 {
            self.state.mode = SimulationMode::FastForward;
        } else {
            self.state.mode = SimulationMode::Playing;
        }
    }

    pub fn create_checkpoint(&mut self) {
        let checkpoint = SimulationCheckpoint {
            state: self.state.clone(),
            players: self.players.clone(),
            events: self.events.clone(),
            loss_streaks: self.loss_streaks.clone(),
        };
        self.checkpoints.insert(self.state.tick_count, checkpoint);
    }

    pub fn restore_checkpoint(&mut self, tick: u64) -> Result<(), String> {
        if let Some(checkpoint) = self.checkpoints.get(&tick).cloned() {
            self.state = checkpoint.state;
            self.players = checkpoint.players;
            self.events = checkpoint.events;
            self.loss_streaks = checkpoint.loss_streaks;
            Ok(())
        } else {
            Err(format!("Checkpoint not found for tick {}", tick))
        }
    }

    pub fn get_current_state(&self) -> &SimulationState {
        &self.state
    }

    pub fn get_filtered_events(&self, filter: &EventFilter) -> Vec<&GameEvent> {
        self.events
            .iter()
            .filter(|event| {
                if let Some(ref event_types) = filter.event_types {
                    let event_name = match event {
                        GameEvent::MatchStart { .. } => "MatchStart",
                        GameEvent::MatchEnd { .. } => "MatchEnd",
                        GameEvent::BuyPhaseStart { .. } => "BuyPhaseStart",
                        GameEvent::BuyPhaseEnd { .. } => "BuyPhaseEnd",
                        GameEvent::RoundStart { .. } => "RoundStart",
                        GameEvent::RoundEnd { .. } => "RoundEnd",
                        GameEvent::Kill { .. } => "Kill",
                        GameEvent::Damage { .. } => "Damage",
                        GameEvent::SpikePlant { .. } => "SpikePlant",
                        GameEvent::SpikeDefuse { .. } => "SpikeDefuse",
                        GameEvent::AbilityUsed { .. } => "AbilityUsed",
                        GameEvent::SideSwap { .. } => "SideSwap",
                    };
                    if !event_types.contains(&event_name.to_string()) {
                        return false;
                    }
                }

                if let Some(ref start_time) = filter.start_timestamp {
                    let event_time = match event {
                        GameEvent::MatchStart { timestamp } => *timestamp,
                        GameEvent::MatchEnd { timestamp, .. } => *timestamp,
                        GameEvent::BuyPhaseStart { timestamp, .. } => *timestamp,
                        GameEvent::BuyPhaseEnd { timestamp, .. } => *timestamp,
                        GameEvent::RoundStart { timestamp, .. } => *timestamp,
                        GameEvent::RoundEnd { timestamp, .. } => *timestamp,
                        GameEvent::Kill { timestamp, .. } => *timestamp,
                        GameEvent::Damage { timestamp, .. } => *timestamp,
                        GameEvent::SpikePlant { timestamp, .. } => *timestamp,
                        GameEvent::SpikeDefuse { timestamp, .. } => *timestamp,
                        GameEvent::AbilityUsed { timestamp, .. } => *timestamp,
                        GameEvent::SideSwap { timestamp, .. } => *timestamp,
                    };
                    if event_time < *start_time {
                        return false;
                    }
                }

                if let Some(ref end_time) = filter.end_timestamp {
                    let event_time = match event {
                        GameEvent::MatchStart { timestamp } => *timestamp,
                        GameEvent::MatchEnd { timestamp, .. } => *timestamp,
                        GameEvent::BuyPhaseStart { timestamp, .. } => *timestamp,
                        GameEvent::BuyPhaseEnd { timestamp, .. } => *timestamp,
                        GameEvent::RoundStart { timestamp, .. } => *timestamp,
                        GameEvent::RoundEnd { timestamp, .. } => *timestamp,
                        GameEvent::Kill { timestamp, .. } => *timestamp,
                        GameEvent::Damage { timestamp, .. } => *timestamp,
                        GameEvent::SpikePlant { timestamp, .. } => *timestamp,
                        GameEvent::SpikeDefuse { timestamp, .. } => *timestamp,
                        GameEvent::AbilityUsed { timestamp, .. } => *timestamp,
                        GameEvent::SideSwap { timestamp, .. } => *timestamp,
                    };
                    if event_time > *end_time {
                        return false;
                    }
                }

                true
            })
            .collect()
    }

    pub fn get_player_stats(&self) -> Vec<PlayerStats> {
        self.players
            .values()
            .map(|player| {
                let kills = self.events.iter().filter(|e| {
                matches!(e, GameEvent::Kill { killer_id, .. } if *killer_id == player.id)
            }).count() as u32;

                let deaths = self.events.iter().filter(|e| {
                matches!(e, GameEvent::Kill { victim_id, .. } if *victim_id == player.id)
            }).count() as u32;

                let headshot_kills = self
                    .events
                    .iter()
                    .filter(|e| {
                        matches!(e, GameEvent::Kill { killer_id, is_headshot, .. }
                    if *killer_id == player.id && *is_headshot)
                    })
                    .count() as u32;

                let damage_dealt = self
                    .events
                    .iter()
                    .filter_map(|e| match e {
                        GameEvent::Damage {
                            attacker_id,
                            amount,
                            ..
                        } if *attacker_id == player.id => Some(*amount),
                        _ => None,
                    })
                    .sum::<u32>();

                PlayerStats {
                    player_id: player.id,
                    kills,
                    deaths,
                    assists: 0, // TODO: Implement assist tracking
                    damage_dealt,
                    headshot_percentage: if kills > 0 {
                        (headshot_kills as f32 / kills as f32) * 100.0
                    } else {
                        0.0
                    },
                    credits: player.current_credits,
                    ultimate_points: player.ultimate_points,
                }
            })
            .collect()
    }

    fn record_event(&mut self, event: GameEvent) {
        self.events.push(event);
    }

    pub fn advance_tick(&mut self) -> Result<(), String> {
        if matches!(self.state.mode, SimulationMode::Paused) {
            return Ok(());
        }

        match &self.state.phase {
            SimulationPhase::NotStarted => {
                self.start_simulation();
            }
            SimulationPhase::BuyPhase { round_number } => {
                self.advance_buy_phase(*round_number)?;
            }
            SimulationPhase::RoundActive {
                round_number,
                spike_planted,
            } => {
                self.advance_round_active(*round_number, *spike_planted)?;
            }
            SimulationPhase::RoundEnd {
                round_number,
                winner,
            } => {
                self.advance_round_end(*round_number, winner.clone())?;
            }
            SimulationPhase::MatchEnd { .. } => {
                return Ok(()); // Match is over, no more ticks
            }
        }

        self.advance_time(500); // Each tick is 500ms
        Ok(())
    }

    fn advance_buy_phase(&mut self, round_number: u8) -> Result<(), String> {
        // Check if we need to start the buy phase (first time entering this phase)
        let should_start_buy_phase = self.events.is_empty()
            || !matches!(self.events.last(), Some(GameEvent::BuyPhaseStart { .. }))
            || matches!(self.events.last(), Some(GameEvent::RoundEnd { .. }));

        if should_start_buy_phase {
            self.record_event(GameEvent::BuyPhaseStart {
                timestamp: self.state.current_timestamp,
                round_number,
            });

            // Set the buy phase start timestamp
            self.round_start_timestamp = self.state.current_timestamp;

            // Reset players for round
            for player in self.players.values_mut() {
                player.reset_for_round();
            }

            // Handle side swaps
            if round_number == 13 {
                for player in self.players.values_mut() {
                    player.team = match player.team {
                        Team::Attackers => Team::Defenders,
                        Team::Defenders => Team::Attackers,
                    };
                    player.current_credits = 800;
                    player.current_loadout = PlayerLoadout {
                        primary_weapon: None,
                        secondary_weapon: Weapon::Classic,
                        armor: ArmorType::None,
                        abilities_purchased: Vec::new(),
                    };
                }
                self.loss_streaks.insert(Team::Attackers, 0);
                self.loss_streaks.insert(Team::Defenders, 0);

                self.record_event(GameEvent::SideSwap {
                    timestamp: self.state.current_timestamp,
                    round_number,
                });
            }
        }

        // Calculate elapsed time since buy phase started
        let elapsed_time = self.state.current_timestamp - self.round_start_timestamp;

        // Simulate buying logic at the start of buy phase (after 1 second to allow setup)
        if (1000..=1500).contains(&elapsed_time) {
            self.simulate_player_purchases();
        }

        // End buy phase after 30 seconds
        if elapsed_time >= 30_000 {
            self.record_event(GameEvent::BuyPhaseEnd {
                timestamp: self.state.current_timestamp,
                round_number,
            });

            self.state.phase = SimulationPhase::RoundActive {
                round_number,
                spike_planted: false,
            };
            self.spike_planted = false;
            self.spike_defused = false;
            self.round_start_timestamp = self.state.current_timestamp;
            self.round_timer_ms = 100_000;
            self.spike_timer_ms = 45_000;

            self.record_event(GameEvent::RoundStart {
                timestamp: self.state.current_timestamp,
                round_number,
                attacker_credits_start: self
                    .players
                    .values()
                    .find(|p| p.team == Team::Attackers)
                    .map_or(0, |p| p.current_credits),
                defender_credits_start: self
                    .players
                    .values()
                    .find(|p| p.team == Team::Defenders)
                    .map_or(0, |p| p.current_credits),
            });
        }

        Ok(())
    }

    fn advance_round_active(
        &mut self,
        round_number: u8,
        _spike_planted: bool,
    ) -> Result<(), String> {
        self.round_timer_ms = self.round_timer_ms.saturating_sub(500);

        let alive_attackers: Vec<u32> = self
            .get_alive_players_on_team(&Team::Attackers)
            .into_iter()
            .map(|p| p.id)
            .collect();
        let alive_defenders: Vec<u32> = self
            .get_alive_players_on_team(&Team::Defenders)
            .into_iter()
            .map(|p| p.id)
            .collect();

        // Check win conditions
        if alive_attackers.is_empty() {
            self.end_round(
                round_number,
                Team::Defenders,
                RoundEndReason::AllAttackersEliminated,
            );
            return Ok(());
        }
        if alive_defenders.is_empty() {
            if self.spike_planted && !self.spike_defused {
                self.end_round(
                    round_number,
                    Team::Attackers,
                    RoundEndReason::SpikeDetonated,
                );
            } else {
                self.end_round(
                    round_number,
                    Team::Attackers,
                    RoundEndReason::AllDefendersEliminated,
                );
            }
            return Ok(());
        }

        // Spike mechanics
        if !self.spike_planted {
            // 15% chance per tick after 30 seconds to plant spike
            if self.state.current_timestamp - self.round_start_timestamp > 30_000 {
                let mut rng = rand::rng();
                if rng.random::<f32>() < 0.15 {
                    let planter_id = alive_attackers[rng.random_range(0..alive_attackers.len())];
                    self.record_event(GameEvent::SpikePlant {
                        timestamp: self.state.current_timestamp,
                        planter_id,
                    });
                    self.award_spike_plant_bonus(planter_id);
                    self.spike_planted = true;
                    self.state.phase = SimulationPhase::RoundActive {
                        round_number,
                        spike_planted: true,
                    };
                }
            }
        } else {
            self.spike_timer_ms = self.spike_timer_ms.saturating_sub(500);
            if self.spike_timer_ms <= 0 {
                self.end_round(
                    round_number,
                    Team::Attackers,
                    RoundEndReason::SpikeDetonated,
                );
                return Ok(());
            }

            // 5% chance per tick for defuse attempt
            let mut rng = rand::rng();
            if !alive_defenders.is_empty() && rng.random::<f32>() < 0.05 {
                let defuser_id = alive_defenders[rng.random_range(0..alive_defenders.len())];
                self.record_event(GameEvent::SpikeDefuse {
                    timestamp: self.state.current_timestamp,
                    defuser_id,
                    successful: true,
                });
                if let Some(defuser) = self.players.get_mut(&defuser_id) {
                    defuser.ultimate_points += 1;
                }
                self.spike_defused = true;
                self.end_round(round_number, Team::Defenders, RoundEndReason::SpikeDefused);
                return Ok(());
            }
        }

        // Combat simulation
        if !alive_attackers.is_empty() && !alive_defenders.is_empty() {
            self.simulate_combat(&alive_attackers, &alive_defenders);
        }

        // Time expiration
        if !self.spike_planted && self.round_timer_ms <= 0 {
            self.end_round(round_number, Team::Defenders, RoundEndReason::TimeExpired);
        }

        Ok(())
    }

    fn advance_round_end(&mut self, round_number: u8, _winner: Team) -> Result<(), String> {
        // Calculate elapsed time since the round ended
        let round_end_timestamp = match self
            .events
            .iter()
            .rev()
            .find(|e| matches!(e, GameEvent::RoundEnd { .. }))
        {
            Some(GameEvent::RoundEnd { timestamp, .. }) => *timestamp,
            _ => {
                // Fallback: use current timestamp if no RoundEnd event found
                log::warn!(
                    "No RoundEnd event found for round {}, using current timestamp",
                    round_number
                );
                self.state.current_timestamp
            }
        };

        let elapsed_since_round_end = self.state.current_timestamp - round_end_timestamp;

        // Wait 2 seconds before starting next round
        if elapsed_since_round_end >= 2000 {
            if self.check_match_end_conditions() {
                return Ok(());
            }

            // Start next round
            let next_round = round_number + 1;
            self.state.current_round = next_round;
            self.state.phase = SimulationPhase::BuyPhase {
                round_number: next_round,
            };
        }
        Ok(())
    }

    fn end_round(&mut self, round_number: u8, winner: Team, reason: RoundEndReason) {
        // Award round-end credits
        self.calculate_round_rewards(&winner, &reason, self.spike_planted);

        // Update scores
        if winner == Team::Attackers {
            self.state.attacker_score += 1;
        } else {
            self.state.defender_score += 1;
        }

        self.record_event(GameEvent::RoundEnd {
            timestamp: self.state.current_timestamp,
            round_number,
            winning_team: winner.clone(),
            reason,
        });

        self.state.phase = SimulationPhase::RoundEnd {
            round_number,
            winner,
        };
    }

    fn check_match_end_conditions(&mut self) -> bool {
        const WIN_SCORE_REGULAR: u8 = 13;
        const WIN_MARGIN_OVERTIME: u8 = 2;

        let current_diff =
            (self.state.attacker_score as i16 - self.state.defender_score as i16).abs();

        if self.state.overtime_active {
            if current_diff >= WIN_MARGIN_OVERTIME as i16 {
                let winning_team = if self.state.attacker_score > self.state.defender_score {
                    Team::Attackers
                } else {
                    Team::Defenders
                };
                self.record_event(GameEvent::MatchEnd {
                    timestamp: self.state.current_timestamp,
                    winning_team: winning_team.clone(),
                    score_attackers: self.state.attacker_score,
                    score_defenders: self.state.defender_score,
                });
                self.state.phase = SimulationPhase::MatchEnd {
                    winner: winning_team,
                    final_score: (self.state.attacker_score, self.state.defender_score),
                };
                return true;
            }
        } else if self.state.attacker_score >= WIN_SCORE_REGULAR
            || self.state.defender_score >= WIN_SCORE_REGULAR
        {
            if current_diff >= 2 {
                let winning_team = if self.state.attacker_score > self.state.defender_score {
                    Team::Attackers
                } else {
                    Team::Defenders
                };
                self.record_event(GameEvent::MatchEnd {
                    timestamp: self.state.current_timestamp,
                    winning_team: winning_team.clone(),
                    score_attackers: self.state.attacker_score,
                    score_defenders: self.state.defender_score,
                });
                self.state.phase = SimulationPhase::MatchEnd {
                    winner: winning_team,
                    final_score: (self.state.attacker_score, self.state.defender_score),
                };
                return true;
            } else if self.state.attacker_score == 12 && self.state.defender_score == 12 {
                self.state.overtime_active = true;
            }
        }

        false
    }

    pub fn get_alive_players_on_team(&self, team: &Team) -> Vec<&Player> {
        self.players
            .values()
            .filter(|p| p.team == *team && p.is_alive)
            .collect()
    }

    pub fn determine_round_type(&self, team: &Team) -> RoundType {
        let team_credits: u32 = self.players
            .values()
            .filter(|p| p.team == *team)
            .map(|p| p.current_credits)
            .sum();
        
        let avg_credits = team_credits / 5; // 5 players per team
        
        if self.state.current_round == 1 || self.state.current_round == 13 {
            RoundType::Pistol
        } else if avg_credits < 2000 {
            RoundType::Eco
        } else if avg_credits > 4500 {
            RoundType::FullBuy
        } else if avg_credits < 3000 {
            let loss_streak = self.loss_streaks.get(team).unwrap_or(&0);
            if *loss_streak >= 2 {
                RoundType::ForceBuy
            } else {
                RoundType::Eco
            }
        } else {
            RoundType::AntiEco
        }
    }

    pub fn predict_enemy_economy(&self, team: &Team) -> EconomyState {
        let enemy_team = match team {
            Team::Attackers => Team::Defenders,
            Team::Defenders => Team::Attackers,
        };
        
        let enemy_credits: u32 = self.players
            .values()
            .filter(|p| p.team == enemy_team)
            .map(|p| p.current_credits)
            .sum();
        
        let avg_enemy_credits = enemy_credits / 5;
        
        if avg_enemy_credits < 2000 {
            EconomyState::Poor
        } else if avg_enemy_credits > 4000 {
            EconomyState::Strong
        } else {
            EconomyState::Moderate
        }
    }

    pub fn create_round_context(&self, team: &Team) -> RoundContext {
        let team_credits: u32 = self.players
            .values()
            .filter(|p| p.team == *team)
            .map(|p| p.current_credits)
            .sum();

        let previous_round_result = self.events
            .iter()
            .rev()
            .find_map(|event| match event {
                GameEvent::RoundEnd { reason, .. } => Some(reason.clone()),
                _ => None,
            });

        RoundContext {
            round_type: self.determine_round_type(team),
            team_economy: team_credits,
            enemy_predicted_economy: self.predict_enemy_economy(team),
            previous_round_result,
            loss_streak: *self.loss_streaks.get(team).unwrap_or(&0),
        }
    }

    pub fn make_dynamic_buy_decision(&self, player: &Player, context: &RoundContext) -> BuyDecision {
        let mut best_weapon: Option<Weapon> = None;
        let mut best_armor = ArmorType::None;
        let mut remaining_credits = player.current_credits;
        let mut confidence = 0.5;
        let mut abilities_budget = 0u32;
        let mut coordination_priority: f32 = 0.5;

        // Use adaptive preferences if ML is enabled, otherwise use base preferences
        let (preferred_weapons, eco_threshold, force_buy_tendency, armor_priority, utility_priority) = 
            if let Some(ref learning_profile) = player.learning_profile {
                let adaptive = &learning_profile.adaptive_preferences;
                (
                    &adaptive.base_preferences.preferred_weapons,
                    adaptive.base_preferences.eco_threshold,
                    adaptive.base_preferences.force_buy_tendency,
                    adaptive.base_preferences.armor_priority,
                    adaptive.base_preferences.utility_priority,
                )
            } else {
                (
                    &player.buy_preferences.preferred_weapons,
                    player.buy_preferences.eco_threshold,
                    player.buy_preferences.force_buy_tendency,
                    player.buy_preferences.armor_priority,
                    player.buy_preferences.utility_priority,
                )
            };

        // Check if player should eco based on their preferences and context
        let should_eco = remaining_credits < eco_threshold
            && context.round_type != RoundType::ForceBuy
            && rand::random::<f32>() > force_buy_tendency;

        if should_eco && context.round_type != RoundType::Pistol {
            // Eco round - only buy cheap utility or save
            if remaining_credits >= 800 && rand::random::<f32>() < 0.3 {
                return BuyDecision {
                    primary_weapon: None,
                    secondary_weapon: Weapon::Sheriff,
                    armor: ArmorType::None,
                    abilities_budget: 0,
                    total_cost: 800,
                    confidence: 0.8,
                    coordination_priority: 0.3,
                };
            }
            return BuyDecision {
                primary_weapon: None,
                secondary_weapon: Weapon::Classic,
                armor: ArmorType::None,
                abilities_budget: 0,
                total_cost: 0,
                confidence: 0.9,
                coordination_priority: 0.2,
            };
        }

        // Calculate coordination priority based on role and team economy
        let role = player.agent.get_role();
        coordination_priority = match role {
            AgentRole::Controller => 0.9,  // High priority for smokers
            AgentRole::Initiator => 0.8,   // High priority for info gatherers
            AgentRole::Duelist => 0.6,     // Medium priority for entry fraggers
            AgentRole::Sentinel => 0.7,    // Medium-high priority for site anchors
        };

        // Adjust coordination priority based on context
        if context.round_type == RoundType::ForceBuy {
            coordination_priority *= 1.2; // Increase importance in force buy situations
        }
        if context.loss_streak >= 3 {
            coordination_priority *= 1.1; // Slightly more important when desperate
        }

        // Sort weapons by priority considering situational modifiers
        let mut sorted_weapons = preferred_weapons.clone();
        sorted_weapons.sort_by(|a, b| {
            let priority_a = a.priority + a.situational_modifiers
                .get(&format!("{:?}", context.round_type))
                .unwrap_or(&0.0);
            let priority_b = b.priority + b.situational_modifiers
                .get(&format!("{:?}", context.round_type))
                .unwrap_or(&0.0);
            priority_b.partial_cmp(&priority_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Calculate utility budget based on role and preferences
        let base_utility_budget = (remaining_credits as f32 * utility_priority * 0.3) as u32;
        abilities_budget = match role {
            AgentRole::Controller => base_utility_budget.max(800).min(1500), // Controllers need smokes
            AgentRole::Initiator => base_utility_budget.max(600).min(1200),  // Initiators need info abilities
            AgentRole::Sentinel => base_utility_budget.max(400).min(800),    // Sentinels need setup abilities
            AgentRole::Duelist => base_utility_budget.min(400),              // Duelists focus on fragging
        };

        // Reserve utility budget
        remaining_credits = remaining_credits.saturating_sub(abilities_budget);

        // Try to buy the highest priority weapon that fits budget
        for weapon_pref in &sorted_weapons {
            let weapon_cost = self.weapon_stats[&weapon_pref.weapon].price;
            let armor_cost = if armor_priority > 0.7 && remaining_credits >= weapon_cost + 1000 {
                1000 // Heavy armor
            } else if armor_priority > 0.4 && remaining_credits >= weapon_cost + 400 {
                400 // Light armor
            } else {
                0 // No armor
            };

            if remaining_credits >= weapon_cost + armor_cost && remaining_credits >= weapon_pref.min_credits {
                // Check if it's a primary or secondary weapon
                if matches!(weapon_pref.weapon, Weapon::Classic | Weapon::Shorty | Weapon::Frenzy | Weapon::Ghost | Weapon::Sheriff) {
                    // Secondary weapon
                    continue; // Handle secondaries separately
                } else {
                    // Primary weapon
                    best_weapon = Some(weapon_pref.weapon.clone());
                    remaining_credits -= weapon_cost;
                    confidence = weapon_pref.priority;
                    break;
                }
            }
        }

        // Determine armor based on remaining credits and preferences
        if armor_priority > 0.7 && remaining_credits >= 1000 {
            best_armor = ArmorType::Heavy;
            remaining_credits -= 1000;
        } else if armor_priority > 0.4 && remaining_credits >= 400 {
            best_armor = ArmorType::Light;
            remaining_credits -= 400;
        }

        // Choose secondary weapon
        let secondary_weapon = if remaining_credits >= 800 && 
            sorted_weapons.iter().any(|w| w.weapon == Weapon::Sheriff && w.priority > 0.6) {
            Weapon::Sheriff
        } else if remaining_credits >= 500 {
            Weapon::Ghost
        } else {
            Weapon::Classic
        };

        let total_cost = player.current_credits - remaining_credits;

        // Phase 3: ML-Enhanced Confidence Scoring
        let final_confidence = self.calculate_ml_confidence(
            player,
            &best_weapon,
            context,
            confidence,
        );

        BuyDecision {
            primary_weapon: best_weapon,
            secondary_weapon,
            armor: best_armor,
            abilities_budget,
            total_cost,
            confidence: final_confidence.clamp(0.1, 1.0),
            coordination_priority: coordination_priority.clamp(0.1, 1.0),
        }
    }


    pub fn create_team_buy_strategy(&self, team: &Team, context: &RoundContext) -> TeamBuyStrategy {
        let team_players: Vec<&Player> = self.players.values()
            .filter(|p| p.team == *team)
            .collect();

        let team_credits: u32 = team_players.iter().map(|p| p.current_credits).sum();
        let _avg_credits = if !team_players.is_empty() { 
            team_credits / team_players.len() as u32 
        } else { 
            0 
        };

        // Determine strategy type based on economy and context
        let strategy_type = match context.round_type {
            RoundType::Pistol => TeamStrategyType::HalfBuy,
            RoundType::Eco => {
                if context.loss_streak >= 3 {
                    TeamStrategyType::EcoFrag // Desperate eco with some buys
                } else {
                    TeamStrategyType::FullSave
                }
            },
            RoundType::ForceBuy => TeamStrategyType::ForceBuy,
            RoundType::FullBuy => TeamStrategyType::FullBuy,
            RoundType::AntiEco => TeamStrategyType::FullBuy,
        };

        // Determine priority roles based on strategy
        let priority_roles = match strategy_type {
            TeamStrategyType::FullSave => vec![], // No priorities when saving
            TeamStrategyType::EcoFrag => vec![AgentRole::Duelist], // Entry fraggers get priority
            TeamStrategyType::HalfBuy => vec![AgentRole::Controller, AgentRole::Initiator], // Utility roles first
            TeamStrategyType::FullBuy => vec![AgentRole::Controller, AgentRole::Initiator, AgentRole::Duelist, AgentRole::Sentinel],
            TeamStrategyType::ForceBuy => vec![AgentRole::Controller, AgentRole::Duelist], // Essential roles only
        };

        // Calculate utility budget based on team economy and strategy
        let utility_budget = match strategy_type {
            TeamStrategyType::FullSave => 0,
            TeamStrategyType::EcoFrag => (team_credits as f32 * 0.1) as u32,
            TeamStrategyType::HalfBuy => (team_credits as f32 * 0.15) as u32,
            TeamStrategyType::FullBuy => (team_credits as f32 * 0.25) as u32,
            TeamStrategyType::ForceBuy => (team_credits as f32 * 0.2) as u32,
        };

        // Determine minimum rifles needed
        let minimum_rifles = match strategy_type {
            TeamStrategyType::FullSave => 0,
            TeamStrategyType::EcoFrag => 0,
            TeamStrategyType::HalfBuy => 1,
            TeamStrategyType::FullBuy => 4,
            TeamStrategyType::ForceBuy => 2,
        };

        let allow_eco_frags = matches!(strategy_type, TeamStrategyType::EcoFrag | TeamStrategyType::ForceBuy);
        
        TeamBuyStrategy {
            strategy_type,
            priority_roles,
            utility_budget,
            minimum_rifles,
            allow_eco_frags,
        }
    }

    pub fn create_utility_budget(&self, team_strategy: &TeamBuyStrategy, team: &Team) -> UtilityBudget {
        let total_budget = team_strategy.utility_budget;
        
        // Count players by role
        let controllers = self.players.values()
            .filter(|p| p.team == *team && p.agent.get_role() == AgentRole::Controller)
            .count() as u32;
        let initiators = self.players.values()
            .filter(|p| p.team == *team && p.agent.get_role() == AgentRole::Initiator)
            .count() as u32;
        let sentinels = self.players.values()
            .filter(|p| p.team == *team && p.agent.get_role() == AgentRole::Sentinel)
            .count() as u32;

        // Allocate budget based on role priorities
        let smokes_budget = if controllers > 0 {
            (total_budget as f32 * 0.4) as u32 // Controllers get 40% for smokes
        } else {
            0
        };

        let flashes_budget = if initiators > 0 {
            (total_budget as f32 * 0.3) as u32 // Initiators get 30% for flashes/darts
        } else {
            0
        };

        let info_budget = if initiators > 0 {
            (total_budget as f32 * 0.2) as u32 // Initiators get 20% for info gathering
        } else {
            0
        };

        let healing_budget = if sentinels > 0 {
            (total_budget as f32 * 0.1) as u32 // Sentinels get 10% for healing/support
        } else {
            0
        };

        UtilityBudget {
            smokes_budget,
            flashes_budget,
            info_budget,
            healing_budget,
            total_utility_spend: smokes_budget + flashes_budget + info_budget + healing_budget,
        }
    }

    pub fn create_team_composition(&self, team: &Team) -> TeamComposition {
        let team_players: Vec<&Player> = self.players.values()
            .filter(|p| p.team == *team)
            .collect();

        let has_smoker = team_players.iter()
            .any(|p| matches!(p.agent, Agent::Omen | Agent::Brimstone | Agent::Viper | Agent::Astra | Agent::Harbor | Agent::Clove));
        
        let has_igl = team_players.iter()
            .any(|p| matches!(p.agent.get_role(), AgentRole::Controller | AgentRole::Sentinel));
        
        let has_entry_fragger = team_players.iter()
            .any(|p| p.agent.get_role() == AgentRole::Duelist);
        
        let has_support = team_players.iter()
            .any(|p| matches!(p.agent.get_role(), AgentRole::Initiator | AgentRole::Sentinel));

        // Count potential rifle and operator players based on current weapons
        let rifle_players = team_players.iter()
            .filter(|p| matches!(p.current_loadout.primary_weapon, 
                Some(Weapon::Vandal) | Some(Weapon::Phantom) | Some(Weapon::Bulldog) | Some(Weapon::Guardian)))
            .count() as u8;

        let operator_players = team_players.iter()
            .filter(|p| matches!(p.current_loadout.primary_weapon, Some(Weapon::Operator)))
            .count() as u8;

        TeamComposition {
            has_smoker,
            has_igl,
            has_entry_fragger,
            has_support,
            rifle_players,
            operator_players,
        }
    }

    pub fn make_coordinated_buy_decision(&self, player: &Player, context: &RoundContext, team_strategy: &TeamBuyStrategy, utility_budget: &UtilityBudget) -> BuyDecision {
        let mut individual_decision = self.make_dynamic_buy_decision(player, context);
        
        let role = player.agent.get_role();
        let is_priority_role = team_strategy.priority_roles.contains(&role);
        
        // Adjust decision based on team coordination
        if !is_priority_role && team_strategy.strategy_type == TeamStrategyType::EcoFrag {
            // Non-priority players should save more aggressively in eco-frag rounds
            if individual_decision.total_cost > 1000 {
                return BuyDecision {
                    primary_weapon: None,
                    secondary_weapon: Weapon::Classic,
                    armor: ArmorType::None,
                    abilities_budget: 0,
                    total_cost: 0,
                    confidence: 0.8,
                    coordination_priority: 0.2,
                };
            }
        }

        // Adjust utility budget based on team allocation
        match role {
            AgentRole::Controller => {
                individual_decision.abilities_budget = (utility_budget.smokes_budget as f32 * 0.8) as u32;
            },
            AgentRole::Initiator => {
                individual_decision.abilities_budget = ((utility_budget.flashes_budget + utility_budget.info_budget) as f32 * 0.6) as u32;
            },
            AgentRole::Sentinel => {
                individual_decision.abilities_budget = (utility_budget.healing_budget as f32 * 0.5) as u32;
            },
            AgentRole::Duelist => {
                // Duelists get minimal utility budget
                individual_decision.abilities_budget = individual_decision.abilities_budget.min(200);
            },
        }

        // Ensure minimum rifles are met
        if team_strategy.minimum_rifles > 0 && individual_decision.primary_weapon.is_none() {
            if is_priority_role && player.current_credits >= 2900 {
                // Force buy rifle for priority players if team needs minimum rifles
                individual_decision.primary_weapon = Some(Weapon::Vandal);
                individual_decision.total_cost = individual_decision.total_cost.max(2900);
            }
        }

        // Recalculate total cost including utility
        individual_decision.total_cost = individual_decision.total_cost.saturating_add(individual_decision.abilities_budget);
        
        // Ensure player can afford the decision
        if individual_decision.total_cost > player.current_credits {
            let overspend = individual_decision.total_cost - player.current_credits;
            individual_decision.abilities_budget = individual_decision.abilities_budget.saturating_sub(overspend);
            individual_decision.total_cost = player.current_credits;
        }

        individual_decision
    }

    pub fn simulate_player_purchases(&mut self) {
        // Create round contexts for both teams
        let attacker_context = self.create_round_context(&Team::Attackers);
        let defender_context = self.create_round_context(&Team::Defenders);

        // Create team strategies
        let attacker_strategy = self.create_team_buy_strategy(&Team::Attackers, &attacker_context);
        let defender_strategy = self.create_team_buy_strategy(&Team::Defenders, &defender_context);

        // Create utility budgets
        let attacker_utility = self.create_utility_budget(&attacker_strategy, &Team::Attackers);
        let defender_utility = self.create_utility_budget(&defender_strategy, &Team::Defenders);

        // Collect all buy decisions first to avoid borrowing conflicts
        let mut buy_decisions: HashMap<u32, BuyDecision> = HashMap::new();
        
        for player in self.players.values() {
            let (context, strategy, utility) = if player.team == Team::Attackers { 
                (&attacker_context, &attacker_strategy, &attacker_utility)
            } else { 
                (&defender_context, &defender_strategy, &defender_utility)
            };
            
            let decision = self.make_coordinated_buy_decision(player, context, strategy, utility);
            buy_decisions.insert(player.id, decision);
        }

        // Apply buy decisions to players
        for player in self.players.values_mut() {
            if let Some(decision) = buy_decisions.get(&player.id) {
                // Reset loadout if they died (don't carry over equipment)
                if !player.survived_round() {
                    player.current_loadout = PlayerLoadout {
                        primary_weapon: None,
                        secondary_weapon: Weapon::Classic,
                        armor: ArmorType::None,
                        abilities_purchased: Vec::new(),
                    };
                }

                // Apply the buy decision
                player.current_loadout.primary_weapon = decision.primary_weapon.clone();
                player.current_loadout.secondary_weapon = decision.secondary_weapon.clone();
                player.current_loadout.armor = decision.armor.clone();
                
                // Add utility purchases based on abilities budget
                if decision.abilities_budget > 0 {
                    let role = player.agent.get_role();
                    match role {
                        AgentRole::Controller => {
                            player.current_loadout.abilities_purchased.push("Smoke".to_string());
                            if decision.abilities_budget >= 600 {
                                player.current_loadout.abilities_purchased.push("Extra Smoke".to_string());
                            }
                        },
                        AgentRole::Initiator => {
                            player.current_loadout.abilities_purchased.push("Flash".to_string());
                            if decision.abilities_budget >= 500 {
                                player.current_loadout.abilities_purchased.push("Info Dart".to_string());
                            }
                        },
                        AgentRole::Sentinel => {
                            player.current_loadout.abilities_purchased.push("Utility".to_string());
                        },
                        AgentRole::Duelist => {
                            if decision.abilities_budget >= 200 {
                                player.current_loadout.abilities_purchased.push("Mobility".to_string());
                            }
                        },
                    }
                }
                
                player.current_credits = player.current_credits.saturating_sub(decision.total_cost);
            }
        }
    }

    fn simulate_combat(&mut self, alive_attackers: &[u32], alive_defenders: &[u32]) {
        // Safety check: ensure both teams have alive players
        if alive_attackers.is_empty() || alive_defenders.is_empty() {
            return;
        }

        let mut rng = rand::rng();

        let attacker_id = alive_attackers[rng.random_range(0..alive_attackers.len())];
        let defender_id = alive_defenders[rng.random_range(0..alive_defenders.len())];

        // Double-check both players are still alive
        let attacker_still_alive = self.players.get(&attacker_id).is_some_and(|p| p.is_alive);
        let defender_still_alive = self.players.get(&defender_id).is_some_and(|p| p.is_alive);

        if !attacker_still_alive || !defender_still_alive {
            return; // Skip combat if either player is dead
        }

        let attacker_player_data = self.players.get(&attacker_id).unwrap().clone();
        let defender_player_data = self.players.get(&defender_id).unwrap().clone();

        // Use equipped weapon for combat effectiveness
        let attacker_weapon = attacker_player_data
            .current_loadout
            .primary_weapon
            .unwrap_or(
                attacker_player_data
                    .current_loadout
                    .secondary_weapon
                    .clone(),
            );
        let defender_weapon = defender_player_data
            .current_loadout
            .primary_weapon
            .unwrap_or(
                defender_player_data
                    .current_loadout
                    .secondary_weapon
                    .clone(),
            );

        // Calculate weapon effectiveness multipliers
        let attacker_weapon_effectiveness = self.calculate_weapon_effectiveness(&attacker_weapon);
        let defender_weapon_effectiveness = self.calculate_weapon_effectiveness(&defender_weapon);

        // Enhanced combat calculation with weapon stats
        let attacker_base_skill =
            attacker_player_data.skills.aim * 0.7 + attacker_player_data.skills.hs * 0.3;
        let defender_base_skill =
            defender_player_data.skills.aim * 0.7 + defender_player_data.skills.hs * 0.3;

        let attacker_effective_skill = attacker_base_skill * attacker_weapon_effectiveness;
        let defender_effective_skill = defender_base_skill * defender_weapon_effectiveness;

        // Fire rate advantage
        let attacker_fire_rate = self.weapon_stats[&attacker_weapon].fire_rate;
        let defender_fire_rate = self.weapon_stats[&defender_weapon].fire_rate;

        let fire_rate_advantage = (attacker_fire_rate / defender_fire_rate).clamp(0.5, 2.0);

        let mut attacker_win_chance =
            0.5 + (attacker_effective_skill - defender_effective_skill) * 0.3;
        attacker_win_chance *= fire_rate_advantage;
        attacker_win_chance = attacker_win_chance.clamp(0.1f32, 0.9f32);

        // Determine hit location and headshot
        let is_attacker_headshot = rng.random::<f32>() < attacker_player_data.skills.hs;
        let is_defender_headshot = rng.random::<f32>() < defender_player_data.skills.hs;

        let hit_body_part = if is_attacker_headshot || is_defender_headshot {
            BodyPart::Head
        } else if rng.random::<f32>() < 0.7 {
            BodyPart::Body
        } else {
            BodyPart::Legs
        };

        // Simulate engagement range (10-50 meters)
        let engagement_range = rng.random_range(10.0..50.0);

        if rng.random::<f32>() < attacker_win_chance {
            // Attacker wins
            let damage = self.calculate_weapon_damage(
                &attacker_weapon,
                &defender_player_data.current_loadout.armor,
                hit_body_part,
                engagement_range,
            );

            if let Some(victim) = self.players.get_mut(&defender_id) {
                victim.take_damage(damage);
            }

            // Only record kill if both killer is alive and victim actually died
            if let (Some(killer), Some(victim)) = (
                self.players.get(&attacker_id),
                self.players.get(&defender_id),
            ) {
                if killer.is_alive && !victim.is_alive {
                    self.record_event(GameEvent::Kill {
                        timestamp: self.state.current_timestamp,
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
                engagement_range,
            );

            if let Some(victim) = self.players.get_mut(&attacker_id) {
                victim.take_damage(damage);
            }

            // Only record kill if both killer is alive and victim actually died
            if let (Some(killer), Some(victim)) = (
                self.players.get(&defender_id),
                self.players.get(&attacker_id),
            ) {
                if killer.is_alive && !victim.is_alive {
                    self.record_event(GameEvent::Kill {
                        timestamp: self.state.current_timestamp,
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

    fn calculate_round_rewards(
        &mut self,
        winning_team: &Team,
        _reason: &RoundEndReason,
        spike_planted: bool,
    ) {
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
                    0 => 1900, // First loss
                    1 => 2400, // Second consecutive loss
                    _ => 2900, // Third+ consecutive loss
                };

                // Update loss streak
                self.loss_streaks
                    .insert(player.team.clone(), loss_streak + 1);

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

    fn calculate_weapon_damage(
        &self,
        weapon: &Weapon,
        armor_type: &ArmorType,
        body_part: BodyPart,
        range_meters: f32,
    ) -> u32 {
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
                if range_meters <= 15.0 {
                    1.0
                } else if range_meters <= 30.0 {
                    0.85
                } else {
                    0.7
                }
            }
            Weapon::Spectre | Weapon::Stinger => {
                if range_meters <= 20.0 {
                    1.0
                } else {
                    0.75
                }
            }
            _ => 1.0, // No damage falloff for most weapons
        };

        (base_damage as f32 * damage_multiplier) as u32
    }

    fn calculate_weapon_effectiveness(&self, weapon: &Weapon) -> f32 {
        match weapon {
            Weapon::Operator => 1.5,  // Massive aim advantage
            Weapon::Vandal => 1.2,    // High damage, good accuracy
            Weapon::Phantom => 1.15,  // Good balance
            Weapon::Guardian => 1.1,  // High damage, slower
            Weapon::Bulldog => 1.0,   // Decent rifle alternative
            Weapon::Spectre => 0.9,   // Good for close range
            Weapon::Sheriff => 0.8,   // High damage pistol
            Weapon::Ghost => 0.6,     // Balanced pistol
            Weapon::Classic => 0.4,   // Basic weapon
            _ => 0.7,                 // Default effectiveness
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

    #[allow(dead_code)]
    fn simulate_buy_phase(&mut self) {
        self.record_event(GameEvent::BuyPhaseStart {
            timestamp: self.state.current_timestamp,
            round_number: self.state.current_round,
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
            if player.current_credits >= 5700 {
                // Operator + Heavy armor
                player.current_loadout.primary_weapon = Some(Weapon::Operator);
                player.current_loadout.armor = ArmorType::Heavy;
                player.current_credits -= 5700;
            } else if player.current_credits >= 3900 {
                // Vandal + Heavy armor
                player.current_loadout.primary_weapon = Some(Weapon::Vandal);
                player.current_loadout.armor = ArmorType::Heavy;
                player.current_credits -= 3900;
            } else if player.current_credits >= 1600 {
                // SMG buy
                player.current_loadout.primary_weapon = Some(Weapon::Spectre);
                player.current_credits -= 1600;
                if player.current_credits >= 400 {
                    player.current_loadout.armor = ArmorType::Light;
                    player.current_credits -= 400;
                }
            } else if player.current_credits >= 800 {
                // Pistol upgrade
                player.current_loadout.secondary_weapon = Weapon::Sheriff;
                player.current_credits -= 800;
            }
        }

        self.advance_time(30000); // 30 second buy phase

        self.record_event(GameEvent::BuyPhaseEnd {
            timestamp: self.state.current_timestamp,
            round_number: self.state.current_round,
        });
    }

    // Convenience method for running entire simulation at once (legacy mode)
    pub fn run_simulation_to_completion(&mut self) -> Result<(), String> {
        self.start_simulation();

        let mut tick_count = 0;
        const MAX_TICKS_PER_MATCH: u64 = 50000; // Prevent infinite loops (about 4 hours at 500ms per tick)

        while !matches!(self.state.phase, SimulationPhase::MatchEnd { .. }) {
            tick_count += 1;
            if tick_count > MAX_TICKS_PER_MATCH {
                return Err(format!(
                    "Match simulation exceeded maximum tick limit ({}). Possible infinite loop detected.",
                    MAX_TICKS_PER_MATCH
                ));
            }

            self.advance_tick()?;
        }

        Ok(())
    }

    // High-level control methods for frontend
    pub fn advance_round(&mut self) -> Result<(), String> {
        let mut tick_count = 0;
        const MAX_TICKS_PER_ROUND: u64 = 2000; // Prevent infinite loops (10 minutes at 500ms per tick)

        loop {
            tick_count += 1;
            if tick_count > MAX_TICKS_PER_ROUND {
                return Err(format!(
                    "Round advancement exceeded maximum tick limit ({}). Possible infinite loop detected.",
                    MAX_TICKS_PER_ROUND
                ));
            }

            self.advance_tick()?;
            if matches!(
                self.state.phase,
                SimulationPhase::RoundEnd { .. } | SimulationPhase::MatchEnd { .. }
            ) {
                break;
            }
        }
        Ok(())
    }

    pub fn advance_multiple_ticks(&mut self, count: u32) -> Result<(), String> {
        for _ in 0..count {
            self.advance_tick()?;
            if matches!(self.state.phase, SimulationPhase::MatchEnd { .. }) {
                break;
            }
        }
        Ok(())
    }

    // Phase 3: Machine Learning Integration Methods

    pub fn enable_ml_for_player(&mut self, player_id: u32) -> Result<(), String> {
        if let Some(player) = self.players.get_mut(&player_id) {
            player.enable_machine_learning(self.state.current_timestamp);
            Ok(())
        } else {
            Err(format!("Player with ID {} not found", player_id))
        }
    }

    pub fn enable_ml_for_all_players(&mut self) {
        for player in self.players.values_mut() {
            player.enable_machine_learning(self.state.current_timestamp);
        }
    }

    pub fn enable_neural_learning_for_player(&mut self, player_id: u32) -> Result<(), String> {
        if let Some(player) = self.players.get_mut(&player_id) {
            player.enable_neural_learning()
        } else {
            Err(format!("Player with ID {} not found", player_id))
        }
    }

    pub fn enable_neural_learning_for_all_players(&mut self) -> Vec<String> {
        let mut errors = Vec::new();
        
        for player in self.players.values_mut() {
            if let Err(e) = player.enable_neural_learning() {
                errors.push(format!("Player {}: {}", player.id, e));
            }
        }
        
        errors
    }


    pub fn get_players(&self) -> &HashMap<u32, Player> {
        &self.players
    }

    pub fn get_players_mut(&mut self) -> &mut HashMap<u32, Player> {
        &mut self.players
    }

    pub fn record_round_performance(&mut self, round_number: u8) {
        let current_timestamp = self.state.current_timestamp;
        
        // Find the last buy decisions for this round
        let mut buy_decisions: HashMap<u32, (Uuid, BuyDecision, RoundContext)> = HashMap::new();
        
        // Create contexts for performance evaluation
        for player in self.players.values() {
            let context = if player.team == Team::Attackers {
                self.create_round_context(&Team::Attackers)
            } else {
                self.create_round_context(&Team::Defenders)
            };
            
            // Generate a decision ID and simulate the buy decision for tracking
            let decision_id = Uuid::new_v4();
            let decision = self.make_dynamic_buy_decision(player, &context);
            
            buy_decisions.insert(player.id, (decision_id, decision, context));
        }

        // Calculate performance outcomes first to avoid borrowing conflicts
        let mut performance_outcomes: HashMap<u32, Option<RoundPerformanceOutcome>> = HashMap::new();
        for player_id in buy_decisions.keys() {
            let performance_outcome = self.calculate_round_performance_outcome(*player_id, round_number);
            performance_outcomes.insert(*player_id, performance_outcome);
        }

        // Prepare neural network training data first (before mutable borrows)
        let mut neural_training_data: HashMap<u32, (GameStateFeatures, BuyDecisionTarget)> = HashMap::new();
        for (player_id, (_, buy_decision, round_context)) in &buy_decisions {
            if let Some(player) = self.players.get(player_id) {
                if player.neural_predictor.is_some() {
                    let features = self.extract_game_features(player, round_context);
                    if let Some(actual_outcome) = performance_outcomes.get(player_id).unwrap() {
                        let target = self.create_neural_target_from_outcome(buy_decision, actual_outcome);
                        neural_training_data.insert(*player_id, (features, target));
                    }
                }
            }
        }

        // Apply performance tracking to each player
        for (player_id, (decision_id, buy_decision, round_context)) in buy_decisions {
            if let Some(player) = self.players.get_mut(&player_id) {
                let performance_outcome = performance_outcomes.get(&player_id).unwrap().clone();
                
                player.record_buy_performance(
                    decision_id,
                    buy_decision,
                    round_context,
                    performance_outcome,
                    current_timestamp,
                );
                
                // Update success rates and adapt preferences
                player.update_weapon_success_rates();
                player.adapt_buy_preferences();
                
                // Add training example to neural network if available
                if let Some((features, target)) = neural_training_data.remove(&player_id) {
                    if let Some(ref mut predictor) = player.neural_predictor {
                        predictor.add_training_example(features, target);
                    }
                }
            }
        }
    }

    fn calculate_round_performance_outcome(&self, player_id: u32, round_number: u8) -> Option<RoundPerformanceOutcome> {
        let player = self.players.get(&player_id)?;
        
        // Count player's round events
        let round_events: Vec<&GameEvent> = self.events
            .iter()
            .filter(|event| {
                match event {
                    GameEvent::Kill { .. } => true,
                    GameEvent::RoundEnd { round_number: r, .. } => *r == round_number,
                    _ => false,
                }
            })
            .collect();

        let mut kills = 0;
        let mut deaths = 0;
        let mut survived = true;
        let mut round_won = false;

        for event in round_events {
            match event {
                GameEvent::Kill { killer_id, victim_id, .. } => {
                    if *killer_id == player_id {
                        kills += 1;
                    }
                    if *victim_id == player_id {
                        deaths += 1;
                        survived = false;
                    }
                }
                GameEvent::RoundEnd { winning_team, round_number: r, .. } if *r == round_number => {
                    round_won = *winning_team == player.team;
                }
                _ => {}
            }
        }

        // Calculate impact score based on performance
        let mut impact_score = 0.0;
        
        // Kill contribution (0-0.4)
        impact_score += (kills as f32 * 0.1).min(0.4);
        
        // Survival bonus (0-0.2)
        if survived {
            impact_score += 0.2;
        }
        
        // Round win bonus (0-0.3)
        if round_won {
            impact_score += 0.3;
        }
        
        // Death penalty (reduce by 0.1 per death)
        impact_score -= deaths as f32 * 0.1;
        
        // Clamp between 0 and 1
        impact_score = impact_score.max(0.0).min(1.0);

        Some(RoundPerformanceOutcome {
            round_won,
            kills,
            deaths,
            assists: 0, // TODO: Track assists from events
            damage_dealt: kills as u32 * 140, // Approximate damage based on kills
            damage_taken: if survived { 0 } else { 100 }, // Approximate damage taken
            clutch_situation: false, // TODO: Detect clutch situations
            first_kill: false, // TODO: Track first kills
            multi_kill: kills > 1,
            round_impact_score: impact_score,
        })
    }

    pub fn get_player_learning_insights(&self, player_id: u32) -> Option<PlayerLearningInsights> {
        let player = self.players.get(&player_id)?;
        let learning_profile = player.learning_profile.as_ref()?;
        
        // Calculate learning statistics
        let total_rounds = learning_profile.performance_history.len();
        let recent_rounds = learning_profile.performance_history
            .iter()
            .rev()
            .take(10)
            .collect::<Vec<_>>();
        
        let recent_avg_impact = if recent_rounds.is_empty() {
            0.0
        } else {
            recent_rounds
                .iter()
                .filter_map(|m| m.performance_outcome.as_ref())
                .map(|o| o.round_impact_score)
                .sum::<f32>() / recent_rounds.len() as f32
        };
        
        // Find most successful weapon
        let best_weapon = learning_profile.adaptive_preferences.success_rates
            .values()
            .max_by(|a, b| {
                let a_ratio = a.successful_rounds as f32 / a.total_rounds as f32;
                let b_ratio = b.successful_rounds as f32 / b.total_rounds as f32;
                a_ratio.partial_cmp(&b_ratio).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|rate| rate.weapon.clone());
        
        // Calculate learning trend (improving/declining/stable)
        let learning_trend = if total_rounds >= 20 {
            let first_half_avg: f32 = learning_profile.performance_history
                .iter()
                .take(total_rounds / 2)
                .filter_map(|m| m.performance_outcome.as_ref())
                .map(|o| o.round_impact_score)
                .sum::<f32>() / (total_rounds / 2) as f32;
            
            let second_half_avg: f32 = learning_profile.performance_history
                .iter()
                .skip(total_rounds / 2)
                .filter_map(|m| m.performance_outcome.as_ref())
                .map(|o| o.round_impact_score)
                .sum::<f32>() / (total_rounds / 2) as f32;
            
            let improvement = second_half_avg - first_half_avg;
            if improvement > 0.1 {
                "Improving".to_string()
            } else if improvement < -0.1 {
                "Declining".to_string()
            } else {
                "Stable".to_string()
            }
        } else {
            "Learning".to_string()
        };

        Some(PlayerLearningInsights {
            player_id,
            total_rounds_analyzed: total_rounds as u32,
            current_playstyle: learning_profile.playstyle_pattern.clone(),
            recent_avg_impact,
            most_successful_weapon: best_weapon,
            adaptation_rate: learning_profile.adaptive_preferences.adaptation_rate,
            learning_trend,
            confidence_score: learning_profile.adaptive_preferences.confidence_threshold,
        })
    }

    fn calculate_ml_confidence(
        &self,
        player: &Player,
        weapon: &Option<Weapon>,
        context: &RoundContext,
        base_confidence: f32,
    ) -> f32 {
        // If player doesn't have ML enabled, return base confidence
        let learning_profile = match &player.learning_profile {
            Some(profile) => profile,
            None => return base_confidence,
        };

        // If insufficient data, return base confidence with slight penalty
        if learning_profile.performance_history.len() < 5 {
            return base_confidence * 0.9;
        }

        let mut ml_confidence = base_confidence;

        // Factor 1: Weapon success rate (40% weight)
        if let Some(weapon) = weapon {
            let weapon_key = format!("{:?}", weapon);
            if let Some(success_rate) = learning_profile.adaptive_preferences.success_rates.get(&weapon_key) {
                let weapon_confidence = success_rate.successful_rounds as f32 / success_rate.total_rounds as f32;
                ml_confidence = ml_confidence * 0.6 + weapon_confidence * 0.4;
            }
        }

        // Factor 2: Contextual success (25% weight)
        let context_key = format!("{:?}_{:?}", context.round_type, context.team_economy);
        if let Some(weapon) = weapon {
            let weapon_key = format!("{:?}", weapon);
            if let Some(success_rate) = learning_profile.adaptive_preferences.success_rates.get(&weapon_key) {
                if let Some(contextual) = success_rate.context_success.get(&context_key) {
                    let contextual_confidence = contextual.success_count as f32 / contextual.total_count as f32;
                    ml_confidence = ml_confidence * 0.75 + contextual_confidence * 0.25;
                }
            }
        }

        // Factor 3: Recent performance trend (20% weight)
        let recent_performance: Vec<&BuyPerformanceMetrics> = learning_profile.performance_history
            .iter()
            .rev()
            .take(10)
            .collect();

        if !recent_performance.is_empty() {
            let recent_avg_impact = recent_performance
                .iter()
                .filter_map(|m| m.performance_outcome.as_ref())
                .map(|o| o.round_impact_score)
                .sum::<f32>() / recent_performance.len() as f32;

            ml_confidence = ml_confidence * 0.8 + recent_avg_impact * 0.2;
        }

        // Factor 4: Playstyle match bonus (10% weight)
        let playstyle_bonus = match (&learning_profile.playstyle_pattern, weapon) {
            (PlaystylePattern::Aggressive, Some(Weapon::Operator | Weapon::Vandal | Weapon::Phantom)) => 0.1,
            (PlaystylePattern::Conservative, Some(Weapon::Guardian | Weapon::Bulldog)) => 0.1,
            (PlaystylePattern::Supportive, _) if context.round_type == RoundType::AntiEco => 0.1,
            (PlaystylePattern::Adaptive, _) => 0.05, // Always gets small bonus
            (PlaystylePattern::Experimental, _) => -0.05, // Slight penalty for unpredictability
            _ => 0.0,
        };

        ml_confidence += playstyle_bonus;

        // Factor 5: Confidence threshold adjustment (5% weight)
        let confidence_threshold = learning_profile.adaptive_preferences.confidence_threshold;
        if ml_confidence < confidence_threshold {
            ml_confidence *= 0.95; // Slightly reduce if below threshold
        }

        ml_confidence
    }

    // Phase 3.5: Neural Network Feature Engineering

    fn extract_game_features(&self, player: &Player, context: &RoundContext) -> GameStateFeatures {
        let role = player.agent.get_role();
        let team_players: Vec<&Player> = self.players.values()
            .filter(|p| p.team == player.team)
            .collect();
        let enemy_players: Vec<&Player> = self.players.values()
            .filter(|p| p.team != player.team)
            .collect();

        // Calculate player statistics
        let player_stats = self.calculate_player_round_stats(player.id);
        
        // Normalize features to 0-1 range for better neural network performance
        let normalized_credits = (player.current_credits as f32 / 9000.0).min(1.0);
        let team_economy = team_players.iter().map(|p| p.current_credits).sum::<u32>() as f32 / (team_players.len() as f32 * 9000.0);
        let enemy_economy = enemy_players.iter().map(|p| p.current_credits).sum::<u32>() as f32 / (enemy_players.len() as f32 * 9000.0);
        
        GameStateFeatures {
            // Player features (normalized)
            player_credits: normalized_credits,
            player_kills: (player_stats.kills as f32 / 5.0).min(1.0),
            player_deaths: (player_stats.deaths as f32 / 5.0).min(1.0),
            player_assists: (player_stats.assists as f32 / 10.0).min(1.0),
            player_survival_rate: player_stats.survival_rate,
            player_avg_damage: (player_stats.avg_damage / 200.0).min(1.0),
            player_headshot_rate: player.skills.hs,
            player_aim_skill: player.skills.aim,
            player_utility_skill: player.skills.util,
            player_movement_skill: player.skills.movement,
            
            // Game context features
            round_number: (self.state.current_round as f32 / 25.0).min(1.0),
            team_score: (self.state.attacker_score as f32 / 13.0).min(1.0),
            enemy_score: (self.state.defender_score as f32 / 13.0).min(1.0),
            team_economy,
            enemy_economy,
            loss_streak: (context.loss_streak as f32 / 5.0).min(1.0),
            round_type_pistol: if context.round_type == RoundType::Pistol { 1.0 } else { 0.0 },
            round_type_eco: if context.round_type == RoundType::Eco { 1.0 } else { 0.0 },
            round_type_fullbuy: if context.round_type == RoundType::FullBuy { 1.0 } else { 0.0 },
            round_type_forcebuy: if context.round_type == RoundType::ForceBuy { 1.0 } else { 0.0 },
            round_type_antieco: if context.round_type == RoundType::AntiEco { 1.0 } else { 0.0 },
            map_side: if player.team == Team::Attackers { 0.0 } else { 1.0 },
            
            // Role features (one-hot encoding)
            is_duelist: if role == AgentRole::Duelist { 1.0 } else { 0.0 },
            is_controller: if role == AgentRole::Controller { 1.0 } else { 0.0 },
            is_initiator: if role == AgentRole::Initiator { 1.0 } else { 0.0 },
            is_sentinel: if role == AgentRole::Sentinel { 1.0 } else { 0.0 },
            
            // Team coordination features (simplified for now)
            team_rifles_needed: 0.5, // TODO: Calculate from team strategy
            team_utility_budget: 0.5, // TODO: Calculate from team needs
            coordination_priority: 0.5, // TODO: Calculate based on role and situation
            team_strategy_eco: if context.round_type == RoundType::Eco { 1.0 } else { 0.0 },
            team_strategy_force: if context.round_type == RoundType::ForceBuy { 1.0 } else { 0.0 },
            team_strategy_full: if context.round_type == RoundType::FullBuy { 1.0 } else { 0.0 },
        }
    }

    fn calculate_player_round_stats(&self, player_id: u32) -> PlayerRoundStats {
        let player_events: Vec<&GameEvent> = self.events
            .iter()
            .filter(|event| {
                match event {
                    GameEvent::Kill { killer_id, victim_id, .. } => *killer_id == player_id || *victim_id == player_id,
                    _ => false,
                }
            })
            .collect();

        let mut kills = 0u8;
        let mut deaths = 0u8;
        let mut total_damage = 0u32;
        let mut damage_instances = 0u32;

        for event in player_events {
            match event {
                GameEvent::Kill { killer_id, victim_id, .. } => {
                    if *killer_id == player_id {
                        kills += 1;
                        total_damage += 140; // Approximate damage for a kill
                        damage_instances += 1;
                    }
                    if *victim_id == player_id {
                        deaths += 1;
                    }
                }
                _ => {}
            }
        }

        let survival_rate = if kills + deaths == 0 {
            1.0
        } else {
            1.0 - (deaths as f32 / (kills + deaths) as f32)
        };

        let avg_damage = if damage_instances == 0 {
            0.0
        } else {
            total_damage as f32 / damage_instances as f32
        };

        PlayerRoundStats {
            kills,
            deaths,
            assists: 0, // TODO: Track assists
            survival_rate,
            avg_damage,
        }
    }

    pub fn make_neural_buy_decision(&mut self, player_id: u32, context: &RoundContext) -> Result<BuyDecision, String> {
        let player = self.players.get(&player_id).ok_or("Player not found")?;
        
        // Extract features
        let features = self.extract_game_features(player, context);
        
        // Get prediction from neural network if available
        if let Some(ref mut predictor) = self.players.get_mut(&player_id).unwrap().neural_predictor {
            match predictor.predict(&features) {
                Ok(prediction) => {
                    // Convert neural network output to BuyDecision
                    let mut best_weapon: Option<Weapon> = None;
                    let mut max_confidence = 0.0f32;
                    
                    // Map weapon confidences to actual weapons (simplified mapping)
                    let weapons = [
                        Weapon::Classic, Weapon::Ghost, Weapon::Sheriff, Weapon::Spectre,
                        Weapon::Bulldog, Weapon::Guardian, Weapon::Phantom, Weapon::Vandal,
                        Weapon::Marshal, Weapon::Operator
                    ];
                    
                    for (i, weapon) in weapons.iter().enumerate() {
                        if i < prediction.weapon_confidences.len() {
                            let confidence = prediction.weapon_confidences[i];
                            if confidence > max_confidence && confidence > 0.5 {
                                max_confidence = confidence;
                                best_weapon = Some(weapon.clone());
                            }
                        }
                    }
                    
                    // Determine armor based on neural network output
                    let armor = if prediction.armor_priority > 0.7 {
                        ArmorType::Heavy
                    } else if prediction.armor_priority > 0.4 {
                        ArmorType::Light
                    } else {
                        ArmorType::None
                    };
                    
                    // Calculate abilities budget based on utility priority
                    let abilities_budget = (prediction.utility_priority * 1200.0) as u32;
                    
                    // Calculate total cost
                    let weapon_cost = if let Some(ref weapon) = best_weapon {
                        self.weapon_stats.get(weapon).map(|stats| stats.price).unwrap_or(0)
                    } else {
                        0
                    };
                    
                    let armor_cost = match armor {
                        ArmorType::Heavy => 1000,
                        ArmorType::Light => 400,
                        ArmorType::None => 0,
                    };
                    
                    let total_cost = weapon_cost + armor_cost + abilities_budget;
                    
                    Ok(BuyDecision {
                        primary_weapon: best_weapon,
                        secondary_weapon: Weapon::Classic, // Default
                        armor,
                        abilities_budget,
                        total_cost,
                        confidence: prediction.overall_confidence,
                        coordination_priority: 0.5, // TODO: Extract from neural network
                    })
                }
                Err(e) => Err(format!("Neural network prediction failed: {:?}", e))
            }
        } else {
            // Fall back to traditional buy decision if neural network not available
            let player = self.players.get(&player_id).unwrap();
            Ok(self.make_dynamic_buy_decision(player, context))
        }
    }

    fn create_neural_target_from_outcome(&self, buy_decision: &BuyDecision, outcome: &RoundPerformanceOutcome) -> BuyDecisionTarget {
        // Create target based on the actual buy decision and its performance outcome
        let weapons = [
            Weapon::Classic, Weapon::Ghost, Weapon::Sheriff, Weapon::Spectre,
            Weapon::Bulldog, Weapon::Guardian, Weapon::Phantom, Weapon::Vandal,
            Weapon::Marshal, Weapon::Operator
        ];
        
        let mut weapon_confidences = vec![0.0; 10];
        
        // If the round was successful, increase confidence for the chosen weapon
        if let Some(ref chosen_weapon) = buy_decision.primary_weapon {
            for (i, weapon) in weapons.iter().enumerate() {
                if weapon == chosen_weapon {
                    // Set confidence based on round performance
                    weapon_confidences[i] = if outcome.round_impact_score > 0.7 {
                        0.9 // Very successful
                    } else if outcome.round_impact_score > 0.5 {
                        0.7 // Successful
                    } else if outcome.round_impact_score > 0.3 {
                        0.5 // Neutral
                    } else {
                        0.2 // Poor performance
                    };
                    break;
                }
            }
        }
        
        // Calculate eco/force buy targets based on outcome and cost efficiency
        let _cost_efficiency = if buy_decision.total_cost > 0 {
            outcome.round_impact_score * 1000.0 / buy_decision.total_cost as f32
        } else {
            outcome.round_impact_score
        };
        
        let should_eco = if buy_decision.total_cost < 1000 && outcome.round_impact_score > 0.5 {
            0.8 // Eco was effective
        } else if buy_decision.total_cost > 4000 && outcome.round_impact_score < 0.3 {
            0.7 // Expensive buy was ineffective, should have eco'd
        } else {
            0.3 // Neutral
        };
        
        let should_force = if buy_decision.total_cost > 2000 && buy_decision.total_cost < 4000 && outcome.round_impact_score > 0.6 {
            0.8 // Force buy was effective
        } else {
            0.3 // Neutral
        };
        
        BuyDecisionTarget {
            weapon_confidences,
            should_eco,
            should_force,
            armor_priority: match buy_decision.armor {
                ArmorType::Heavy => if outcome.deaths == 0 { 0.9 } else { 0.6 },
                ArmorType::Light => if outcome.deaths == 0 { 0.7 } else { 0.4 },
                ArmorType::None => if outcome.deaths == 0 { 0.3 } else { 0.1 },
            },
            utility_priority: (buy_decision.abilities_budget as f32 / 1200.0) * outcome.round_impact_score,
            overall_confidence: outcome.round_impact_score,
        }
    }
    
    /// Enable neural learning for all players in the simulation
    pub fn enable_neural_learning_for_all_players(&mut self) -> Vec<String> {
        let mut errors = Vec::new();
        
        for (player_id, player) in self.players.iter_mut() {
            match player.enable_neural_learning() {
                Ok(_) => {
                    // Success - neural learning enabled
                }
                Err(e) => {
                    errors.push(format!("Player {}: {}", player_id, e));
                }
            }
        }
        
        errors
    }
    
    /// Enable traditional ML for all players (existing adaptive preferences system)
    pub fn enable_ml_for_all_players(&mut self) {
        for player in self.players.values_mut() {
            player.adaptive_preferences = Some(AdaptivePreferences::new());
            player.performance_tracker = Some(PerformanceTracker::new());
        }
    }
    
    /// Train neural networks for all players and return loss values
    pub fn train_neural_networks(&mut self) -> HashMap<u32, f32> {
        let mut training_losses = HashMap::new();
        
        // For each player with neural learning enabled
        for (player_id, player) in self.players.iter_mut() {
            if let Some(ref mut predictor) = player.neural_predictor {
                // Get recent training examples (simplified - in real implementation would use actual training data)
                match predictor.train_step() {
                    Ok(loss) => {
                        training_losses.insert(*player_id, loss);
                    }
                    Err(_) => {
                        // Training failed, continue with other players
                        training_losses.insert(*player_id, f32::INFINITY);
                    }
                }
            }
        }
        
        training_losses
    }
    
    /// Get learning insights for a specific player
    pub fn get_player_learning_insights(&self, player_id: u32) -> Option<PlayerLearningInsights> {
        let mut insights = self.players.get(&player_id)?.neural_predictor.as_ref()?.get_learning_insights()?;
        insights.player_id = player_id;
        Some(insights)
    }

    fn simulate_combat(&mut self, alive_attackers: &[u32], alive_defenders: &[u32]) {
        // Safety check: ensure both teams have alive players
        if alive_attackers.is_empty() || alive_defenders.is_empty() {
            return;
        }

        let mut rng = rand::rng();

        let attacker_id = alive_attackers[rng.random_range(0..alive_attackers.len())];
        let defender_id = alive_defenders[rng.random_range(0..alive_defenders.len())];

        // Double-check both players are still alive
        let attacker_still_alive = self.players.get(&attacker_id).is_some_and(|p| p.is_alive);
        let defender_still_alive = self.players.get(&defender_id).is_some_and(|p| p.is_alive);

        if !attacker_still_alive || !defender_still_alive {
            return; // Skip combat if either player is dead
        }

        let attacker_player_data = self.players.get(&attacker_id).unwrap().clone();
        let defender_player_data = self.players.get(&defender_id).unwrap().clone();

        // Use equipped weapon for combat effectiveness
        let attacker_weapon = attacker_player_data
            .current_loadout
            .primary_weapon
            .unwrap_or(
                attacker_player_data
                    .current_loadout
                    .secondary_weapon
                    .clone(),
            );
        let defender_weapon = defender_player_data
            .current_loadout
            .primary_weapon
            .unwrap_or(
                defender_player_data
                    .current_loadout
                    .secondary_weapon
                    .clone(),
            );

        // Calculate weapon effectiveness multipliers
        let attacker_weapon_effectiveness = self.calculate_weapon_effectiveness(&attacker_weapon);
        let defender_weapon_effectiveness = self.calculate_weapon_effectiveness(&defender_weapon);

        // Enhanced combat calculation with weapon stats
        let attacker_base_skill =
            attacker_player_data.skills.aim * 0.7 + attacker_player_data.skills.hs * 0.3;
        let defender_base_skill =
            defender_player_data.skills.aim * 0.7 + defender_player_data.skills.hs * 0.3;

        let attacker_effective_skill = attacker_base_skill * attacker_weapon_effectiveness;
        let defender_effective_skill = defender_base_skill * defender_weapon_effectiveness;

        // Fire rate advantage
        let attacker_fire_rate = self.weapon_stats[&attacker_weapon].fire_rate;
        let defender_fire_rate = self.weapon_stats[&defender_weapon].fire_rate;

        let fire_rate_advantage = (attacker_fire_rate / defender_fire_rate).clamp(0.5, 2.0);

        let mut attacker_win_chance =
            0.5 + (attacker_effective_skill - defender_effective_skill) * 0.3;
        attacker_win_chance *= fire_rate_advantage;
        attacker_win_chance = attacker_win_chance.clamp(0.1f32, 0.9f32);

        // Determine hit location and headshot
        let is_attacker_headshot = rng.random::<f32>() < attacker_player_data.skills.hs;
        let is_defender_headshot = rng.random::<f32>() < defender_player_data.skills.hs;

        let hit_body_part = if is_attacker_headshot || is_defender_headshot {
            BodyPart::Head
        } else if rng.random::<f32>() < 0.7 {
            BodyPart::Body
        } else {
            BodyPart::Legs
        };

        // Simulate engagement range (10-50 meters)
        let engagement_range = rng.random_range(10.0..50.0);

        if rng.random::<f32>() < attacker_win_chance {
            // Attacker wins
            let damage = self.calculate_weapon_damage(
                &attacker_weapon,
                &defender_player_data.current_loadout.armor,
                hit_body_part,
                engagement_range,
            );

            if let Some(victim) = self.players.get_mut(&defender_id) {
                victim.take_damage(damage);
            }

            // Only record kill if both killer is alive and victim actually died
            if let (Some(killer), Some(victim)) = (
                self.players.get(&attacker_id),
                self.players.get(&defender_id),
            ) {
                if killer.is_alive && !victim.is_alive {
                    self.record_event(GameEvent::Kill {
                        timestamp: self.state.current_timestamp,
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
                engagement_range,
            );

            if let Some(victim) = self.players.get_mut(&attacker_id) {
                victim.take_damage(damage);
            }

            // Only record kill if both killer is alive and victim actually died
            if let (Some(killer), Some(victim)) = (
                self.players.get(&defender_id),
                self.players.get(&attacker_id),
            ) {
                if killer.is_alive && !victim.is_alive {
                    self.record_event(GameEvent::Kill {
                        timestamp: self.state.current_timestamp,
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

    fn calculate_round_rewards(
        &mut self,
        winning_team: &Team,
        _reason: &RoundEndReason,
        spike_planted: bool,
    ) {
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
                    0 => 1900, // First loss
                    1 => 2400, // Second consecutive loss
                    _ => 2900, // Third+ consecutive loss
                };

                // Update loss streak
                self.loss_streaks
                    .insert(player.team.clone(), loss_streak + 1);

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

    fn calculate_weapon_damage(
        &self,
        weapon: &Weapon,
        armor_type: &ArmorType,
        body_part: BodyPart,
        range_meters: f32,
    ) -> u32 {
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
                if range_meters <= 15.0 {
                    1.0
                } else if range_meters <= 30.0 {
                    0.85
                } else {
                    0.7
                }
            }
            Weapon::Spectre | Weapon::Stinger => {
                if range_meters <= 20.0 {
                    1.0
                } else {
                    0.75
                }
            }
            _ => 1.0, // No damage falloff for most weapons
        };

        (base_damage as f32 * damage_multiplier) as u32
    }

    fn calculate_weapon_effectiveness(&self, weapon: &Weapon) -> f32 {
        match weapon {
            Weapon::Operator => 1.5, // Massive aim advantage
            Weapon::Vandal => 1.2,   // High damage, good accuracy
            Weapon::Phantom => 1.15, // Good balance
            Weapon::Guardian => 1.1, // High damage, slower
            Weapon::Spectre => 0.9,  // Good for close range
            Weapon::Sheriff => 0.8,  // High damage pistol
            Weapon::Ghost => 0.6,    // Balanced pistol
            Weapon::Classic => 0.4,  // Basic weapon
            _ => 0.7,                // Default effectiveness
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

    #[allow(dead_code)]
    fn simulate_buy_phase(&mut self) {
        self.record_event(GameEvent::BuyPhaseStart {
            timestamp: self.state.current_timestamp,
            round_number: self.state.current_round,
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
            if player.current_credits >= 5700 {
                // Operator + Heavy armor
                player.current_loadout.primary_weapon = Some(Weapon::Operator);
                player.current_loadout.armor = ArmorType::Heavy;
                player.current_credits -= 5700;
            } else if player.current_credits >= 3900 {
                // Vandal + Heavy armor
                player.current_loadout.primary_weapon = Some(Weapon::Vandal);
                player.current_loadout.armor = ArmorType::Heavy;
                player.current_credits -= 3900;
            } else if player.current_credits >= 1600 {
                // SMG buy
                player.current_loadout.primary_weapon = Some(Weapon::Spectre);
                player.current_credits -= 1600;
                if player.current_credits >= 400 {
                    player.current_loadout.armor = ArmorType::Light;
                    player.current_credits -= 400;
                }
            } else if player.current_credits >= 800 {
                // Pistol upgrade
                player.current_loadout.secondary_weapon = Weapon::Sheriff;
                player.current_credits -= 800;
            }
        }

        self.advance_time(30000); // 30 second buy phase

        self.record_event(GameEvent::BuyPhaseEnd {
            timestamp: self.state.current_timestamp,
            round_number: self.state.current_round,
        });
    }

    // Convenience method for running entire simulation at once (legacy mode)
    pub fn run_simulation_to_completion(&mut self) -> Result<(), String> {
        self.start_simulation();

        let mut tick_count = 0;
        const MAX_TICKS_PER_MATCH: u64 = 50000; // Prevent infinite loops (about 4 hours at 500ms per tick)

        while !matches!(self.state.phase, SimulationPhase::MatchEnd { .. }) {
            tick_count += 1;
            if tick_count > MAX_TICKS_PER_MATCH {
                return Err(format!(
                    "Match simulation exceeded maximum tick limit ({}). Possible infinite loop detected.",
                    MAX_TICKS_PER_MATCH
                ));
            }

            self.advance_tick()?;
        }

        Ok(())
    }

    // High-level control methods for frontend
    pub fn advance_round(&mut self) -> Result<(), String> {
        let mut tick_count = 0;
        const MAX_TICKS_PER_ROUND: u64 = 2000; // Prevent infinite loops (10 minutes at 500ms per tick)

        loop {
            tick_count += 1;
            if tick_count > MAX_TICKS_PER_ROUND {
                return Err(format!(
                    "Round advancement exceeded maximum tick limit ({}). Possible infinite loop detected.",
                    MAX_TICKS_PER_ROUND
                ));
            }

            self.advance_tick()?;
            if matches!(
                self.state.phase,
                SimulationPhase::RoundEnd { .. } | SimulationPhase::MatchEnd { .. }
            ) {
                break;
            }
        }
        Ok(())
    }

    pub fn advance_multiple_ticks(&mut self, count: u32) -> Result<(), String> {
        for _ in 0..count {
            self.advance_tick()?;
            if matches!(self.state.phase, SimulationPhase::MatchEnd { .. }) {
                break;
            }
        }
        Ok(())
    }
}
