pub mod db;
pub mod models;
pub mod offers;
pub mod ranked;
pub mod sim;
pub mod simulation_manager;

// Re-export enums from simulation_manager for external use
pub use simulation_manager::{AdvanceMode, SimulationCommand};

// Re-export core simulation types for examples
pub use sim::{
    Agent, AgentRole, Player, PlayerSkills, Team, ValorantSimulation,
    RoundType, RoundContext, EconomyState, RoundEndReason, Weapon, ArmorType,
    BuyDecision, GameStateFeatures, NeuralBuyPredictor, PlayerLearningInsights
};
