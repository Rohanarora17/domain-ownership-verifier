//! Module for DNS TXT record verification.
//!
//! This module provides functionality to verify DNS TXT records for domain ownership verification.

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use std::net::*;
use async_std::prelude::*;
use async_std_resolver::{resolver, config};

/// Represents a request to verify a TXT record.
#[derive(Debug, Serialize, Deserialize)]
pub struct VerificationRequest {
    /// The ID of the user requesting verification.
    user_id: String,
    /// The domain to verify.
    domain: String,
}

/// Looks up TXT records for a given domain.
///
/// # Arguments
///
/// * `domain` - A string slice that holds the domain name to look up.
///
/// # Returns
///
/// A `Result` containing a vector of strings (TXT records) if successful,
/// or an error message as a string if the lookup fails.
async fn lookup_txt_record(domain: &str) -> Result<Vec<String>, String> {
    let resolver = resolver(
        config::ResolverConfig::default(),
        config::ResolverOpts::default(),
    ).await;
    
    // Perform TXT lookup
    let response = resolver.txt_lookup(domain).await
        .map_err(|e| format!("TXT record lookup failed: {}", e))?;
    
    // Extract and collect all the TXT records
    let records: Vec<String> = response.iter()
        .flat_map(|rdata| rdata.txt_data().iter().map(|b| String::from_utf8_lossy(b).to_string()))
        .collect();
    
    Ok(records)
}

/// Verifies a TXT record for domain ownership.
///
/// This function checks if the expected TXT record exists for the given domain.
/// If found, it marks the record as verified in the database.
///
/// # Arguments
///
/// * `verification_request` - A JSON payload containing the user ID and domain to verify.
/// * `db_pool` - A connection pool for the database.
///
/// # Returns
///
/// An implementation of `Responder`, which will be a JSON response indicating
/// the result of the verification process.
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
    .fetch_optional(db_pool.get_ref())
    .await
    {
        Ok(Some(record)) => record.record,
        Ok(None) => return HttpResponse::NotFound().json("Record not found or already verified"),
        Err(e) => return HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
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
