import subprocess
import json
import os

__all__ = ["read_file", "write_file", "run_file"]

def read_file(*, filename):

    # path = _resolve_workspace_path(filename)

    #В будущем сделать errors="replace" и бинарное чтение. Всё 3 разные функции
    with open(filename, "r", encoding="utf-8") as file:
        return file.read()

def write_file(*, filename, text):

    # path = _resolve_workspace_path(filename)

    with open(filename, "w") as file:
        file.write(text)

def _resolve_workspace_path(filename):
    from pathlib import Path

    WORKSPACE = Path("../workspace").resolve()

    return (WORKSPACE / filename).resolve()

'''
path - путь к исполняемому файлу относительно папки agent
args - аргументы запускаемой программы в формате list
'''
def run_file(*, path, args = None):
    output = {
            "success" : False,
            "output" : "",
            "error" : ""
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