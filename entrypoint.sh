#!/bin/sh
set -e

#Копировать файлы агенту
cp -R /agent/import/. /agent/workspace/

#Запустить агента
exec /agent/agent_core/agent_core "$@"