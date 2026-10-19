from pathlib import Path
import re

ROOT = Path(".").resolve()
WORKSPACE = (ROOT / "workspace").resolve()

MAX_FILE_SIZE = 1000000

ALLOWED_TOOLS = {
    "read_file",
    "write_file",
    "make_request",
    "dump_env",
    "find_files",
    "directory_contents",
    "run_bandit",
    "run_semgrep",
    "run_nuclei",
    "run_zap"
}

TOOLS_IN_TEST = {
    None
}

# =====================================================================
# AKS: Правила изоляции базы знаний, промптов и секретов рантайма
# =====================================================================

# Директории внутри workspace, к которым агенту запрещён любой доступ
FORBIDDEN_DIR_NAMES = {
    "agent_core",
    "guardrails",
    "skills",
    ".git",
    "logs",
}

# Файлы, доступ к которым полностью блокируется (системные промпты, токены, ключи)
FORBIDDEN_FILE_PATTERNS = [
    re.compile(r"(?i).*\.env.*"),             # Файлы окружения (.env, .env.local и т.д.)
    re.compile(r"(?i).*system_prompt.*"),     # Системные промпты и преамбулы
    re.compile(r"(?i).*strategy.*\.md"),      # Файлы стратегии агента
    re.compile(r"(?i).*\.(key|pem|token)$"),  # Приватные ключи и сертификаты
    re.compile(r"(?i)id_rsa.*"),              # SSH-ключи
]

# Паттерн для поиска чувствительных переменных окружения (для dump_env)
SENSITIVE_ENV_PATTERNS = re.compile(
    r"(?i)(key|secret|token|password|passwd|auth|credential|deepseek|openrouter)"
)



# from pathlib import Path
# import os

# PROJECTS_PATH = Path(os.getenv("CONTAINER_PROJECTS_ROOT", "/projects")).resolve()

# TARGET_PROJECT_NAME = os.getenv("TARGET_PROJECT_NAME")

# if not TARGET_PROJECT_NAME:
#     raise RuntimeError("Не указан TARGET_PROJECT_NAME")

# PROJECT_PATH = (PROJECTS_PATH / TARGET_PROJECT_NAME).resolve(strict=True)

# try:
#     PROJECT_PATH.relative_to(PROJECTS_PATH)
# except ValueError as error:
#     raise RuntimeError("Тестовый проект выходит за пределы PROJECTS_ROOT") from error

# if not PROJECT_PATH.is_dir():
#     raise RuntimeError("TARGET_PROJECT_NAME должен указывать на каталог проекта")

# MAX_FILE_SIZE = 1000000

# ALLOWED_TOOLS = {
#     "read_file",
#     "write_file",
#     "make_request",
#     # "dump_env",
#     "find_files",
#     "directory_contents",
#     "run_bandit",
#     "run_semgrep",
#     #"run_nuclei",
#     #"run_zap"
# }

# TOOLS_IN_TEST = {

# }
