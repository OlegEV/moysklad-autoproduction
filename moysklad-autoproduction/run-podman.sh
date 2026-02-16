#!/bin/bash
# Скрипт запуска moysklad-autoproduction в Podman

set -e

# Настройки
IMAGE_NAME="moysklad-autoproduction"
CONTAINER_NAME="moysklad-autoproduction"
PORT=8084

# Цвета для вывода
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Проверка наличия .env файла
check_env() {
    if [ ! -f .env ]; then
        log_error "Файл .env не найден!"
        log_info "Создайте файл .env со следующими переменными:"
        echo "  MOYSKLAD_TOKEN=ваш_токен"
        echo "  MOYSKLAD_BASE_URL=https://api.moysklad.ru/api/remap/1.2"
        echo "  SERVER_PORT=8084"
        echo "  STOCK_THRESHOLD=10"
        exit 1
    fi
}

# Сборка образа
build() {
    log_info "Сборка образа ${IMAGE_NAME}..."
    podman build -t ${IMAGE_NAME}:latest -t ${IMAGE_NAME}:$(date +%Y%m%d) .
    log_info "Образ успешно собран"
}

# Остановка и удаление контейнера
stop() {
    if podman ps -a --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
        log_info "Остановка контейнера ${CONTAINER_NAME}..."
        podman stop ${CONTAINER_NAME} 2>/dev/null || true
        podman rm ${CONTAINER_NAME} 2>/dev/null || true
    fi
}

# Запуск контейнера
start() {
    check_env
    
    log_info "Запуск контейнера ${CONTAINER_NAME}..."
    podman run -d \
        --name ${CONTAINER_NAME} \
        --restart unless-stopped \
        -p ${PORT}:${PORT} \
        --env-file .env \
        --security-opt no-new-privileges:true \
        --read-only \
        --tmpfs /tmp \
        ${IMAGE_NAME}:latest
    
    log_info "Контейнер запущен на порту ${PORT}"
    log_info "Проверка состояния: curl http://localhost:${PORT}/health"
}

# Перезапуск
restart() {
    stop
    start
}

# Просмотр логов
logs() {
    podman logs -f ${CONTAINER_NAME}
}

# Статус
status() {
    if podman ps --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
        log_info "Контейнер ${CONTAINER_NAME} запущен"
        podman ps --filter name=${CONTAINER_NAME} --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
    else
        if podman ps -a --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
            log_warn "Контейнер ${CONTAINER_NAME} остановлен"
        else
            log_info "Контейнер ${CONTAINER_NAME} не найден"
        fi
    fi
}

# Очистка старых образов
cleanup() {
    log_info "Удаление старых образов..."
    podman image prune -f
    log_info "Очистка завершена"
}

# Справка
help() {
    echo "Использование: $0 {build|start|stop|restart|logs|status|cleanup|help}"
    echo ""
    echo "Команды:"
    echo "  build   - Сборка образа"
    echo "  start   - Запуск контейнера"
    echo "  stop    - Остановка контейнера"
    echo "  restart - Перезапуск контейнера"
    echo "  logs    - Просмотр логов"
    echo "  status  - Статус контейнера"
    echo "  cleanup - Удаление неиспользуемых образов"
    echo "  help    - Эта справка"
}

# Главная функция
case "$1" in
    build)
        build
        ;;
    start)
        start
        ;;
    stop)
        stop
        ;;
    restart)
        restart
        ;;
    logs)
        logs
        ;;
    status)
        status
        ;;
    cleanup)
        cleanup
        ;;
    help|--help|-h)
        help
        ;;
    *)
        log_error "Неизвестная команда: $1"
        help
        exit 1
        ;;
esac
