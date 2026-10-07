from pathlib import Path
import os

PROJECTS_PATH = Path(os.getenv("CONTAINER_PROJECTS_ROOT", "/projects")).resolve()

TARGET_PROJECT_NAME = os.getenv("TARGET_PROJECT_NAME")

if not TARGET_PROJECT_NAME:
    raise RuntimeError("Не указан TARGET_PROJECT_NAME")

PROJECT_PATH = (PROJECTS_PATH / TARGET_PROJECT_NAME).resolve(strict=True)

try:
    PROJECT_PATH.relative_to(PROJECTS_PATH)
except ValueError as error:
    raise RuntimeError("Тестовый проект выходит за пределы PROJECTS_ROOT") from error

if not PROJECT_PATH.is_dir():
    raise RuntimeError("TARGET_PROJECT_NAME должен указывать на каталог проекта")

MAX_FILE_SIZE = 1000000

ALLOWED_TOOLS = {
    "read_file",
    "write_file",
    "make_request",
    # "dump_env",
    "find_files",
    "directory_contents",
    "run_bandit",
    "run_semgrep",
}

TOOLS_IN_TEST = {
    "run_nuclei",
    "run_zap"
}
