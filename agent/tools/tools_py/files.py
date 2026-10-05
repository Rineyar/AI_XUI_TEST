import subprocess
import json
import os
import glob
from pathlib import Path

__all__ = ["read_file", "write_file", "run_file", "find_files"]

WORKSPACE = Path("./workspace").resolve()


def _resolve_workspace_path(filename: str) -> Path:
    """
    Приводит любой переданный путь (как 'test.py', так и 'workspace/test.py'
    или './workspace/test.py') к абсолютному пути внутри папки workspace.
    """
    p = Path(filename)
    parts = list(p.parts)

    # Отрезаем ведущие точки и префикс workspace, если модель передала путь с ними
    if parts and parts[0] == ".":
        parts = parts[1:]
    if parts and parts[0] == "workspace":
        parts = parts[1:]

    clean_rel = Path(*parts) if parts else Path(".")
    return (WORKSPACE / clean_rel).resolve()


def find_files(*, pattern: str, path: str = ".") -> str:
    base = _resolve_workspace_path(path)
    matches = glob.glob(f"{base}/**/{pattern}", recursive=True)
    unique = sorted(set(matches))
    return "\n".join(unique) if unique else "(no matches)"


def read_file(*, filename):
    #В будущем сделать errors="replace" и бинарное чтение. Всё 3 разные функции
    path = _resolve_workspace_path(filename)
    with open(path, "r", encoding="utf-8", errors="replace") as file:
        return file.read()


def write_file(*, filename, text):
    path = _resolve_workspace_path(filename)
    # Создаем родительские подпапки, если их еще нет
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as file:
        file.write(text)


'''
path - путь к исполняемому файлу относительно папки agent
args - аргументы запускаемой программы в формате list
'''
def run_file(*, path, args=None):
    output = {
        "success": False,
        "output": "",
        "error": ""
    }
    if args is None:
        args = []
    try:
        full_path = os.path.join("./agent", path)
        proc = subprocess.run(
            [full_path] + args,
            capture_output=True,
            text=True,
            check=True,
        )
        output["success"] = True
        try:
            output["output"] = json.loads(proc.stdout)
        except json.JSONDecodeError:
            output["output"] = proc.stdout

    except subprocess.CalledProcessError as e:
        try:
            output["error"] = json.loads(e.stderr)
        except (json.JSONDecodeError, TypeError):
            output["error"] = e.stderr or ""

    except FileNotFoundError as e:
        output["error"] = f"Executable not found: {e}"

    except Exception as e:
        output["error"] = str(e)

    return json.dumps(output, indent=4, ensure_ascii=False)