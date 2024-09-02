mod txt_generator;
mod domain_status;
pub use txt_generator::generate_txt_record;
mod record_verification;
pub use record_verification::verify_txt_record;
pub use domain_status::query_domain_status;

use actix_web::{web, App, HttpServer, middleware};
use sqlx::postgres::PgPoolOptions;
use dotenv::dotenv;
use std::env;
use actix_governor::{Governor, GovernorConfigBuilder};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    // Get database URL from environment variable
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Set up database connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool");

    // Run pending migrations
    sqlx::migrate!().run(&pool).await.expect("Failed to run migrations");
    

    println!("Starting server at http://127.0.0.1:8080");

    // rate limiter
    let governor_conf = GovernorConfigBuilder::default()
        .per_second(5)  
        .burst_size(10) 
        .finish()
        .unwrap();

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(Governor::new(&governor_conf))
            .app_data(web::Data::new(pool.clone()))
            .route("/generate_txt_record", web::post().to(generate_txt_record))
            .route("/verify_txt_record", web::post().to(verify_txt_record))
            .route("/domain_status", web::get().to(query_domain_status))
            .route("/", web::get().to(|| async { "Welcome to the DNS TXT Record Service" }))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}