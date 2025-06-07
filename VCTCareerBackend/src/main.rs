mod models;
mod db;
mod offers;

use actix_cors::Cors;
use actix_web::post;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use deadpool_postgres::{Manager, Pool};
use log::debug;
use rand::seq::{IndexedRandom, SliceRandom};
use rand::rng;
use tokio_postgres::NoTls;
use tokio_postgres::Config;
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_actix_web::AppExt;
use VCTCareerBackend::ranked::{estimate_rr_change, MapPoolRequest, MatchInput, RandomMapResponse, RankTier, RrEstimateResponse};
use std::str::FromStr;
use crate::db::weighted_tier;
use crate::offers::OfferRequest;
use crate::db::{TeamQuery, get_teams_handler};
use crate::models::{CareerInfo, CreateSimulationRequest, CreateSimulationResponse, AdvanceSimulationRequest, SimulationControlRequest, EventFilterRequest};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use VCTCareerBackend::sim::{ValorantSimulation, Player, Agent, Team};
use VCTCareerBackend::simulation_manager;

type SimulationManager = Arc<Mutex<HashMap<uuid::Uuid, ValorantSimulation>>>;

fn create_simulation_manager() -> SimulationManager {
    Arc::new(Mutex::new(HashMap::new()))
}

fn parse_agent(agent_str: &str) -> Result<Agent, String> {
    match agent_str {
        "Jett" => Ok(Agent::Jett),
        "Raze" => Ok(Agent::Raze),
        "Phoenix" => Ok(Agent::Phoenix),
        "Breach" => Ok(Agent::Breach),
        "Sova" => Ok(Agent::Sova),
        "Sage" => Ok(Agent::Sage),
        "Omen" => Ok(Agent::Omen),
        "Brimstone" => Ok(Agent::Brimstone),
        "Viper" => Ok(Agent::Viper),
        "Cypher" => Ok(Agent::Cypher),
        "Killjoy" => Ok(Agent::Killjoy),
        "Skye" => Ok(Agent::Skye),
        "Yoru" => Ok(Agent::Yoru),
        "Astra" => Ok(Agent::Astra),
        "Kayo" => Ok(Agent::Kayo),
        "Chamber" => Ok(Agent::Chamber),
        "Neon" => Ok(Agent::Neon),
        "Fade" => Ok(Agent::Fade),
        "Harbor" => Ok(Agent::Harbor),
        "Gekko" => Ok(Agent::Gekko),
        "Deadlock" => Ok(Agent::Deadlock),
        "Iso" => Ok(Agent::Iso),
        "Clove" => Ok(Agent::Clove),
        _ => Err(format!("Unknown agent: {}", agent_str)),
    }
}

fn parse_team(team_str: &str) -> Result<Team, String> {
    match team_str {
        "Attackers" => Ok(Team::Attackers),
        "Defenders" => Ok(Team::Defenders),
        _ => Err(format!("Unknown team: {}", team_str)),
    }
}
use utoipa_swagger_ui::SwaggerUi;
use uuid::Uuid;

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct CreateCareerResponse {
    pub starting_tier: String,
    pub career_info: CareerInfo,
}

#[get("/")]
async fn index() -> impl Responder {
    debug!("GET / called");
    HttpResponse::Ok().body("VCTCareer Backend is running!")
}

#[utoipa::path(
    post,
    path = "/createCareer", 
    request_body = CareerInfo,
    params(
        ("age" = u32, Query, description = "Player age"),
        ("current_rank" = String, Query, description = "Current rank"),
        ("past_experience" = String, Query, description = "Past experience"),
        ("division" = String, Query, description = "Division"),
    ),
    responses(
        (status = 200, description = "Career created", body = CreateCareerResponse),
        (status = 400, description = "Invalid query parameters", body = String),
    )
)]
#[post("/createCareer")]
async fn create_career(info: web::Json<CareerInfo>) -> impl Responder {
    debug!("POST /createCareer called with: {:?}", info);
    let tier = weighted_tier(&info);
    let response = serde_json::json!({
        "starting_tier": tier.as_str(),
        "career_info": info.into_inner()
    });
    debug!("Response: {}", response);
    HttpResponse::Ok().json(response)
}

#[utoipa::path(
    get,
    path = "/teams",
    params(
        ("team_name" = Option<String>, Query, description = "Team name to filter"),
        ("ranking" = Option<i32>, Query, description = "Ranking to filter"),
        ("tier" = Option<String>, Query, description = "Tier to filter"),
        ("region" = Option<String>, Query, description = "Region to filter"),
    ),
    responses(
        (status = 200, description = "List of teams", body = Vec<crate::models::Team>),
        (status = 400, description = "Invalid query parameters", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
#[get("/teams")]
async fn get_teams(query: web::Query<TeamQuery>) -> impl Responder {
    get_teams_handler(query).await
}

#[utoipa::path(
    get,
    path = "/generateOffers",
    params(
        ("tier" = String, Query, description = "Tier for offer generation"),
        ("count" = usize, Query, description = "Number of offers to generate"),
        ("region" = Option<String>, Query, description = "Region for offers"),
        ("overall" = Option<u32>, Query, description = "Player overall rating"),
    ),
    responses(
        (status = 200, description = "List of offers", body = Vec<crate::offers::Offer>),
        (status = 400, description = "Invalid query parameters", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
#[get("/generateOffers")]
async fn generate_offers(
    pool: web::Data<Pool>,
    query: web::Query<OfferRequest>
) -> impl Responder {
    match offers::generate_offers(pool.get_ref(), &query).await {
        Ok(offers) => HttpResponse::Ok().json(offers),
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

#[utoipa::path(
    get,
    path = "/estimate_rr",
    params(
        ("rank" = String, Query, description = "Player's visible rank"),
        ("is_win" = bool, Query, description = "True if match was won"),
        ("rounds_won" = i32, Query, description = "Rounds won in the match"),
        ("rounds_lost" = i32, Query, description = "Rounds lost in the match"),
        ("acs_percentile" = f32, Query, description = "ACS percentile from 0.0 to 1.0"),
        ("hidden_mmr" = i32, Query, description = "Player's hidden MMR"),
        ("five_stack_penalty" = bool, Query, description = "Whether a 5-stack penalty applies")
    ),
    responses(
        (status = 200, description = "RR change estimated", body = RrEstimateResponse),
        (status = 400, description = "Invalid rank string", body = String),
    )
)]
#[get("/estimate_rr")]
async fn estimate_rr(input: web::Query<MatchInput>) -> impl Responder {
    match RankTier::from_str(input.rank.trim()) {
        Some(rank) => {
            let rr_change = estimate_rr_change(&input, rank);
            HttpResponse::Ok().json(RrEstimateResponse { rr_change })
        }
        None => HttpResponse::BadRequest().body("Invalid rank provided"),
    }
}

#[utoipa::path(
    get,
    path = "/random_map",
    params(
        ("maps", Query, description = "List of maps to choose from. Use repeated 'maps' params, e.g. /random_map?maps=Ascent&maps=Bind")
    ),
    responses(
        (status = 200, description = "Random map selected", body = RandomMapResponse),
        (status = 400, description = "Empty map pool", body = String),
    )
)]
#[get("/random_map")]
async fn random_map(query: web::Query<MapPoolRequest>) -> impl Responder {
    let pool = &query.maps;
    if pool.is_empty() {
        return HttpResponse::BadRequest().body("Map pool cannot be empty");
    }

    let mut maps = pool.clone();
    let mut rng = rand::thread_rng();
    maps.shuffle(&mut rng);
    let map = maps.choose(&mut rng).unwrap().clone();

    HttpResponse::Ok().json(RandomMapResponse { selected_map: map })
}

// Simulation API Endpoints

#[utoipa::path(
    post,
    path = "/simulation/create",
    request_body = CreateSimulationRequest,
    responses(
        (status = 200, description = "Simulation created", body = CreateSimulationResponse),
        (status = 400, description = "Invalid request", body = String),
    )
)]
#[post("/simulation/create")]
async fn create_simulation(
    sim_manager: web::Data<SimulationManager>,
    request: web::Json<CreateSimulationRequest>
) -> impl Responder {
    let mut sim = ValorantSimulation::new();
    let simulation_id = sim.get_current_state().id;
    
    // Convert and add players to simulation
    for player_data in &request.players {
        let agent = match parse_agent(&player_data.agent) {
            Ok(a) => a,
            Err(e) => return HttpResponse::BadRequest().body(e),
        };
        let team = match parse_team(&player_data.team) {
            Ok(t) => t,
            Err(e) => return HttpResponse::BadRequest().body(e),
        };
        
        let player = Player::new(
            player_data.id,
            player_data.name.clone(),
            agent,
            team,
            player_data.aim_skill,
            player_data.hs_skill,
            player_data.movement_skill,
            player_data.util_skill,
        );
        
        sim.add_player(player);
    }
    
    // Store simulation in manager
    sim_manager.lock().unwrap().insert(simulation_id, sim);
    
    HttpResponse::Ok().json(CreateSimulationResponse {
        simulation_id: simulation_id.to_string(),
        message: "Simulation created successfully".to_string(),
    })
}

#[utoipa::path(
    get,
    path = "/simulation/{id}/state",
    params(
        ("id" = String, Path, description = "Simulation ID")
    ),
    responses(
        (status = 200, description = "Simulation state", body = serde_json::Value),
        (status = 404, description = "Simulation not found", body = String),
    )
)]
#[get("/simulation/{id}/state")]
async fn get_simulation_state(
    sim_manager: web::Data<SimulationManager>,
    path: web::Path<String>
) -> impl Responder {
    let simulation_id = path.into_inner();
    let uuid_id = match Uuid::parse_str(&simulation_id) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid simulation ID format"),
    };
    
    let simulations = sim_manager.lock().unwrap();
    match simulations.get(&uuid_id) {
        Some(sim) => HttpResponse::Ok().json(sim.get_current_state()),
        None => HttpResponse::NotFound().body("Simulation not found"),
    }
}

#[utoipa::path(
    post,
    path = "/simulation/{id}/advance",
    params(
        ("id" = String, Path, description = "Simulation ID")
    ),
    request_body = AdvanceSimulationRequest,
    responses(
        (status = 200, description = "Simulation advanced", body = String),
        (status = 404, description = "Simulation not found", body = String),
        (status = 400, description = "Invalid request", body = String),
    )
)]
#[post("/simulation/{id}/advance")]
async fn advance_simulation(
    sim_manager: web::Data<SimulationManager>,
    path: web::Path<String>,
    request: web::Json<AdvanceSimulationRequest>
) -> impl Responder {
    let simulation_id = path.into_inner();
    let uuid_id = match Uuid::parse_str(&simulation_id) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid simulation ID format"),
    };
    
    let mut simulations = sim_manager.lock().unwrap();
    let sim = match simulations.get_mut(&uuid_id) {
        Some(s) => s,
        None => return HttpResponse::NotFound().body("Simulation not found"),
    };
    
    let result = match request.mode.as_deref() {
        Some("tick") => {
            let tick_count = request.ticks.unwrap_or(1);
            sim.advance_multiple_ticks(tick_count)
        }
        Some("round") => {
            sim.advance_round()
        }
        Some("match") => {
            sim.run_simulation_to_completion()
        }
        _ => {
            sim.advance_tick()
        }
    };
    
    match result {
        Ok(()) => HttpResponse::Ok().body("Simulation advanced successfully"),
        Err(e) => HttpResponse::BadRequest().body(e),
    }
}

#[utoipa::path(
    put,
    path = "/simulation/{id}/control",
    params(
        ("id" = String, Path, description = "Simulation ID")
    ),
    request_body = SimulationControlRequest,
    responses(
        (status = 200, description = "Simulation control applied", body = String),
        (status = 404, description = "Simulation not found", body = String),
        (status = 400, description = "Invalid request", body = String),
    )
)]
#[actix_web::put("/simulation/{id}/control")]
async fn control_simulation(
    sim_manager: web::Data<SimulationManager>,
    path: web::Path<String>,
    request: web::Json<SimulationControlRequest>
) -> impl Responder {
    let simulation_id = path.into_inner();
    match simulation_manager::control_simulation_legacy(
        &sim_manager, 
        simulation_id, 
        request.action.clone(), 
        request.speed
    ) {
        Ok(()) => HttpResponse::Ok().body("Control applied successfully"),
        Err(e) => HttpResponse::BadRequest().body(e),
    }
}

#[utoipa::path(
    get,
    path = "/simulation/{id}/events",
    params(
        ("id" = String, Path, description = "Simulation ID"),
        ("event_types" = Option<Vec<String>>, Query, description = "Filter by event types"),
        ("player_ids" = Option<Vec<u32>>, Query, description = "Filter by player IDs"),
        ("start_timestamp" = Option<u64>, Query, description = "Filter by start timestamp"),
        ("end_timestamp" = Option<u64>, Query, description = "Filter by end timestamp"),
    ),
    responses(
        (status = 200, description = "Simulation events", body = Vec<VCTCareerBackend::sim::GameEvent>),
        (status = 404, description = "Simulation not found", body = String),
    )
)]
#[get("/simulation/{id}/events")]
async fn get_simulation_events(
    sim_manager: web::Data<SimulationManager>,
    path: web::Path<String>,
    query: web::Query<EventFilterRequest>
) -> impl Responder {
    let simulation_id = path.into_inner();
    let filter = query.into_inner();
    // Convert local EventFilterRequest to the one expected by simulation_manager
    let sim_filter = VCTCareerBackend::models::EventFilterRequest {
        event_types: filter.event_types,
        player_ids: filter.player_ids,
        round_numbers: filter.round_numbers,
        start_timestamp: filter.start_timestamp,
        end_timestamp: filter.end_timestamp,
    };
    match simulation_manager::get_simulation_events_legacy(&sim_manager, simulation_id, sim_filter) {
        Ok(events) => HttpResponse::Ok().json(events),
        Err(e) => HttpResponse::NotFound().body(e),
    }
}

#[utoipa::path(
    get,
    path = "/simulation/{id}/stats",
    params(
        ("id" = String, Path, description = "Simulation ID")
    ),
    responses(
        (status = 200, description = "Player statistics", body = Vec<VCTCareerBackend::sim::PlayerStats>),
        (status = 404, description = "Simulation not found", body = String),
    )
)]
#[get("/simulation/{id}/stats")]
async fn get_simulation_stats(
    sim_manager: web::Data<SimulationManager>,
    path: web::Path<String>
) -> impl Responder {
    let simulation_id = path.into_inner();
    match simulation_manager::get_simulation_stats_legacy(&sim_manager, simulation_id) {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(e) => HttpResponse::NotFound().body(e),
    }
}

// Phase 2 API Endpoints
#[utoipa::path(
    get,
    path = "/simulation/{id}/live-stats",
    responses(
        (status = 200, description = "Live match statistics", body = VCTCareerBackend::simulation_manager::LiveStats),
        (status = 404, description = "Simulation not found", body = String),
    )
)]
#[get("/simulation/{id}/live-stats")]
async fn get_live_stats(
    sim_manager: web::Data<SimulationManager>,
    path: web::Path<String>
) -> impl Responder {
    let simulation_id = path.into_inner();
    match simulation_manager::get_live_stats_legacy(&sim_manager, simulation_id) {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(e) => HttpResponse::NotFound().body(e),
    }
}

#[utoipa::path(
    get,
    path = "/simulation/{id}/scoreboard",
    responses(
        (status = 200, description = "Match scoreboard", body = VCTCareerBackend::simulation_manager::Scoreboard),
        (status = 404, description = "Simulation not found", body = String),
    )
)]
#[get("/simulation/{id}/scoreboard")]
async fn get_scoreboard(
    sim_manager: web::Data<SimulationManager>,
    path: web::Path<String>
) -> impl Responder {
    let simulation_id = path.into_inner();
    match simulation_manager::get_scoreboard_legacy(&sim_manager, simulation_id) {
        Ok(scoreboard) => HttpResponse::Ok().json(scoreboard),
        Err(e) => HttpResponse::NotFound().body(e),
    }
}

#[utoipa::path(
    get,
    path = "/simulation/{id}/economy",
    responses(
        (status = 200, description = "Economy status", body = VCTCareerBackend::simulation_manager::EconomyStatus),
        (status = 404, description = "Simulation not found", body = String),
    )
)]
#[get("/simulation/{id}/economy")]
async fn get_economy_status(
    sim_manager: web::Data<SimulationManager>,
    path: web::Path<String>
) -> impl Responder {
    let simulation_id = path.into_inner();
    match simulation_manager::get_economy_status_legacy(&sim_manager, simulation_id) {
        Ok(economy) => HttpResponse::Ok().json(economy),
        Err(e) => HttpResponse::NotFound().body(e),
    }
}

// Phase 3 API Endpoints
#[utoipa::path(
    post,
    path = "/simulation/{id}/checkpoint",
    responses(
        (status = 200, description = "Checkpoint created", body = String),
        (status = 404, description = "Simulation not found", body = String),
    )
)]
#[post("/simulation/{id}/checkpoint")]
async fn create_checkpoint(
    sim_manager: web::Data<SimulationManager>,
    path: web::Path<String>,
    description: Option<web::Json<String>>
) -> impl Responder {
    let simulation_id = path.into_inner();
    let desc = description.map(|d| d.into_inner());
    match simulation_manager::create_checkpoint_legacy(&sim_manager, simulation_id, desc) {
        Ok(checkpoint_id) => HttpResponse::Ok().json(checkpoint_id),
        Err(e) => HttpResponse::BadRequest().body(e),
    }
}

#[utoipa::path(
    get,
    path = "/simulation/{id}/events/at/{timestamp}",
    responses(
        (status = 200, description = "Events at timestamp", body = Vec<VCTCareerBackend::sim::GameEvent>),
        (status = 404, description = "Simulation not found", body = String),
    )
)]
#[get("/simulation/{id}/events/at/{timestamp}")]
async fn get_events_at_timestamp(
    sim_manager: web::Data<SimulationManager>,
    path: web::Path<(String, u64)>,
    query: web::Query<TimestampQuery>
) -> impl Responder {
    let (simulation_id, timestamp) = path.into_inner();
    let window = query.window_ms.unwrap_or(5000); // Default 5 second window
    
    match simulation_manager::get_events_at_timestamp_legacy(&sim_manager, simulation_id, timestamp, window) {
        Ok(events) => HttpResponse::Ok().json(events),
        Err(e) => HttpResponse::NotFound().body(e),
    }
}

#[derive(serde::Deserialize)]
struct TimestampQuery {
    window_ms: Option<u64>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    #[derive(OpenApi)]
    #[openapi(
        paths(
            create_career, 
            get_teams, 
            generate_offers, 
            estimate_rr, 
            random_map,
            create_simulation,
            get_simulation_state,
            advance_simulation,
            control_simulation,
            get_simulation_events,
            get_simulation_stats
        ),
        components(
            schemas(
                crate::models::CareerInfo,
                crate::models::Team,
                crate::models::CreateSimulationRequest,
                crate::models::CreateSimulationResponse,
                crate::models::SimulationPlayer,
                crate::models::AdvanceSimulationRequest,
                crate::models::SimulationControlRequest,
                crate::models::EventFilterRequest,
                crate::offers::OfferRequest,
                crate::offers::Offer,
            )
        ),
        info(
            title = "VCTCareer Backend API",
            version = "1.0.0",
            description = "Valorant Career Platform API with Actix-Web, Swagger UI, and Real-time Simulation Control"
        )
    )]
    struct ApiDoc;

    dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let config = Config::from_str(&db_url).expect("Invalid DATABASE_URL");
    let mgr = Manager::new(config, NoTls);
    let pool = Pool::builder(mgr).max_size(16).build().unwrap();
    let simulation_manager = create_simulation_manager();
    debug!("Starting server at http://127.0.0.1:8080");
    HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .wrap(cors)
            .service(create_career)
            .service(get_teams)
            .service(generate_offers)
            .service(estimate_rr)
            .service(random_map)
            .service(create_simulation)
            .service(get_simulation_state)
            .service(advance_simulation)
            .service(control_simulation)
            .service(get_simulation_events)
            .service(get_simulation_stats)
            // Phase 2 endpoints
            .service(get_live_stats)
            .service(get_scoreboard)
            .service(get_economy_status)
            // Phase 3 endpoints
            .service(create_checkpoint)
            .service(get_events_at_timestamp)
            .into_utoipa_app()
            .openapi(ApiDoc::openapi())
            .openapi_service(|api| {
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", api)
            })
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(simulation_manager.clone()))
            .into_app()
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
