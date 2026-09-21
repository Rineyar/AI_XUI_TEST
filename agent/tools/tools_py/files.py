import subprocess

__all__ = ["read_file", "write_file", "run_file"]

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

def run_file(path, *args):
    result = subprocess.run(
        [path] + list(args),
        capture_output=True,      
        text=True,                
        check=True,               
    )

    print(result.stdout)
    print(result.stderr)