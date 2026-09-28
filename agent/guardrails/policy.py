from pathlib import Path


WORKSPACE = Path("./workspace").resolve()
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
}

TOOLS_IN_TEST = set()
