use crate::models::CareerInfo;
use crate::models::StartingTier;
use actix_web::{web, HttpResponse, Responder};
use dotenv::dotenv;
use tokio_postgres::NoTls;
use serde::Deserialize;

pub fn weighted_tier(info: &CareerInfo) -> StartingTier {
    let mut score = 0.0;
    let rank_score: f32 = match info.current_rank.as_str() {
        "Radiant" => 5.0,
        "Immortal" => 4.0,
        "Ascendant" => 2.0,
        "Diamond" => 1.0,
        "Platinum" => 0.5,
        _ => 0.0,
    };
    score += rank_score;
    let exp_score = match info.past_experience.as_str() {
        "Tier 1" => 4.0,
        "Tier 2" => 3.0,
        "Tier 3" => 2.0,
        _ => 0.0,
    };
    if rank_score >= 2.0 {
        score += exp_score;
    } else if exp_score > 0.0 {
        score += 1.0;
    }
    match info.division.to_lowercase().as_str() {
        "na" | "eu" => score += 1.0,
        _ => {}
    }
    match score {
        s if s >= 9.0 => StartingTier::Tier1,
        s if s >= 6.0 => StartingTier::Tier2,
        s if s >= 3.0 => {
            if (17..=24).contains(&info.age) {
                StartingTier::Tier3
            } else {
                StartingTier::RankedPlay
            }
        },
        _ => StartingTier::RankedPlay,
    }
}

#[derive(Debug, Deserialize)]
pub struct TeamQuery {
    pub team_name: Option<String>,
    pub ranking: Option<i32>,
    pub tier: Option<String>,
    pub region: Option<String>,
}

pub async fn get_teams_handler(query: web::Query<TeamQuery>) -> impl Responder {
    dotenv().ok();
    let (client, connection) = match tokio_postgres::connect(&std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"), NoTls).await {
        Ok((client, connection)) => (client, connection),
        Err(e) => {
            eprintln!("Failed to connect to database: {}", e);
            return HttpResponse::InternalServerError().body(format!("Failed to connect to database: {}", e));
        }
    };
    actix_web::rt::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    let mut query_str = String::from("SELECT * FROM teams");
    let mut conditions = Vec::new();
    let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();
    let mut ranking_val: Option<i32> = None;
    let mut tier_val: Option<i16> = None;
    if let Some(ref name) = query.team_name {
        conditions.push(format!("team_name = ${}", params.len() + 1));
        params.push(name);
    }
    if let Some(ranking) = query.ranking {
        ranking_val = Some(ranking);
        conditions.push(format!("ranking = ${}", params.len() + 1));
        params.push(ranking_val.as_ref().unwrap());
    }
    if let Some(ref tier) = query.tier {
        tier_val = Some(match tier.as_str() {
            "Tier 1" => 1,
            "Tier 2" => 2,
            "Tier 3" => 3,
            _ => tier.parse().unwrap_or(0),
        });
        conditions.push(format!("tier = ${}", params.len() + 1));
        params.push(tier_val.as_ref().unwrap());
    }
    if let Some(ref region) = query.region {
        conditions.push(format!("region = ${}", params.len() + 1));
        params.push(region);
    }
    if !conditions.is_empty() {
        query_str.push_str(&format!(" WHERE {}", conditions.join(" AND ")));
    }
    let rows = match client.query(&query_str, &params).await {
        Ok(rows) => rows,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to fetch from database"),
    };
    let teams: Vec<serde_json::Value> = rows.iter().map(|row| {
        let mut obj = serde_json::Map::new();
        for (i, col) in row.columns().iter().enumerate() {
            let col_name = col.name();
            let value: serde_json::Value = match col.type_().name() {
                "int2" => row.try_get::<usize, i16>(i).map(serde_json::to_value).unwrap_or(Ok(serde_json::Value::Null)).unwrap_or(serde_json::Value::Null),
                "int4" => row.try_get::<usize, i32>(i).map(serde_json::to_value).unwrap_or(Ok(serde_json::Value::Null)).unwrap_or(serde_json::Value::Null),
                "int8" => row.try_get::<usize, i64>(i).map(serde_json::to_value).unwrap_or(Ok(serde_json::Value::Null)).unwrap_or(serde_json::Value::Null),
                "float4" => row.try_get::<usize, f32>(i).map(serde_json::to_value).unwrap_or(Ok(serde_json::Value::Null)).unwrap_or(serde_json::Value::Null),
                "float8" => row.try_get::<usize, f64>(i).map(serde_json::to_value).unwrap_or(Ok(serde_json::Value::Null)).unwrap_or(serde_json::Value::Null),
                "bool" => row.try_get::<usize, bool>(i).map(serde_json::to_value).unwrap_or(Ok(serde_json::Value::Null)).unwrap_or(serde_json::Value::Null),
                _ => row.try_get::<usize, String>(i).map(serde_json::to_value).unwrap_or(Ok(serde_json::Value::Null)).unwrap_or(serde_json::Value::Null),
            };
            obj.insert(col_name.to_string(), value);
        }
        serde_json::Value::Object(obj)
    }).collect();
    HttpResponse::Ok().json(teams)
}

