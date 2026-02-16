//! Конфигурация приложения

use std::env;

/// Настройки приложения
#[derive(Debug, Clone)]
pub struct Settings {
    /// Токен доступа к API МойСклад
    pub moysklad_token: String,
    
    /// Название склада для отслеживания
    pub store_name: String,
    
    /// Название поля с тех. картой в карточке товара
    pub tech_card_field_name: String,
    
    /// Минимальный порог остатка
    pub min_stock_threshold: f64,
    
    /// Порт веб-сервера
    pub server_port: u16,
    
    /// Хост веб-сервера
    pub server_host: String,
}

impl Settings {
    /// Загрузить настройки из переменных окружения
    pub fn from_env() -> Result<Self, String> {
        let moysklad_token = env::var("MOYSKLAD_TOKEN")
            .map_err(|_| "MOYSKLAD_TOKEN is required".to_string())?;
        
        let store_name = env::var("STORE_NAME")
            .unwrap_or_else(|_| "Кобрино FBS".to_string());
        
        let tech_card_field_name = env::var("TECH_CARD_FIELD_NAME")
            .unwrap_or_else(|_| "Техкарта".to_string());
        
        let min_stock_threshold = env::var("MIN_STOCK_THRESHOLD")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(2.0);
        
        let server_port = env::var("SERVER_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(8080);
        
        let server_host = env::var("SERVER_HOST")
            .unwrap_or_else(|_| "0.0.0.0".to_string());
        
        Ok(Self {
            moysklad_token,
            store_name,
            tech_card_field_name,
            min_stock_threshold,
            server_port,
            server_host,
        })
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            moysklad_token: String::new(),
            store_name: "Кобрино FBS".to_string(),
            tech_card_field_name: "Техкарта".to_string(),
            min_stock_threshold: 2.0,
            server_port: 8080,
            server_host: "0.0.0.0".to_string(),
        }
    }
}
