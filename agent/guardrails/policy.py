from pathlib import Path


WORKSPACE = Path("../workspace").resolve()

ALLOWED_TOOLS = {
    "read_file",
    "find_files",
    "directory_contents",
}

TOOLS_IN_TEST = {
    "run_bandit",
    "run_semgrep",
    "read_file"
}
