//! Клиент API МойСклад

use crate::models::*;
use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use tracing::{debug, info, warn};

const MOYSKLAD_API_BASE: &str = "https://api.moysklad.ru/api/remap/1.2";

/// Клиент API МойСклад
pub struct MoyskladClient {
    client: Client,
    token: String,
}

impl MoyskladClient {
    /// Создать новый клиент
    pub fn new(token: String) -> Self {
        let client = Client::builder()
            .gzip(true)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        
        Self { client, token }
    }

    /// Выполнить GET запрос к API
    async fn get<T: serde::de::DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
        let url = if endpoint.starts_with("http") {
            endpoint.to_string()
        } else {
            format!("{}{}", MOYSKLAD_API_BASE, endpoint)
        };
        
        debug!("GET request to: {}", url);
        
        let response = self.client
            .get(&url)
            .bearer_auth(&self.token)
            .header("Accept-Encoding", "gzip")
            .send()
            .await
            .context("Failed to send request")?;
        
        let status = response.status();
        let body = response.text().await.context("Failed to read response body")?;
        
        if !status.is_success() {
            warn!("API error response: {} - {}", status, body);
            return Err(anyhow!("API error {}: {}", status, body));
        }
        
        debug!("Response body (first 1000 chars): {}", &body[..body.len().min(1000)]);
        
        serde_json::from_str(&body).with_context(|| format!("Failed to parse response from {}: {}", url, &body[..body.len().min(500)]))
    }

    /// Выполнить POST запрос к API
    async fn post<T: serde::de::DeserializeOwned, B: serde::Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}{}", MOYSKLAD_API_BASE, endpoint);
        
        debug!("POST request to: {}", url);
        
        let response = self.client
            .post(&url)
            .bearer_auth(&self.token)
            .header("Accept-Encoding", "gzip")
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await
            .context("Failed to send request")?;
        
        let status = response.status();
        let response_body = response.text().await.context("Failed to read response body")?;
        
        if !status.is_success() {
            warn!("API error response: {} - {}", status, response_body);
            return Err(anyhow!("API error {}: {}", status, response_body));
        }
        
        serde_json::from_str(&response_body).context("Failed to parse response")
    }

    /// Выполнить PUT запрос к API
    async fn put<T: serde::de::DeserializeOwned, B: serde::Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}{}", MOYSKLAD_API_BASE, endpoint);
        
        debug!("PUT request to: {}", url);
        
        let response = self.client
            .put(&url)
            .bearer_auth(&self.token)
            .header("Accept-Encoding", "gzip")
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await
            .context("Failed to send request")?;
        
        let status = response.status();
        let response_body = response.text().await.context("Failed to read response body")?;
        
        if !status.is_success() {
            warn!("API error response: {} - {}", status, response_body);
            return Err(anyhow!("API error {}: {}", status, response_body));
        }
        
        serde_json::from_str(&response_body).context("Failed to parse response")
    }

    /// Найти склад по названию
    pub async fn find_store_by_name(&self, name: &str) -> Result<Option<EntityRef>> {
        info!("Searching for store: {}", name);
        
        let response: ApiResponse<EntityRef> = self
            .get(&format!("/entity/store?filter=name={}", urlencoding::encode(name)))
            .await?;
        
        Ok(response.rows.and_then(|mut rows| rows.pop()))
    }

    /// Получить остаток конкретного товара на складе
    pub async fn get_product_stock(&self, product_id: &str, store_id: &str) -> Result<f64> {
        debug!("Getting stock for product {} on store {}", product_id, store_id);
        
        // Получаем все остатки и фильтруем по product_id и store_id
        let response: ApiResponse<StockByStoreRow> = self
            .get("/report/stock/bystore?limit=1000")
            .await?;
        
        if let Some(rows) = response.rows {
            for row in rows {
                // Извлекаем ID продукта из meta.href
                let row_product_id = row.meta.href
                    .rsplit('/')
                    .next()
                    .unwrap_or("");
                
                if row_product_id == product_id {
                    // Нашли нужный продукт, ищем нужный склад
                    if let Some(stocks) = &row.stock_by_store {
                        for store_stock in stocks {
                            let row_store_id = store_stock.meta.href
                                .rsplit('/')
                                .next()
                                .unwrap_or("");
                            
                            if row_store_id == store_id {
                                // Возвращаем доступный остаток (stock - reserve)
                                return Ok(store_stock.stock - store_stock.reserve);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(0.0)
    }

    /// Получить товар с атрибутами
    pub async fn get_product(&self, product_id: &str) -> Result<Product> {
        debug!("Getting product: {}", product_id);
        
        self.get(&format!("/entity/product/{}?expand=attributes", product_id))
            .await
    }

    /// Найти тех. карту по названию
    pub async fn find_processing_plan_by_name(&self, name: &str) -> Result<Option<ProcessingPlan>> {
        info!("Searching for processing plan: {}", name);
        
        let response: ApiResponse<ProcessingPlan> = self
            .get(&format!(
                "/entity/processingplan?filter=name={}&expand=materials,products",
                urlencoding::encode(name)
            ))
            .await?;
        
        Ok(response.rows.and_then(|mut rows| rows.pop()))
    }

    /// Создать тех. операцию
    pub async fn create_processing(&self, request: &CreateProcessingRequest) -> Result<Processing> {
        info!("Creating processing operation");
        
        self.post("/entity/processing", request).await
    }

    /// Провести тех. операцию
    pub async fn apply_processing(&self, processing_id: &str) -> Result<Processing> {
        info!("Applying processing: {}", processing_id);
        
        #[derive(serde::Serialize)]
        struct ApplyRequest {
            applicable: bool,
        }
        
        self.put(
            &format!("/entity/processing/{}", processing_id),
            &ApplyRequest { applicable: true },
        )
        .await
    }

    /// Получить организацию
    pub async fn get_organization(&self) -> Result<Option<EntityRef>> {
        debug!("Getting organization");
        
        let response: ApiResponse<EntityRef> = self.get("/entity/organization").await?;
        Ok(response.rows.and_then(|mut rows| rows.pop()))
    }

    /// Получить заказ покупателя по ID
    pub async fn get_customer_order(&self, order_id: &str) -> Result<CustomerOrder> {
        info!("Getting customer order: {}", order_id);

        self.get(&format!(
            "/entity/customerorder/{}?expand=positions,store,organization,agent",
            order_id
        ))
        .await
    }
}
