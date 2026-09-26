# Available Tools
## read_file
Reads a text file from the workspace.
Use when file contents are required for the current task.

Arguments:
- `filename` — path relative to the workspace

Returns:
- file contents

## write_file
Writes text to a file in the workspace.
Use when the task requires creating or modifying a file.

Arguments:
- `filename` — path relative to the workspace
- `text` — content to write

## http_request
Performs an HTTP or HTTPS request.
Use when the task requires interacting with a remote HTTP service.

Supports:
- GET
- POST
- query parameters
- request body

## dump_env
Returns available environment variables with sensitive values filtered.
Use when runtime environment information is relevant to the task.

## run_bandit
Runs Bandit - a SAST tool for analyzing Python code security.
Discovers target files, runs Bandit tests, and returns discovered issues
and skipped files serialized as a JSON string.

Arguments:
- `targets` - list of files/directories to analyze
- `recursive` - whether to discover files recursively
- `config_file` - optional Bandit config file path
- `agg_type` - aggregation type: "vuln", "file", "baseline"
- `sev_level` - severity level filter: "LOW", "MEDIUM", "HIGH"
- `conf_level` - confidence level filter: "LOW", "MEDIUM", "HIGH"

Supports:
- Returns JSON string with fields `success`, `output` (`result`, `skipped`), `error`

## run_semgrep
Runs Semgrep - a SAST tool for code analysis.
Always appends `p/security-audit` and `p/secrets` to the provided configs.

Arguments:
- `targets` - list of targets (files/dirs) to analyze
- `configs` - list of semgrep configs
- `timeout` - per-file analysis timeout in seconds

Supports:
- Returns JSON string with fields `success`, `output`, `error`