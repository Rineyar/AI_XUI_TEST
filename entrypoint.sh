#!/bin/sh
set -e

#Копировать файлы агенту
cp -R /agent/import/. /agent/workspace/
chown -R agent:agent /agent/workspace/zap-plans 2>/dev/null || true

#Запустить агента
exec setpriv --reuid=1000 --regid=1000 --clear-groups /agent/agent_core/agent_core "$@"