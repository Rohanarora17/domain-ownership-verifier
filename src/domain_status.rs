//! Domain status module for querying the verification status of a domain.

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
/// Represents a query for domain status.
#[derive(Debug, Serialize, Deserialize)]
pub struct DomainStatusQuery {
    /// The ID of the user querying the domain status.
    pub user_id: String,
    /// The domain being queried.
    pub domain: String,
}

/// Represents the response for a domain status query.
#[derive(Debug, Serialize, Deserialize)]
pub struct DomainStatusResponse {
    /// The status of the domain ownership verification.
    status: String
}

/// Queries the status of domain ownership verification.
///
/// This function checks the database for the verification status of a given domain
/// for a specific user. It returns a JSON response indicating whether the domain
/// ownership is verified, pending verification, or not found.
///
/// # Arguments
///
/// * `query` - A web::Query containing the user ID and domain to check.
/// * `db_pool` - A connection pool for the database.
///
/// # Returns
///
/// An implementation of Responder, which will be a JSON response containing
/// the status of the domain ownership verification.
pub async fn query_domain_status(
    query: web::Query<DomainStatusQuery>,
    db_pool: web::Data<Pool<Postgres>>,
) -> impl Responder {
    let is_verified = match sqlx::query!(
        "SELECT is_verified FROM txt_records WHERE user_id = $1 AND domain = $2",
        query.user_id,
        query.domain
    )
    .fetch_optional(db_pool.get_ref())
    .await
    {
        Ok(Some(record)) => record.is_verified,
        Ok(None) => return HttpResponse::NotFound().json(DomainStatusResponse {
            status: "Record not found".to_string()
        }),
        Err(e) => return HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    };

    if is_verified {
        HttpResponse::Ok().json(DomainStatusResponse {
            status: "Domain ownership verified".to_string()
        })
    } else {
        HttpResponse::Ok().json(DomainStatusResponse {
            status: "Record found but not yet verified".to_string()
        })
    }
}
