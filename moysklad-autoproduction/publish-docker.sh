#!/bin/bash

# Скрипт для сборки и публикации Docker образа в GitHub Container Registry
# Использование: ./publish-docker.sh [версия] [github_username]
# Пример: ./publish-docker.sh 1.0.0 myusername

set -e

# Конфигурация
IMAGE_NAME="moysklad-autoproduction"
REGISTRY="ghcr.io"

# Получаем параметры или используем значения по умолчанию
VERSION="${1:-latest}"
GITHUB_USER="${2:-}"

# Если пользователь не указан, пытаемся получить из git remote
if [ -z "$GITHUB_USER" ]; then
    GITHUB_USER=$(git remote get-url origin 2>/dev/null | sed -E 's|.*github\.com[/:]([^/]+).*|\1|' || echo "")
fi

if [ -z "$GITHUB_USER" ]; then
    echo "Ошибка: Не удалось определить GitHub username"
    echo "Использование: $0 [версия] [github_username]"
    echo "Пример: $0 1.0.0 myusername"
    exit 1
fi

# Полное имя образа
FULL_IMAGE_NAME="${REGISTRY}/${GITHUB_USER}/${IMAGE_NAME}"

echo "=========================================="
echo "Сборка и публикация Docker образа"
echo "=========================================="
echo "Registry: ${REGISTRY}"
echo "Image: ${FULL_IMAGE_NAME}"
echo "Version: ${VERSION}"
echo "=========================================="

# Переходим в директорию скрипта
cd "$(dirname "$0")"

# Сборка Docker образа
echo ""
echo "[1/3] Сборка Docker образа..."
docker build -t "${FULL_IMAGE_NAME}:${VERSION}" -t "${FULL_IMAGE_NAME}:latest" .

# Проверка успешности сборки
if [ $? -ne 0 ]; then
    echo "Ошибка при сборке Docker образа"
    exit 1
fi

echo ""
echo "[2/3] Вход в GitHub Container Registry..."
echo "Примечание: Для входа нужен GitHub Personal Access Token (PAT) с правами write:packages"
echo "Токен можно создать здесь: https://github.com/settings/tokens"
echo ""

# Вход в реестр (потребуется ввод токена)
docker login ${REGISTRY} -u "${GITHUB_USER}" --password-stdin || {
    echo "Ошибка при входе в реестр. Убедитесь, что у вас есть PAT с правами write:packages"
    exit 1
}

echo ""
echo "[3/3] Публикация образа..."

# Публикация образа с тегом версии
docker push "${FULL_IMAGE_NAME}:${VERSION}"

# Публикация образа с тегом latest
docker push "${FULL_IMAGE_NAME}:latest"

echo ""
echo "=========================================="
echo "Успешно опубликовано!"
echo "=========================================="
echo "Образ доступен по адресу:"
echo "  ${FULL_IMAGE_NAME}:${VERSION}"
echo "  ${FULL_IMAGE_NAME}:latest"
echo ""
echo "Для использования образа выполните:"
echo "  docker pull ${FULL_IMAGE_NAME}:${VERSION}"
echo ""
echo "Для запуска контейнера:"
echo "  docker run -p 8084:8084 ${FULL_IMAGE_NAME}:${VERSION}"
echo "=========================================="
