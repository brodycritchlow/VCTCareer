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
use crate::models::CareerInfo;
use utoipa_swagger_ui::SwaggerUi;

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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    #[derive(OpenApi)]
    #[openapi(
        paths(create_career, get_teams, generate_offers),
        components(
            schemas(
                crate::models::CareerInfo,
                crate::models::Team,
                crate::offers::OfferRequest,
                crate::offers::Offer
            )
        ),
        info(
            title = "VCTCareer Backend API",
            version = "1.0.0",
            description = "Valorant Career Platform API with Actix-Web and Swagger UI"
        )
    )]
    struct ApiDoc;

    dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let config = Config::from_str(&db_url).expect("Invalid DATABASE_URL");
    let mgr = Manager::new(config, NoTls);
    let pool = Pool::builder(mgr).max_size(16).build().unwrap();
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
            .into_utoipa_app()
            .openapi(ApiDoc::openapi())
            .openapi_service(|api| {
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", api)
            })
            .app_data(web::Data::new(pool.clone()))
            .into_app()
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
