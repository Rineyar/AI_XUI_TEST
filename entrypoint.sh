#!/bin/sh
set -e

# Отдаём агенту
chown -R agent:agent /agent/workspace/zap_plans 2>/dev/null || true

#Копировать файлы агенту
cp -R /agent/import/. /agent/workspace/

#Запустить агента
exec /agent/agent_core/agent_core "$@"