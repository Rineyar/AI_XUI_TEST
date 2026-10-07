# Запуск агента для проверки проектов

## 1. Подготовка проверяемых проектов

Все проекты рекомендуется хранить внутри одной общей директории:

```text
C:\<Имя общей директории>
├── <проект 1>
├── <проект 2>
└── <проект 3>
```

Агенту подключается вся директория `<глобальный путь>/<Имя общей директории>`, а переменная `TARGET_PROJECT_NAME` выбирает конкретный проект.

## 3. Настройка `.env` агента
```env
PROJECTS_ROOT=C:/SecurityProjects
CONTAINER_PROJECTS_ROOT=/projects
TARGET_PROJECT_NAME=vulnerable-app
TARGET_URL=http://test-project:8000
```

### Новые параметры

#### `PROJECTS_ROOT`

Абсолютный путь на компьютере до общей директории с проверяемыми проектами:

```env
PROJECTS_ROOT=C:/SecurityProjects
```

#### `CONTAINER_PROJECTS_ROOT`

Путь, по которому проекты доступны внутри контейнера агента:

```env
CONTAINER_PROJECTS_ROOT=/projects
```
#### `TARGET_PROJECT_NAME`

Название проверяемого проекта внутри `PROJECTS_ROOT`:

```env
TARGET_PROJECT_NAME=vulnerable-app
```
#### `TARGET_URL`

Адрес запущенного тестового приложения внутри Docker-сети:

```env
TARGET_URL=http://test-project:8000
```

Здесь:

- `test-project` — имя сервиса в Compose тестового проекта;
- `8000` — порт, который слушает приложение внутри контейнера.

## 4. Сборка агента

Собрать образ:

```powershell
docker compose build agent
```

## 5. Запуск агента

```powershell
docker compose up -d agent
```

Подключиться к интерактивной консоли:

```powershell
docker attach agent-core
```

## 6. Определение имени сети агента

Compose создаёт сеть для DAST автоматически. Узнать её точное имя:

```powershell
docker network ls --filter name=scan-network
```

Пример результата:

```text
ai_xui_test_scan-network
```

Это имя потребуется в Compose тестового проекта.

## 7. Шаблон Compose для тестового проекта

В корне тестового проекта создать отдельный файл `compose.test.yaml`:

```yaml
services:
  test-project:
    build:
      context: .
      dockerfile: Dockerfile

    container_name: test-project

    expose:
      - "8000"

    networks:
      - agent-scan-network
      
networks:
  agent-scan-network:
    external: true
    name: ${AGENT_SCAN_NETWORK}
```

Рядом создать `.env` тестового проекта:

```env
AGENT_SCAN_NETWORK=ai_xui_test_scan-network
```

Значение должно совпадать с именем сети, полученным командой:

```powershell
docker network ls --filter name=scan-network
```

## 8. Требования к тестовому приложению

Приложение внутри контейнера должно слушать:

```text
0.0.0.0
```

Использовать `127.0.0.1` нельзя, иначе другие контейнеры не смогут подключиться.

Пример для Flask:

```python
app.run(host="0.0.0.0", port=8000)
```

Пример для FastAPI:

```dockerfile
CMD ["uvicorn", "main:app", "--host", "0.0.0.0", "--port", "8000"]
```

Порт приложения должен соответствовать `expose` и `TARGET_URL`.

## 11. Смена проверяемого проекта

Остановить предыдущий тестовый проект:


В `.env` агента изменить:
```env
TARGET_PROJECT_NAME=project-two
TARGET_URL=http://test-project:8000
```

Пересоздать контейнер агента, чтобы он получил новые переменные:

```powershell
docker compose up -d agent
```

Затем запустить Compose нового тестового проекта.

Повторная сборка агента при смене проекта не требуется.
