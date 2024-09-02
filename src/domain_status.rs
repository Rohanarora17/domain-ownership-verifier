use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

#[derive(Debug, Serialize, Deserialize)]
pub struct DomainStatusQuery {
    pub user_id: String,
    pub domain: String,
}

#[derive(Debug, Serialize, Deserialize)]

pub struct DomainStatusResponse {
    status: String
}

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
