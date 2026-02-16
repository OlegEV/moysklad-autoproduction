//! Обработчики HTTP запросов

use actix_web::{web, HttpResponse, Responder};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};

use crate::config::Settings;
use crate::models::WebhookEvent;
use crate::processing::DemandProcessor;

/// Состояние приложения
pub struct AppState {
    pub settings: Settings,
    pub processor: Mutex<DemandProcessor>,
}

/// Health check endpoint
pub async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "moysklad-autoproduction"
    }))
}

/// Webhook endpoint для приёма событий от МойСклад
pub async fn webhook(
    state: web::Data<Arc<AppState>>,
    body: web::Json<WebhookEvent>,
) -> impl Responder {
    let event = body.into_inner();
    
    info!(
        "Received webhook: type={}, action={}",
        event.entity_type, event.action
    );
    
    // Обрабатываем только события создания/изменения отгрузок
    if event.entity_type != "demand" {
        info!("Ignoring non-demand event");
        return HttpResponse::Ok().json(serde_json::json!({
            "status": "ignored",
            "message": "Not a demand event"
        }));
    }
    
    // Получаем процессор и обрабатываем событие
    let mut processor = state.processor.lock().await;
    
    match processor.process_webhook(&event).await {
        Ok(results) => {
            let success_count = results.iter().filter(|r| r.success).count();
            let total_count = results.len();
            
            info!(
                "Processed demand: {} of {} positions successful",
                success_count, total_count
            );
            
            HttpResponse::Ok().json(serde_json::json!({
                "status": "processed",
                "results": results
            }))
        }
        Err(e) => {
            error!("Error processing webhook: {}", e);
            
            HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": e.to_string()
            }))
        }
    }
}

/// Endpoint для ручной обработки отгрузки по ID
pub async fn process_demand(
    state: web::Data<Arc<AppState>>,
    path: web::Path<String>,
) -> impl Responder {
    let demand_id = path.into_inner();
    
    info!("Manual processing request for demand: {}", demand_id);
    
    // Создаём фейковое webhook событие
    let event = WebhookEvent {
        meta: None,
        id: None,
        name: None,
        account_id: String::new(),
        entity_type: "demand".to_string(),
        action: "update".to_string(),
        entity: None,
        content: Some(crate::models::WebhookContent {
            entity: None,
            id: Some(demand_id),
            entity_type: Some("demand".to_string()),
        }),
    };
    
    let mut processor = state.processor.lock().await;
    
    match processor.process_webhook(&event).await {
        Ok(results) => {
            HttpResponse::Ok().json(serde_json::json!({
                "status": "processed",
                "results": results
            }))
        }
        Err(e) => {
            error!("Error processing demand: {}", e);
            
            HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": e.to_string()
            }))
        }
    }
}

/// Получить текущую конфигурацию
pub async fn get_config(state: web::Data<Arc<AppState>>) -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "store_name": state.settings.store_name,
        "tech_card_field_name": state.settings.tech_card_field_name,
        "min_stock_threshold": state.settings.min_stock_threshold,
    }))
}
