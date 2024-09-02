mod txt_generator;

pub use txt_generator::generate_txt_record;
mod record_verification;
pub use record_verification::verify_txt_record;

use actix_web::{web, App, HttpServer};
use sqlx::postgres::PgPoolOptions;
use dotenv::dotenv;
use std::env;

// fn main() {
//     match generate_txt_record("0xday.tech") {
//         Ok(instruction) => println!("Generated instruction: {:?}", instruction),
//         Err(e) => eprintln!("Error: {}", e),
//     }
// }






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

    // Log that the server is starting
    println!("Starting server at http://127.0.0.1:8080");

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/generate_txt_record", web::post().to(generate_txt_record))
            .route("/verify_txt_record", web::post().to(verify_txt_record))
            .route("/", web::get().to(|| async { "Welcome to the DNS TXT Record Service" }))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}