//! Module for generating DNS TXT records for domain ownership verification.
//!
//! This module provides functionality to generate and manage TXT records
//! for domain ownership verification purposes.

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use ksuid::Ksuid;

/// Represents the configuration for generating a TXT record.
#[derive(Debug, Serialize, Deserialize)]
pub struct TxtRecordGenerator {
    /// The domain for which the TXT record is being generated.
    domain: String,
    /// The attribute name for the TXT record.
    record_attribute: String,
    /// The value for the TXT record attribute.
    record_attribute_value: String,
}

/// Represents the instruction for creating a DNS record.
#[derive(Debug, Serialize, Deserialize)]
pub struct DnsRecordInstruction {
    /// The domain for which the DNS record is to be created.
    pub domain: String,
    /// The content of the DNS record.
    pub record: String,
    /// A human-readable description of the action to be taken.
    pub action: String,
}

/// Represents a user's request for generating a TXT record.
#[derive(Debug, Serialize, Deserialize)]      
pub struct UserRequest {
    /// The ID of the user making the request.
    user_id: String,
    /// The domain for which the TXT record is requested.
    domain: String,
}

/// Represents the response to a TXT record generation request.
#[derive(Debug, Serialize, Deserialize)]
pub struct TxtRecordResponse {
    /// The ID of the user who made the request.
    user_id: String,
    /// The DNS record instruction generated.
    dns_record: DnsRecordInstruction,
}

impl TxtRecordGenerator {
    /// Validates the TXT record configuration.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the configuration is valid, or an `Err` with a description of the error.
    fn validate(&self) -> Result<(), String> {
        if self.domain.trim().is_empty() {
            return Err("Domain is empty".to_string());
        }

        if self.record_attribute.trim().is_empty() {
            return Err("Record attribute is empty".to_string());
        }

        if self.record_attribute_value.trim().is_empty() {
            return Err("Record attribute value is empty".to_string());
        }

        Ok(())
    }
}

/// Generates a new KSUID (K-Sortable Unique IDentifier).
///
/// # Returns
///
/// Returns a String representation of the generated KSUID.
fn generate_ksuid() -> String {
    Ksuid::generate().to_base62()
}

/// Generates a DNS record instruction from the provided configuration.
///
/// # Arguments
///
/// * `config` - A mutable reference to a `TxtRecordGenerator` configuration.
///
/// # Returns
///
/// Returns a `Result` containing either a `DnsRecordInstruction` if successful,
/// or a `String` describing the error if unsuccessful.
fn generate_txt_record_from_config(
    config: &mut TxtRecordGenerator,
) -> Result<DnsRecordInstruction, String> {
    config.validate()?;

    let txt_record = format!("{}={}", config.record_attribute, config.record_attribute_value);

    let instruction = DnsRecordInstruction {
        domain: config.domain.clone(),
        record: txt_record.clone(),
        action: format!(
            "Create a TXT record for the domain {} with the content {}",
            config.domain, txt_record
        ),
    };

    Ok(instruction)
}

/// Handles the generation of a TXT record for domain ownership verification.
///
/// This function checks if a verification record already exists for the given user and domain.
/// If it does and is verified, it returns a message indicating that.
/// If it exists but is not verified, it returns the existing record.
/// If no record exists, it generates a new one and stores it in the database.
///
/// # Arguments
///
/// * `user_request` - A JSON payload containing the user's request.
/// * `db_pool` - A connection pool for the database.
///
/// # Returns
///
/// Returns an implementation of `Responder`, which will be a JSON response
/// containing either the generated TXT record or an error message.
pub async fn generate_txt_record(
    user_request: web::Json<UserRequest>,
    db_pool: web::Data<Pool<Postgres>>,
) -> impl Responder {
    let domain = &user_request.domain;
    let user_id = &user_request.user_id;

    if domain.trim().is_empty() {
        return HttpResponse::BadRequest().json("Domain is empty");
    }

    // Check if a verification code already exists for this user_id and domain
    match sqlx::query!(
        "SELECT record, is_verified FROM txt_records WHERE user_id = $1 AND domain = $2",
        user_id,
        domain
    )
    .fetch_optional(db_pool.get_ref())
    .await
    {
        Ok(Some(existing_record)) => {
            if existing_record.is_verified {
                // If the user is already verified, return a message
                return HttpResponse::Ok().json("This domain onwnership is already verified");
            } else {
                // If a record exists but not verified, return the existing verification code
                let dns_record = DnsRecordInstruction {
                    domain: domain.to_string(),
                    record: existing_record.record.clone(),
                    action: format!(
                        "Use existing TXT record for the domain {} with the content {}",
                        domain, existing_record.record.clone()
                    ),
                };
                let response = TxtRecordResponse {
                    user_id: user_id.to_string(),
                    dns_record,
                };
                return HttpResponse::Ok().json(response);
            }
        }
        Ok(None) => {
            // If no record exists, proceed to generate a new one
            println!("No record exists, proceeding to generate a new one");
            let txt_record_attribute_suffix = "_verification";

            let mut txt_config = TxtRecordGenerator {
                domain: domain.to_string(),
                record_attribute: format!("{}_{}", domain.replace(".", "_"), txt_record_attribute_suffix),
                record_attribute_value: generate_ksuid(),
            };

            match generate_txt_record_from_config(&mut txt_config) {
                Ok(dns_record) => {
                    // Store the new record in the database
                    match sqlx::query!(
                        "INSERT INTO txt_records (user_id, domain, record, is_verified) VALUES ($1, $2, $3, $4)",
                        user_id,
                        dns_record.domain,
                        dns_record.record,
                        false
                    )
                    .execute(db_pool.get_ref())
                    .await
                    {
                        Ok(_) => {
                            let response = TxtRecordResponse {
                                user_id: user_id.to_string(),
                                dns_record,
                            };
                            HttpResponse::Ok().json(response)
                        }
                        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
                    }
                }
                Err(e) => HttpResponse::InternalServerError().body(e),
            }
            

        }
        Err(e) => return HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }

    
}
