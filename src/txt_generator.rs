use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use ksuid::Ksuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct TxtRecordGenerator {
    domain: String,
    record_attribute: String,
    record_attribute_value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DnsRecordInstruction {
    pub domain: String,
    pub record: String,
    pub action: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserRequest {
    user_id: String,
    domain: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TxtRecordResponse {
    user_id: String,
    dns_record: DnsRecordInstruction,
}

impl TxtRecordGenerator {
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

fn generate_ksuid() -> String {
    Ksuid::generate().to_base62()
}

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

pub async fn generate_txt_record(
    user_request: web::Json<UserRequest>,
    db_pool: web::Data<Pool<Postgres>>,
) -> impl Responder {
    let domain = &user_request.domain;
    let user_id = &user_request.user_id;

    if domain.trim().is_empty() {
        return HttpResponse::BadRequest().json("Domain is empty");
    }

    let txt_record_attribute_suffix = "_verification";

    let mut txt_config = TxtRecordGenerator {
        domain: domain.to_string(),
        record_attribute: format!("{}_{}", domain.replace(".", "_"), txt_record_attribute_suffix),
        record_attribute_value: generate_ksuid(),
    };

    match generate_txt_record_from_config(&mut txt_config) {
        Ok(dns_record) => {
            // Store the record in the database
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


