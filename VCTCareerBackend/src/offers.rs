use actix_web::HttpResponse;
use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct OfferRequest {
    pub tier: String,
    pub count: usize,
    pub region: Option<String>,
    pub overall: Option<u32>,
}

#[derive(Serialize, ToSchema)]
pub struct Offer {
    pub team: String,
    pub contract_length_months: u32,
    pub yearlysalary: u32,
    pub region: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct RegionSalaryInfo {
    pub min: i32,
    pub max: i32,
    pub plus_minus: i32,
}

impl RegionSalaryInfo {
    pub fn calculate_expected_salary(
        overall_rating: f64,
        overall_min: f64,
        overall_max: f64,
        region_min_salary: f64,
        region_max_salary: f64,
    ) -> f64 {
        if overall_max == overall_min {
            println!("Warning: overall_max cannot be equal to overall_min. Returning 0.0.");
            return 0.0;
        }

        let relative_overall_position =
            (overall_rating - overall_min) / (overall_max - overall_min);

        let regional_salary_range = region_max_salary - region_min_salary;
        let salary_increment = relative_overall_position * regional_salary_range;
        let base_salary = region_min_salary + salary_increment;

        base_salary.clamp(region_min_salary, f64::MAX)
    }
}

pub async fn generate_offers(
    pool: &deadpool_postgres::Pool,
    req: &OfferRequest,
) -> Result<Vec<Offer>, String> {
    let client = pool
        .get()
        .await
        .map_err(|e| format!("Failed to get DB client: {}", e))?;
    let mut query_str = String::from("SELECT team_name, tier, region, budget, expenses FROM teams");
    let mut conditions = Vec::new();
    let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();
    let mut owned_tier_vals: Vec<i16> = Vec::new();

    if let Some(ref region) = req.region {
        conditions.push(format!("region = ${}", params.len() + 1));
        params.push(region);
    }
    if !req.tier.is_empty() {
        let tier_val = match req.tier.as_str() {
            "1" => 1i16,
            "2" => 2i16,
            "3" => 3i16,
            _ => req.tier.parse().unwrap_or(0),
        };
        owned_tier_vals.push(tier_val);
        conditions.push(format!("tier = ${}", params.len() + 1));
        params.push(owned_tier_vals.last().unwrap());
    }

    if !conditions.is_empty() {
        query_str.push_str(&format!(" WHERE {}", conditions.join(" AND ")));
    }
    let rows = client
        .query(&query_str, &params)
        .await
        .map_err(|e| format!("DB query error: {}", e))?;

    if rows.is_empty() {
        return Err("No teams found for the given criteria".to_string());
    }

    let json_str = fs::read_to_string("src/region_offer_extrema.json")
        .map_err(|e| format!("Failed to read JSON file: {}", e))?;

    let region_offer_extrema: HashMap<String, RegionSalaryInfo> =
        serde_json::from_str(&json_str).map_err(|e| format!("Failed to parse JSON: {}", e))?;

    let mut offers = Vec::new();

    let mut expected_salaries: HashMap<String, f64> = HashMap::new();
    for (region, info) in &region_offer_extrema {
        let salary = RegionSalaryInfo::calculate_expected_salary(
            req.overall.unwrap_or(50) as f64,
            0.0,
            100.0,
            info.min as f64,
            info.max as f64,
        );
        expected_salaries.insert(region.clone(), salary);
    }

    let mut rng = rand::thread_rng();
    for row in rows {
        let team: String = row.get("team_name");
        let region: String = row.get("region");
        let budget: i64 = row.get::<_, Option<i64>>("budget").unwrap_or(0);
        let expenses: i64 = row.get::<_, Option<i64>>("expenses").unwrap_or(0);

        if let Some(expected_salary) = expected_salaries.get(&region) {
            let contract_length_months = rng.gen_range(1..=3) * 12 as u32;

            let pm_factor = region_offer_extrema.get(region.as_str()).unwrap().plus_minus;
            let variance = rng.gen_range(-pm_factor..=pm_factor);
            let yearly_salary = (*expected_salary + variance as f64) as u32;

            if yearly_salary <= budget as u32 - expenses as u32 {
                offers.push(Offer {
                    team,
                    contract_length_months,
                    yearlysalary: yearly_salary,
                    region,
                });
            }
        }
    }
    offers.shuffle(&mut rng);
    if offers.is_empty() {
        return Err("No valid offers could be generated based on the criteria".to_string());
    }
    if req.count > 0 && offers.len() > req.count {
        offers.truncate(req.count);
    }
    println!("Generated {} offers", offers.len());
    Ok(offers)
}
