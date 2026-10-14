# Main Tools

General-purpose tools for working with files, inspecting the runtime environment, and making HTTP requests.

File operations are restricted to the agent's `./workspace`. Pass paths relative to that workspace (for example, `"src/main.py"`), not absolute paths.

Use these tools for basic workspace operations, environment inspection, and HTTP communication.

## find_files
Searches for files by pattern inside the workspace.  
Use when you need to find files by name or extension.

Arguments:
- `pattern` — glob search pattern
    - Example: `"*.py"`
- `path` — path relative to the workspace, default `"."`
    - Example: `"."`

Returns:
- a string with a list of found files separated by newlines
- if nothing is found — `"(no matches)"`

Example:
```text
app.py
lib/util.py
```

or:

```text
(no matches)
```

## read_file
Reads a text file from the workspace.
Use when file contents are required for the current task.

Arguments:
- `filename` — path relative to the workspace
    - Example: `"file.txt"`

Returns:
- file contents
    - Example: `"Lorem ipsum"`

## write_file
Writes text to a file in the workspace.
Use when the task requires creating or modifying a file.

Arguments:
- `filename` — path relative to the workspace
    - Example: `"file.txt"`
- `text` — content to write
    - Example: `"Lorem ipsum"`

Returns:
- none

## directory_contents
Returns files and directories in the specified workspace folder.  
Use when you need to inspect directory contents.

Arguments:
- `path` — path relative to the workspace, default `"."`
    - Example: `"."`

Returns:
- JSON string with a list of objects containing `kind`, `size`, `name`
    - Example: `[{"kind":"FILE","size":123,"name":"app.py"}]`


## http_request
Performs a single HTTP/HTTPS GET or POST request.  
Use when you need to inspect an HTTP endpoint, including its response status, headers, and body.

Arguments:
- `url` — full HTTP or HTTPS URL (required)
    - Example: `"http://127.0.0.1:8080/health"`
- `req_type` — request method: `"get"` or `"post"` (required)
    - Example: `"get"`
- `post_data` — optional dictionary of POST form fields, sent as `application/x-www-form-urlencoded`; use for POST only
    - Example: `{ "name": "example" }`
- `get_params` — optional dictionary of query parameters; use for GET only
    - Example: `{ "q": "python", "limit": "10" }`

Returns:
- JSON string with fields `success`, `output`, `error`
- On a completed request, `output` contains:
    - `status_code` — HTTP response status
    - `headers` — response headers
    - `body` — first 10,000 characters of the response body, as text
    - `truncated` — `true` if the body was longer than 10,000 characters
- HTTP 4xx/5xx responses are valid HTTP responses and do not by themselves set `success=false`.
- Network failures and timeouts set `success=false` and describe the error in `error`.
- Automatic redirects are disabled. The connection and read timeouts are 5 and 15 seconds, respectively.

Example:
```json
{
    "success": true,
    "output": {
        "status_code": 404,
        "headers": {"Content-Type": "text/plain"},
        "body": "Not found",
        "truncated": false
    },
    "error": ""
}
```

## dump_env

Returns available environment variables as a JSON-formatted string.

Use when you need to inspect the agent's environment configuration.

Arguments:
- none

Returns:
- JSON string with fields `success`, `output`, `error`
- On success, `output` contains a JSON-formatted string of environment variable names and values.
- If any environment variable name contains a blacklisted string (currently `"MAIN_MODEL_API_KEY"`), the tool returns `success=false` instead of exposing the environment.
- On failure, `error` contains `"Could not get the environment."`.
- Individual sensitive variables are not filtered or redacted. Do not expose the returned values in user-facing reports.

Example:
```json
{
    "success": true,
    "output": "{\n    \"PATH\": \"/usr/local/bin:/usr/bin\",\n    \"HOME\": \"/agent\"\n}",
    "error": ""
}
```
