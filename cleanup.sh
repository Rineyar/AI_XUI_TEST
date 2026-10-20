#!/bin/sh

cleanup() {
    echo "[cleanup] Удаление внутренних контейнеров..."
    docker ps -aq 2>/dev/null | xargs -r docker rm -f 2>/dev/null || true
    echo "[cleanup] Готово"
}

shutdown() {
    echo "[shutdown] Получен сигнал, останавливаю внутренние контейнеры..."
    if [ -n "$CHILD" ]; then
        kill -TERM "$CHILD" 2>/dev/null || true
        wait "$CHILD" 2>/dev/null || true
    fi
    cleanup
    exit 0
}

# Почистить остатки с прошлого запуска, если есть
cleanup

# На SIGTERM/SIGINT — остановить агента и почистить
trap shutdown TERM INT

# Запускаем настоящий entrypoint
/bin/sh /usr/local/bin/entrypoint.sh "$@" &
CHILD=$!

wait "$CHILD" 2>/dev/null
EXIT_CODE=$?

# На exit тоже чистим
cleanup
exit "$EXIT_CODE"