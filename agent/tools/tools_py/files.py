__all__ = ["read_file", "write_file"]

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
