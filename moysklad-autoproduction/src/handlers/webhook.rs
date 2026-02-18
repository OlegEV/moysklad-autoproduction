//! HTTP request handlers

use actix_web::{web, HttpResponse, Responder};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

use crate::config::Settings;
use crate::models::WebhookEvent;
use crate::processing::DemandProcessor;

/// Application state
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

/// Query parameters for Moysklad webhook
#[derive(Debug, serde::Deserialize)]
pub struct WebhookQuery {
    /// Entity ID (e.g., demand ID)
    pub id: String,
    /// Entity type (e.g., "Demand", "Supply", etc.)
    #[serde(rename = "type")]
    pub entity_type: String,
}

/// Webhook endpoint for receiving events from Moysklad
/// Moysklad sends: POST /webhook?id={id}&type={type}
/// Example: POST /webhook?id=e74614f8-0c05-11f1-0a80-0f27004c4df2&type=Demand
pub async fn webhook(
    state: web::Data<Arc<AppState>>,
    query: web::Query<WebhookQuery>,
) -> impl Responder {
    let id = &query.id;
    let entity_type = &query.entity_type;
    
    info!(
        "Received webhook: id={}, type={}",
        id, entity_type
    );
    
    // Normalize entity type to lowercase for comparison
    let entity_type_lower = entity_type.to_lowercase();
    
    // Process only demand (shipment) events
    if entity_type_lower != "demand" {
        info!("Ignoring non-demand event (type={})", entity_type);
        return HttpResponse::Ok().json(serde_json::json!({
            "status": "ignored",
            "message": format!("Not a demand event (type={})", entity_type)
        }));
    }
    
    // Create webhook event from query parameters
    let event = WebhookEvent {
        meta: None,
        id: None,
        name: None,
        account_id: String::new(),
        entity_type: entity_type_lower.clone(),
        action: "update".to_string(),
        entity: None,
        content: Some(crate::models::WebhookContent {
            entity: None,
            id: Some(id.clone()),
            entity_type: Some(entity_type_lower),
        }),
    };
    
    // Get processor and handle the event
    let mut processor = state.processor.lock().await;
    
    match processor.process_webhook(&event).await {
        Ok(results) => {
            let success_count = results.iter().filter(|r| r.success).count();
            let total_count = results.len();
            
            info!(
                "Processed demand {}: {} of {} positions successful",
                id, success_count, total_count
            );
            
            HttpResponse::Ok().json(serde_json::json!({
                "status": "processed",
                "demand_id": id,
                "results": results
            }))
        }
        Err(e) => {
            error!("Error processing webhook for demand {}: {}", id, e);
            
            HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "demand_id": id,
                "message": e.to_string()
            }))
        }
    }
}

/// Endpoint for manual demand processing by ID
pub async fn process_demand(
    state: web::Data<Arc<AppState>>,
    path: web::Path<String>,
) -> impl Responder {
    let demand_id = path.into_inner();
    
    info!("Manual processing request for demand: {}", demand_id);
    
    // Create webhook event
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
            id: Some(demand_id.clone()),
            entity_type: Some("demand".to_string()),
        }),
    };
    
    let mut processor = state.processor.lock().await;
    
    match processor.process_webhook(&event).await {
        Ok(results) => {
            HttpResponse::Ok().json(serde_json::json!({
                "status": "processed",
                "demand_id": demand_id,
                "results": results
            }))
        }
        Err(e) => {
            error!("Error processing demand {}: {}", demand_id, e);
            
            HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "demand_id": demand_id,
                "message": e.to_string()
            }))
        }
    }
}

/// Get current configuration
pub async fn get_config(state: web::Data<Arc<AppState>>) -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "store_name": state.settings.store_name,
        "tech_card_field_name": state.settings.tech_card_field_name,
        "min_stock_threshold": state.settings.min_stock_threshold,
    }))
}
