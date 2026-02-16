//! Типы данных для API МойСклад

use serde::{Deserialize, Serialize};

/// Метаданные сущности
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
    pub href: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "metadataHref")]
    pub metadata_href: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    pub entity_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
}

/// Ссылка на сущность
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityRef {
    pub meta: Meta,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Склад
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Store {
    pub meta: Meta,
    pub id: String,
    pub name: String,
}

/// Товар
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub meta: Meta,
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Vec<Attribute>>,
}

/// Дополнительное поле (атрибут)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attribute {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub attr_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<AttributeValue>,
}

/// Значение атрибута
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AttributeValue {
    String(String),
    Number(f64),
    Boolean(bool),
    EntityRef(EntityRef),
}

impl Attribute {
    /// Получить строковое значение атрибута
    pub fn as_string(&self) -> Option<String> {
        match &self.value {
            Some(AttributeValue::String(s)) => Some(s.clone()),
            Some(AttributeValue::Number(n)) => Some(n.to_string()),
            Some(AttributeValue::Boolean(b)) => Some(b.to_string()),
            Some(AttributeValue::EntityRef(e)) => e.name.clone(),
            None => None,
        }
    }
}

/// Строка отчёта по остаткам
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockRow {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<Meta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stock: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reserve: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_transit: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub article: Option<String>,
    pub assortment_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stock_by_store: Option<Vec<StoreStock>>,
}

/// Остаток по конкретному складу
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreStock {
    pub meta: Meta,
    pub stock: f64,
    pub reserve: f64,
    pub in_transit: f64,
}

impl StockRow {
    /// Получить доступный остаток (stock - reserve)
    pub fn available(&self) -> f64 {
        (self.stock.unwrap_or(0.0)) - (self.reserve.unwrap_or(0.0))
    }
}

/// Строка отчёта по остаткам по складам
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockByStoreRow {
    pub meta: Meta,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stock_by_store: Option<Vec<StoreStockInfo>>,
}

/// Остаток по конкретному складу в отчёте
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreStockInfo {
    pub meta: Meta,
    pub name: String,
    pub stock: f64,
    pub reserve: f64,
    pub in_transit: f64,
}

/// Техническая карта
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingPlan {
    pub meta: Meta,
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub products: Option<ProcessingPlanProductsExpanded>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub materials: Option<ProcessingPlanMaterialsExpanded>,
}

/// Продукты тех. карты (развёрнутые)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingPlanProductsExpanded {
    pub meta: Meta,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rows: Option<Vec<ProcessingPlanProduct>>,
}

/// Материалы тех. карты (развёрнутые)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingPlanMaterialsExpanded {
    pub meta: Meta,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rows: Option<Vec<ProcessingPlanMaterial>>,
}

/// Продукт в тех. карте (что производим)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingPlanProduct {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub product: EntityRef,
    pub assortment: EntityRef,
    pub quantity: f64,
}

/// Материал в тех. карте (из чего производим)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingPlanMaterial {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub product: EntityRef,
    pub assortment: EntityRef,
    pub quantity: f64,
}

/// Технологическая операция
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Processing {
    pub meta: Meta,
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub applicable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "status")]
    pub status_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "processingPlan")]
    pub processing_plan: Option<EntityRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub products: Option<ProcessingProducts>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub materials: Option<ProcessingMaterials>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<EntityRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization: Option<EntityRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<String>,
}

/// Продукты тех. операции (с мета-ссылкой)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingProducts {
    pub meta: Meta,
}

/// Материалы тех. операции (с мета-ссылкой)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingMaterials {
    pub meta: Meta,
}

/// Продукт в тех. операции
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingProduct {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<Meta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processing_plan_position: Option<PlanPosition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processing_plan_product: Option<EntityRef>,
    pub assortment: EntityRef,
    pub product: EntityRef,
    pub quantity: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity_per_product: Option<f64>,
}

/// Материал в тех. операции
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingMaterial {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<Meta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processing_plan_position: Option<PlanPosition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processing_plan_material: Option<EntityRef>,
    pub assortment: EntityRef,
    pub product: EntityRef,
    pub quantity: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity_per_product: Option<f64>,
}

/// Позиция в тех. карте
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanPosition {
    pub meta: Meta,
    pub id: String,
    pub quantity: f64,
}

/// Отгрузка (Demand)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Demand {
    pub meta: Meta,
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_code: Option<String>,
    pub moment: String,
    pub applicable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "status")]
    pub status_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<EntityRef>,
    pub store: EntityRef,
    pub organization: EntityRef,
    pub agent: EntityRef,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub positions: Option<DemandPositions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<String>,
}

/// Позиции отгрузки
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemandPositions {
    pub meta: Meta,
    pub rows: Vec<DemandPosition>,
}

/// Позиция отгрузки
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemandPosition {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<Meta>,
    pub assortment: EntityRef,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product: Option<EntityRef>,
    pub quantity: f64,
    #[serde(default)]
    pub price: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discount: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vat: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reserve: Option<f64>,
}

/// Событие webhook от МойСклад
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEvent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<Meta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub account_id: String,
    pub entity_type: String,
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity: Option<Demand>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<WebhookContent>,
}

/// Контент webhook события
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity: Option<Demand>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_type: Option<String>,
}

/// Ответ API с пагинацией
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<ResponseMeta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rows: Option<Vec<T>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<Context>,
}

/// Метаданные ответа
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMeta {
    pub href: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    pub meta_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "mediaType")]
    pub media_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
}

/// Контекст ответа
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub employee: Option<EmployeeRef>,
}

/// Ссылка на сотрудника
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeRef {
    pub meta: Meta,
}

/// Данные для создания тех. операции
#[derive(Debug, Clone, Serialize)]
pub struct CreateProcessingRequest {
    #[serde(rename = "processingPlan")]
    pub processing_plan: ProcessingPlanRef,
    pub store: EntityRefSmall,
    #[serde(rename = "productsStore")]
    pub products_store: EntityRefSmall,
    pub organization: EntityRefSmall,
    pub quantity: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "processingSum")]
    pub processing_sum: f64,
}

/// Ссылка на тех. карту
#[derive(Debug, Clone, Serialize)]
pub struct ProcessingPlanRef {
    pub meta: Meta,
}

/// Сокращённая ссылка на сущность
#[derive(Debug, Clone, Serialize)]
pub struct EntityRefSmall {
    pub meta: Meta,
}

/// Входной продукт для тех. операции
#[derive(Debug, Clone, Serialize)]
pub struct ProcessingProductInput {
    pub product: EntityRefSmall,
    pub quantity: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processing_plan_position: Option<PlanPositionRef>,
}

/// Входной материал для тех. операции
#[derive(Debug, Clone, Serialize)]
pub struct ProcessingMaterialInput {
    pub product: EntityRefSmall,
    pub quantity: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processing_plan_position: Option<PlanPositionRef>,
}

/// Ссылка на позицию тех. карты
#[derive(Debug, Clone, Serialize)]
pub struct PlanPositionRef {
    pub meta: Meta,
}

/// Результат обработки отгрузки
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingResult {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub demand_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub demand_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processing_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processing_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product: Option<ProductInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Информация о продукте
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductInfo {
    pub id: String,
    pub name: String,
    pub quantity: f64,
    pub stock_before: f64,
}
