use actix_web::{web, App, HttpServer, HttpResponse, Result, middleware::Logger};
use serde_json::json;
use core_sdk::{Problem, Contest};

async fn health() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "status": "ok",
        "service": "judicia-backend"
    })))
}

async fn get_problems() -> Result<HttpResponse> {
    let problems = vec![
        Problem {
            id: "1".to_string(),
            title: "A + B".to_string(),
            description: "Calculate the sum of two numbers".to_string(),
        }
    ];
    Ok(HttpResponse::Ok().json(problems))
}

async fn get_contests() -> Result<HttpResponse> {
    let contests = vec![
        Contest {
            id: "1".to_string(),
            name: "Sample Contest".to_string(),
            contest_type: "IOI".to_string(),
        }
    ];
    Ok(HttpResponse::Ok().json(contests))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    println!("Starting Judicia Backend on http://localhost:8080");
    
    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .route("/health", web::get().to(health))
            .route("/api/problems", web::get().to(get_problems))
            .route("/api/contests", web::get().to(get_contests))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
