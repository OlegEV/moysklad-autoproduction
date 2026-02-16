# Автопроизводство для МойСклад

Автоматическое создание тех. операций при низких остатках товара.

## Принцип работы

1. МойСклад отправляет webhook при создании/изменении отгрузки
2. Сервис проверяет остатки товаров на складе
3. Если остаток ниже порога (< 2 шт.), проверяется наличие тех. карты
4. Проверяется доступность материалов с учётом резервов
5. Создаётся и проводится тех. операция на производство

## Требования

- Rust 1.70+ (для сборки)
- Docker (для запуска в контейнере)
- Токен доступа к API МойСклад
- Настроенные тех. карты в МойСклад

## Конфигурация

| Переменная | Описание | По умолчанию |
|------------|----------|--------------|
| `MOYSKLAD_TOKEN` | Токен API МойСклад | (обязательно) |
| `STORE_NAME` | Название склада | `Кобрино FBS` |
| `TECH_CARD_FIELD_NAME` | Имя поля с тех. картой | `Техкарта` |
| `MIN_STOCK_THRESHOLD` | Мин. остаток | `2` |
| `SERVER_PORT` | Порт сервера | `8080` |
| `SERVER_HOST` | Хост сервера | `0.0.0.0` |

## Запуск

### Через Podman (рекомендуется)

```bash
# Создайте .env файл с токеном
echo "MOYSKLAD_TOKEN=ваш_токен" > .env

# Вариант 1: Через podman-compose
podman-compose up -d

# Вариант 2: Через скрипт
chmod +x run-podman.sh
./run-podman.sh build   # Сборка образа
./run-podman.sh start   # Запуск контейнера
./run-podman.sh logs    # Просмотр логов
./run-podman.sh status  # Статус
./run-podman.sh stop    # Остановка
```

### Через systemd (автозапуск с Podman)

```bash
# Копируем файлы
sudo mkdir -p /opt/moysklad-autoproduction
sudo cp .env /opt/moysklad-autoproduction/
sudo cp moysklad-autoproduction.service /etc/systemd/system/

# Включаем и запускаем
sudo systemctl daemon-reload
sudo systemctl enable moysklad-autoproduction
sudo systemctl start moysklad-autoproduction

# Просмотр логов
sudo journalctl -u moysklad-autoproduction -f
```

### Через Docker Compose

```bash
# Создайте .env файл с токеном
echo "MOYSKLAD_TOKEN=ваш_токен" > .env

# Запустите
docker-compose up -d
```

### Через Docker

```bash
docker build -t moysklad-autoproduction .
docker run -d \
  -p 8084:8084 \
  -e MOYSKLAD_TOKEN=ваш_токен \
  moysklad-autoproduction
```

### Локально (для разработки)

```bash
cp .env.example .env
# Отредактируйте .env, добавив токен
cargo run --release
```

## API Endpoints

| Endpoint | Method | Описание |
|----------|--------|----------|
| `/health` | GET | Health check |
| `/webhook` | POST | Webhook от МойСклад |
| `/demand/{id}/process` | POST | Ручная обработка отгрузки |
| `/config` | GET | Текущая конфигурация |

## Настройка webhook в МойСклад

1. Откройте МойСклад → Настройки → API
2. Создайте webhook на события:
   - Тип сущности: `demand` (Отгрузка)
   - Действие: `create`, `update`
3. URL: `http://ваш-сервер:8080/webhook`

## Пример webhook события

```json
{
  "accountId": "xxx",
  "entityType": "demand",
  "action": "create",
  "content": {
    "id": "demand-uuid-here"
  }
}
```

## Логирование

Все события логируются в stdout в формате JSON. Для просмотра:

```bash
docker logs -f moysklad-autoproduction
```

## Проверка работы

```bash
# Health check
curl http://localhost:8080/health

# Ручная обработка отгрузки
curl -X POST http://localhost:8080/demand/UUID-ОТГРУЗКИ/process
```

## Архитектура

```
                    ┌─────────────────────────────────────┐
                    │           МойСклад                  │
                    │  (webhook при отгрузке)              │
                    └──────────────┬──────────────────────┘
                                   │
                                   ▼
                    ┌─────────────────────────────────────┐
                    │         HTTP Server (Actix-web)     │
                    │              :8080                   │
                    └──────────────┬──────────────────────┘
                                   │
                    ┌──────────────▼──────────────────────┐
                    │          Webhook Handler            │
                    └──────────────┬──────────────────────┘
                                   │
                    ┌──────────────▼──────────────────────┐
                    │         DemandProcessor             │
                    │                                      │
                    │  1. Получить данные отгрузки         │
                    │  2. Проверить склад                  │
                    │  3. Для каждой позиции:              │
                    │     - Проверить остаток              │
                    │     - Найти тех. карту               │
                    │     - Проверить материалы            │
                    │     - Создать тех. операцию          │
                    └──────────────┬──────────────────────┘
                                   │
                    ┌──────────────▼──────────────────────┐
                    │        MoyskladClient               │
                    │       (API HTTP requests)            │
                    └─────────────────────────────────────┘
```

## Лицензия

MIT
