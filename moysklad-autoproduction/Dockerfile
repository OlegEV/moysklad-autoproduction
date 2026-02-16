# Этап сборки - используем последнюю стабильную версию Rust
FROM rust:bookworm AS builder

WORKDIR /app

# Сначала копируем только файлы зависимостей для кэширования
COPY Cargo.toml Cargo.lock ./

# Создаём пустой src для кэширования зависимостей
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Теперь копируем исходный код и собираем
COPY src ./src
RUN cargo build --release

# Этап runtime - минимальный образ
FROM debian:bookworm-slim

# Устанавливаем необходимые библиотеки и curl для healthcheck
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Копируем бинарный файл
COPY --from=builder /app/target/release/moysklad-autoproduction /app/

# Создаём непривилегированного пользователя
RUN useradd -r -s /bin/false appuser && \
    chown -R appuser:appuser /app

USER appuser

# Открываем порт
EXPOSE 8080

# Переменные окружения по умолчанию
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT=8080
ENV MIN_STOCK_THRESHOLD=2
ENV STORE_NAME="Кобрино FBS"
ENV TECH_CARD_FIELD_NAME="Техкарта"

# Запускаем приложение
CMD ["./moysklad-autoproduction"]
