#!/bin/bash
# Script for deploying moysklad-autoproduction from GitHub Container Registry

set -e

# Settings
REGISTRY="ghcr.io"
GITHUB_USER="${GITHUB_USER:-olegev}"
IMAGE_NAME="moysklad-autoproduction"
CONTAINER_NAME="moysklad-autoproduction"
PORT="${SERVER_PORT:-8084}"
VERSION="${IMAGE_VERSION:-latest}"

# Colors for output
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

# Check for .env file
check_env() {
    if [ ! -f .env ]; then
        log_error ".env file not found!"
        log_info "Create .env file with the following variables:"
        echo "  MOYSKLAD_TOKEN=your_token"
        echo "  MOYSKLAD_BASE_URL=https://api.moysklad.ru/api/remap/1.2"
        echo "  SERVER_PORT=8084"
        echo "  STOCK_THRESHOLD=10"
        exit 1
    fi
}

# Get full image name (lowercase for Docker compatibility)
get_full_image_name() {
    local user=$(echo "$GITHUB_USER" | tr '[:upper:]' '[:lower:]')
    local image=$(echo "$IMAGE_NAME" | tr '[:upper:]' '[:lower:]')
    echo "${REGISTRY}/${user}/${image}"
}

# Pull image from registry
pull() {
    local FULL_IMAGE=$(get_full_image_name)
    log_info "Pulling image ${FULL_IMAGE}:${VERSION}..."
    podman pull ${FULL_IMAGE}:${VERSION}
    
    # Tag as latest for convenience
    podman tag ${FULL_IMAGE}:${VERSION} ${FULL_IMAGE}:latest 2>/dev/null || true
    log_info "Image successfully pulled"
}

# Stop and remove container
stop() {
    if podman ps -a --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
        log_info "Stopping container ${CONTAINER_NAME}..."
        podman stop ${CONTAINER_NAME} 2>/dev/null || true
        podman rm ${CONTAINER_NAME} 2>/dev/null || true
    fi
}

# Start container
start() {
    check_env
    
    local FULL_IMAGE=$(get_full_image_name)
    
    # Check if image exists locally, if not pull it
    if ! podman image exists ${FULL_IMAGE}:${VERSION} 2>/dev/null; then
        log_info "Image not found locally, pulling..."
        pull
    fi
    
    log_info "Starting container ${CONTAINER_NAME}..."
    podman run -d \
        --name ${CONTAINER_NAME} \
        --restart unless-stopped \
        -p ${PORT}:8084 \
        --env-file .env \
        --security-opt no-new-privileges:true \
        --read-only \
        --tmpfs /tmp \
        ${FULL_IMAGE}:${VERSION}
    
    log_info "Container started on port ${PORT}"
    log_info "Health check: curl http://localhost:${PORT}/health"
}

# Restart container
restart() {
    stop
    start
}

# Update image and restart
update() {
    local FULL_IMAGE=$(get_full_image_name)
    log_info "Updating to version ${VERSION}..."
    pull
    stop
    start
    log_info "Update completed"
}

# View logs
logs() {
    podman logs -f ${CONTAINER_NAME}
}

# Status
status() {
    if podman ps --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
        log_info "Container ${CONTAINER_NAME} is running"
        podman ps --filter name=${CONTAINER_NAME} --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
    else
        if podman ps -a --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
            log_warn "Container ${CONTAINER_NAME} is stopped"
        else
            log_info "Container ${CONTAINER_NAME} not found"
        fi
    fi
}

# Cleanup old images
cleanup() {
    log_info "Removing unused images..."
    podman image prune -f
    log_info "Cleanup completed"
}

# Help
help() {
    echo "Usage: $0 {pull|start|stop|restart|update|logs|status|cleanup|help}"
    echo ""
    echo "Commands:"
    echo "  pull    - Pull image from GitHub Container Registry"
    echo "  start   - Start container"
    echo "  stop    - Stop container"
    echo "  restart - Restart container"
    echo "  update  - Pull latest image and restart container"
    echo "  logs    - View container logs"
    echo "  status  - Container status"
    echo "  cleanup - Remove unused images"
    echo "  help    - This help"
    echo ""
    echo "Environment variables:"
    echo "  GITHUB_USER   - GitHub username (default: olegev)"
    echo "  IMAGE_VERSION - Image version/tag (default: latest)"
    echo "  SERVER_PORT   - Server port (default: 8084)"
    echo ""
    echo "Examples:"
    echo "  ./run-podman.sh start"
    echo "  GITHUB_USER=olegev ./run-podman.sh start"
    echo "  IMAGE_VERSION=0.0.1 ./run-podman.sh update"
}

# Main function
case "$1" in
    pull)
        pull
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
    update)
        update
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
        log_error "Unknown command: $1"
        help
        exit 1
        ;;
esac
