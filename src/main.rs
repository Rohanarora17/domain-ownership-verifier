mod txt_generator;
mod domain_status;
mod record_verification;

pub use txt_generator::generate_txt_record;
pub use domain_status::query_domain_status;
pub use record_verification::verify_txt_record;

use actix_web::{web, App, HttpServer, middleware};
use sqlx::postgres::PgPoolOptions;
use dotenv::dotenv;
use std::env;
use actix_governor::{Governor, GovernorConfigBuilder};





/// The main function that sets up and runs the web server.
///
/// This function performs the following tasks:
/// 1. Loads environment variables from a .env file
/// 2. Sets up a database connection pool
/// 3. Runs any pending database migrations
/// 4. Configures and starts the web server with rate limiting and defined routes
///
/// # Returns
///
/// Returns a Result which is Ok if the server runs successfully, or an Err
/// if there's an error during setup or execution.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables from .env file
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

    // Configure rate limiter
    let governor_conf = GovernorConfigBuilder::default()
        .per_second(5)  // Allow 5 requests per second
        .burst_size(10) // Allow bursts of up to 10 requests
        .finish()
        .unwrap();


    // Set up and run the HTTP server
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
