__all__ = ["read_file", "write_file"]

def read_file(*, filename):

    path = _resolve_workspace_path(filename)

    print(f"[TOOL] actual path={path}")

    with open(path, "r") as file:
        return file.read()

def write_file(*, filename, text):

    path = _resolve_workspace_path(filename)

    print(f"[TOOL] actual path={path}")

    with open(path, "w") as file:
        file.write(text)

def _resolve_workspace_path(filename):
    from pathlib import Path

    WORKSPACE = Path("../workspace").resolve()

    return (WORKSPACE / filename).resolve()
