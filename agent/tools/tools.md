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
