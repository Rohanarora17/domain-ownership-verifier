use actix_web::{web, HttpResponse, Responder};
use hickory_resolver::config::*;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use hickory_resolver::TokioAsyncResolver;
use tokio::runtime::Handle;

#[derive(Debug, Serialize, Deserialize)]
pub struct VerificationRequest {
    user_id: String,
    domain: String,
}

async fn lookup_txt_record(domain: &str) -> Result<Vec<String>, String> {
    // Create a resolver with the default configuration
    let resolver = TokioAsyncResolver::new(
        ResolverConfig::default(),
        ResolverOpts::default(),
        Handle::current()
    )
    .map_err(|e| format!("Failed to create resolver: {}", e))?;
    
    // Perform the lookup for TXT records
    let response = resolver.txt_lookup(domain).await
        .map_err(|e| format!("TXT record lookup failed: {}", e))?;
    
    // Extract and collect all the TXT records
    let records: Vec<String> = response.iter()
        .flat_map(|rdata| rdata.txt_data().iter().map(|b| String::from_utf8_lossy(b).to_string()))
        .collect();
    
    Ok(records)
}

// async fn lookup_txt_record(domain: &str) -> Result<Vec<String>, String> {
//     // Create a resolver with the default configuration
//     let resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default())
//         .map_err(|e| format!("Failed to create resolver: {}", e))?;
    
//     // Perform the lookup for TXT records
//     let response = resolver.txt_lookup(domain)
//         .map_err(|e| format!("TXT record lookup failed: {}", e))?;
    
//     // Extract and collect all the TXT records
//     let records: Vec<String> = response.iter()
//         .flat_map(|rdata| rdata.txt_data().iter().map(|b| String::from_utf8_lossy(b).to_string()))
//         .collect();
    
//     Ok(records)
// }



pub async fn verify_txt_record(
    verification_request: web::Json<VerificationRequest>,
    db_pool: web::Data<Pool<Postgres>>,
) -> impl Responder {
    let domain = &verification_request.domain;
    let user_id = &verification_request.user_id;

    // Fetch the expected TXT record from the database
    let expected_record = match sqlx::query!(
        "SELECT record FROM txt_records WHERE user_id = $1 AND domain = $2 AND is_verified = false",
        user_id,
        domain
    )
    .fetch_one(db_pool.get_ref())
    .await
    {
        Ok(record) => record.record,
        Err(_) => return HttpResponse::NotFound().json("Record not found or already verified"),
    };

    // Lookup the TXT records for the domain
    match lookup_txt_record(domain).await {
        Ok(txt_records) => {
            if txt_records.contains(&expected_record) {
                // Update the database to mark the record as verified
                match sqlx::query!(
                    "UPDATE txt_records SET is_verified = true WHERE user_id = $1 AND domain = $2",
                    user_id,
                    domain
                )
                .execute(db_pool.get_ref())
                .await
                {
                    Ok(_) => HttpResponse::Ok().json("TXT record verified successfully"),
                    Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
                }
            } else {
                HttpResponse::BadRequest().json("TXT record not found")
            }
        }
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}
