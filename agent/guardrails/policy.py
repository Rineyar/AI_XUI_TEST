from pathlib import Path


WORKSPACE = Path("../workspace").resolve()

ALLOWED_TOOLS = {

}

TOOLS_IN_TEST = {
    "run_bandit",
    "run_semgrep",
    "read_file",
    "directory_contents",
    "write_file",
    "find_files",
    "make_request"
}
