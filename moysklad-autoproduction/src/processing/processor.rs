//! Обработчик отгрузок и создание тех. операций

use crate::api::MoyskladClient;
use crate::config::Settings;
use crate::models::*;
use anyhow::{anyhow, Result};
use tracing::{debug, error, info, warn};

/// Процессор обработки отгрузок
pub struct DemandProcessor {
    client: MoyskladClient,
    settings: Settings,
    store_cache: Option<EntityRef>,
    organization_cache: Option<EntityRef>,
}

impl DemandProcessor {
    /// Создать новый процессор
    pub fn new(settings: Settings) -> Self {
        let token = settings.moysklad_token.clone();
        let client = MoyskladClient::new(token);
        
        Self {
            client,
            settings,
            store_cache: None,
            organization_cache: None,
        }
    }

    /// Получить кэшированный склад
    async fn get_store(&mut self) -> Result<EntityRef> {
        if let Some(ref store) = self.store_cache {
            return Ok(store.clone());
        }
        
        let store = self
            .client
            .find_store_by_name(&self.settings.store_name)
            .await?
            .ok_or_else(|| anyhow!("Store '{}' not found", self.settings.store_name))?;
        
        info!("Found store: {:?} ({:?})", store.name, store.id);
        self.store_cache = Some(store.clone());
        Ok(store)
    }

    /// Получить кэшированную организацию
    async fn get_organization(&mut self) -> Result<EntityRef> {
        if let Some(ref org) = self.organization_cache {
            return Ok(org.clone());
        }
        
        let org = self
            .client
            .get_organization()
            .await?
            .ok_or_else(|| anyhow!("No organization found"))?;
        
        info!("Found organization: {:?} ({:?})", org.name, org.id);
        self.organization_cache = Some(org.clone());
        Ok(org)
    }

    /// Обработать webhook событие
    pub async fn process_webhook(&mut self, event: &WebhookEvent) -> Result<Vec<ProcessingResult>> {
        info!(
            "Processing webhook event: type={}, action={}",
            event.entity_type, event.action
        );
        
        // Проверяем, что это событие отгрузки
        if event.entity_type != "demand" {
            debug!("Ignoring non-demand event: {}", event.entity_type);
            return Ok(vec![]);
        }
        
        // Получаем данные отгрузки
        let demand = if let Some(ref demand) = event.entity {
            demand.clone()
        } else if let Some(ref content) = event.content {
            if let Some(ref id) = content.id {
                self.client.get_demand(id).await?
            } else {
                return Err(anyhow!("No demand ID in webhook content"));
            }
        } else {
            return Err(anyhow!("No demand data in webhook event"));
        };
        
        // Проверяем, что отгрузка проведена
        if !demand.applicable {
            info!("Demand {} is not applicable, skipping", demand.name);
            return Ok(vec![ProcessingResult {
                success: true,
                message: "Отгрузка не проведена, пропускаем".to_string(),
                demand_id: Some(demand.id.clone()),
                demand_name: Some(demand.name.clone()),
                processing_id: None,
                processing_name: None,
                product: None,
                error: None,
            }]);
        }
        
        // Проверяем склад
        let store = self.get_store().await?;
        let demand_store_id = demand.store.id.as_ref().ok_or_else(|| anyhow!("Store ID missing"))?;
        let cached_store_id = store.id.as_ref().ok_or_else(|| anyhow!("Cached store ID missing"))?;
        
        if demand_store_id != cached_store_id {
            info!(
                "Demand store '{:?}' doesn't match monitored store '{:?}', skipping",
                demand.store.name, store.name
            );
            return Ok(vec![ProcessingResult {
                success: true,
                message: format!("Отгрузка с другого склада ({:?})", demand.store.name),
                demand_id: Some(demand.id.clone()),
                demand_name: Some(demand.name.clone()),
                processing_id: None,
                processing_name: None,
                product: None,
                error: None,
            }]);
        }
        
        // Обрабатываем позиции отгрузки
        self.process_demand_positions(&demand).await
    }

    /// Обработать позиции отгрузки
    async fn process_demand_positions(&mut self, demand: &Demand) -> Result<Vec<ProcessingResult>> {
        let mut results = Vec::new();
        
        let positions = match &demand.positions {
            Some(p) => &p.rows,
            None => {
                warn!("Demand {} has no positions", demand.name);
                return Ok(results);
            }
        };
        
        info!("Processing {} positions in demand {}", positions.len(), demand.name);
        
        for position in positions {
            match self.process_position(demand, position).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    error!("Error processing position: {}", e);
                    let product_info = self.extract_product_info_from_position(position);
                    results.push(ProcessingResult {
                        success: false,
                        message: format!("Ошибка обработки позиции: {}", e),
                        demand_id: Some(demand.id.clone()),
                        demand_name: Some(demand.name.clone()),
                        processing_id: None,
                        processing_name: None,
                        product: Some(product_info),
                        error: Some(e.to_string()),
                    });
                }
            }
        }
        
        Ok(results)
    }

    /// Извлечь информацию о продукте из позиции
    fn extract_product_info_from_position(&self, position: &DemandPosition) -> ProductInfo {
        // Извлекаем ID продукта из meta.href
        let product_id = position.assortment.meta.href
            .rsplit('/')
            .next()
            .unwrap_or("unknown")
            .to_string();
        
        ProductInfo {
            id: product_id,
            name: position.assortment.name.clone().unwrap_or_else(|| "unknown".to_string()),
            quantity: position.quantity,
            stock_before: 0.0,
        }
    }

    /// Обработать одну позицию отгрузки
    async fn process_position(
        &mut self,
        demand: &Demand,
        position: &DemandPosition,
    ) -> Result<ProcessingResult> {
        // Извлекаем ID продукта из meta.href ассортимента
        let product_id = position.assortment.meta.href
            .rsplit('/')
            .next()
            .ok_or_else(|| anyhow!("Cannot extract product ID from assortment href"))?
            .to_string();
        
        let product_name = position.assortment.name.clone()
            .unwrap_or_else(|| "unknown".to_string());
        let quantity = position.quantity;
        
        info!(
            "Processing position: product={}, quantity={}",
            product_name, quantity
        );
        
        // Получаем текущий остаток товара
        let store = self.get_store().await?;
        let store_id = store.id.as_ref().ok_or_else(|| anyhow!("Store ID missing"))?;
        let current_stock = self.client.get_product_stock(&product_id, store_id).await?;
        
        info!("Current stock for {}: {} (threshold: {})", product_name, current_stock, self.settings.min_stock_threshold);
        
        // Проверяем, нужно ли пополнение
        if current_stock >= self.settings.min_stock_threshold {
            info!("Stock is sufficient, skipping production for {}", product_name);
            return Ok(ProcessingResult {
                success: true,
                message: format!(
                    "Остаток достаточен ({} >= {})",
                    current_stock, self.settings.min_stock_threshold
                ),
                demand_id: Some(demand.id.clone()),
                demand_name: Some(demand.name.clone()),
                processing_id: None,
                processing_name: None,
                product: Some(ProductInfo {
                    id: product_id.clone(),
                    name: product_name.clone(),
                    quantity,
                    stock_before: current_stock,
                }),
                error: None,
            });
        }
        
        // Получаем товар для чтения атрибутов
        let product = self.client.get_product(&product_id).await?;
        
        // Ищем название тех. карты в атрибутах
        let tech_card_name = self.find_tech_card_name(&product)?;
        
        if tech_card_name.is_empty() {
            warn!("No tech card found for product {}", product_name);
            return Ok(ProcessingResult {
                success: false,
                message: "Тех. карта не найдена в карточке товара".to_string(),
                demand_id: Some(demand.id.clone()),
                demand_name: Some(demand.name.clone()),
                processing_id: None,
                processing_name: None,
                product: Some(ProductInfo {
                    id: product_id.clone(),
                    name: product_name.clone(),
                    quantity,
                    stock_before: current_stock,
                }),
                error: Some("Тех. карта не найдена".to_string()),
            });
        }
        
        info!("Found tech card name: {}", tech_card_name);
        
        // Получаем тех. карту
        let processing_plan = self
            .client
            .find_processing_plan_by_name(&tech_card_name)
            .await?
            .ok_or_else(|| anyhow!("Processing plan '{}' not found", tech_card_name))?;
        
        info!("Found processing plan: {} ({})", processing_plan.name, processing_plan.id);
        
        // Проверяем доступность материалов
        let materials_check = self
            .check_materials_availability(&processing_plan, quantity, store_id)
            .await?;
        
        if !materials_check.available {
            let missing = materials_check
                .missing
                .iter()
                .map(|(name, qty)| format!("{}: нужно {}, нет в наличии", name, qty))
                .collect::<Vec<_>>()
                .join(", ");
            
            warn!("Insufficient materials for production: {}", missing);
            return Ok(ProcessingResult {
                success: false,
                message: format!("Недостаточно материалов: {}", missing),
                demand_id: Some(demand.id.clone()),
                demand_name: Some(demand.name.clone()),
                processing_id: None,
                processing_name: None,
                product: Some(ProductInfo {
                    id: product_id.clone(),
                    name: product_name.clone(),
                    quantity,
                    stock_before: current_stock,
                }),
                error: Some(format!("Недостаточно материалов: {}", missing)),
            });
        }
        
        // Создаём тех. операцию
        let organization = self.get_organization().await?;
        let processing = self
            .create_processing_operation(
                &processing_plan,
                &store,
                &organization,
                quantity,
                demand,
                &product_name,
            )
            .await?;
        
        // Проводим тех. операцию
        let applied_processing = self.client.apply_processing(&processing.id).await?;
        
        info!(
            "Successfully created and applied processing: {} ({})",
            applied_processing.name, applied_processing.id
        );
        
        Ok(ProcessingResult {
            success: true,
            message: format!(
                "Создана тех. операция для производства {} шт. '{}'",
                quantity, product_name
            ),
            demand_id: Some(demand.id.clone()),
            demand_name: Some(demand.name.clone()),
            processing_id: Some(applied_processing.id.clone()),
            processing_name: Some(applied_processing.name.clone()),
            product: Some(ProductInfo {
                id: product_id.clone(),
                name: product_name.clone(),
                quantity,
                stock_before: current_stock,
            }),
            error: None,
        })
    }

    /// Найти название тех. карты в атрибутах товара
    fn find_tech_card_name(&self, product: &Product) -> Result<String> {
        let attributes = match &product.attributes {
            Some(attrs) => attrs,
            None => return Ok(String::new()),
        };
        
        for attr in attributes {
            if attr.name == self.settings.tech_card_field_name {
                if let Some(value) = attr.as_string() {
                    return Ok(value);
                }
            }
        }
        
        Ok(String::new())
    }

    /// Проверить доступность материалов
    async fn check_materials_availability(
        &self,
        processing_plan: &ProcessingPlan,
        quantity: f64,
        store_id: &str,
    ) -> Result<MaterialsCheckResult> {
        let materials_expanded = match &processing_plan.materials {
            Some(m) => m,
            None => return Ok(MaterialsCheckResult::available()),
        };
        
        let materials = match &materials_expanded.rows {
            Some(r) => r,
            None => return Ok(MaterialsCheckResult::available()),
        };
        
        let mut missing: Vec<(String, f64)> = Vec::new();
        
        for material in materials {
            // Количество материала на единицу продукции
            let material_qty = material.quantity * quantity;
            
            // Извлекаем ID материала из product.meta.href
            let material_id = material.product.meta.href
                .rsplit('/')
                .next()
                .unwrap_or("");
            
            // Получаем остаток материала
            let stock = self.client.get_product_stock(material_id, store_id).await?;
            
            let material_name = material.product.name.clone()
                .unwrap_or_else(|| "unknown".to_string());
            
            debug!(
                "Material {} stock: {}, needed: {}",
                material_name, stock, material_qty
            );
            
            // Проверяем с учётом резервов (доступный остаток уже учитывает резервы)
            if stock < material_qty {
                missing.push((material_name, material_qty - stock));
            }
        }
        
        if missing.is_empty() {
            Ok(MaterialsCheckResult::available())
        } else {
            Ok(MaterialsCheckResult::missing(missing))
        }
    }

    /// Создать тех. операцию
    async fn create_processing_operation(
        &self,
        processing_plan: &ProcessingPlan,
        store: &EntityRef,
        organization: &EntityRef,
        quantity: f64,
        demand: &Demand,
        product_name: &str,
    ) -> Result<Processing> {
        let request = CreateProcessingRequest {
            processing_plan: ProcessingPlanRef {
                meta: processing_plan.meta.clone(),
            },
            store: EntityRefSmall {
                meta: store.meta.clone(),
            },
            products_store: EntityRefSmall {
                meta: store.meta.clone(),
            },
            organization: EntityRefSmall {
                meta: organization.meta.clone(),
            },
            quantity,
            name: None,
            description: Some(format!(
                "Автоматически создано для отгрузки {} от {}",
                demand.name, demand.moment
            )),
            processing_sum: 0.0,
        };
        
        self.client.create_processing(&request).await
    }
}

/// Результат проверки материалов
struct MaterialsCheckResult {
    available: bool,
    missing: Vec<(String, f64)>,
}

impl MaterialsCheckResult {
    fn available() -> Self {
        Self {
            available: true,
            missing: Vec::new(),
        }
    }
    
    fn missing(missing: Vec<(String, f64)>) -> Self {
        Self {
            available: false,
            missing,
        }
    }
}
