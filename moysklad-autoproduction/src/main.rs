//! Автоматическое создание тех. операций при низких остатках товара
//!
//! Сервис отслеживает подтверждённые заказы покупателей и автоматически создаёт
//! тех. операции для пополнения остатков через производство.

use actix_web::{web, App, HttpServer};
use std::sync::Arc;
use tracing::info;

mod api;
mod config;
mod handlers;
mod models;
mod processing;

use config::Settings;
use handlers::AppState;
use processing::OrderProcessor;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Инициализация логирования
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_thread_ids(false)
        .pretty()
        .init();
    
    // Загрузка конфигурации
    dotenvy::dotenv().ok();
    let settings = Settings::from_env().expect("Failed to load settings");
    
    info!("Starting moysklad-autoproduction service");
    info!("Monitoring store: {}", settings.store_name);
    info!("Tech card field: {}", settings.tech_card_field_name);
    info!("Min stock threshold: {}", settings.min_stock_threshold);
    
    // Создаём состояние приложения
    let processor = OrderProcessor::new(settings.clone());
    let app_state = Arc::new(AppState {
        settings: settings.clone(),
        processor: tokio::sync::Mutex::new(processor),
    });
    
    let host = settings.server_host.clone();
    let port = settings.server_port;
    
    info!("Starting HTTP server on {}:{}", host, port);
    
    // Запуск HTTP сервера
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .route("/health", web::get().to(handlers::health))
            .route("/webhook", web::post().to(handlers::webhook))
            .route("/order/{id}/process", web::post().to(handlers::process_order))
            .route("/config", web::get().to(handlers::get_config))
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
