# Available Tools
You allowed to use tools only in ./workspace

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
/workspace/app.py
/workspace/lib/util.py
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


## make_request
Performs an HTTP/HTTPS request.  
Use when the task requires interacting with a remote HTTP service.

Arguments:
- `url` — full request URL
    - Example: `"https://api.example.com/v1/items"`
- `req_type` — request type: `"get"` or `"post"`
    - Example: `"get"`
- `post_data` — POST request body, sent as `application/x-www-form-urlencoded`. Use only for POST.
    - Example: `{ "user": "alice", "password": "secret" }`
- `get_params` — query parameters for a GET request. Use only for GET.
    - Example: `{ "q": "python", "limit": "10" }`

Returns:
- JSON string with fields `success`, `output`, `error`
    - For GET: `output` contains the HTTP status code
        - Example: `{"success": true, "output": 200, "error": ""}`
    - For POST: `output` contains the parsed JSON response
        - Example: `{"success": true, "output": {"token": "abc123"}, "error": ""}`

## dump_env
Shows available environment variables.  
Use when runtime environment information is relevant to the task.

Arguments:
- none

Returns:
- JSON string with fields `success`, `output`, `error`
- `output` — JSON string with environment variables
- if any key contains `PASSWORD`, `TOKEN`, or `SECRET`, the environment is not returned and `success=false`

Example:
```json
{
    "success": true,
    "output": "{\n    \"PATH\": \"/usr/local/bin:/usr/bin:/bin\",\n    \"HOME\": \"/home/user\"\n}",
    "error": ""
}
```

## run_bandit
Runs Bandit — a SAST tool for analyzing Python code security.  
Use when you need to find potential vulnerabilities in Python code.

Arguments:
- `targets` — list of files/directories to analyze
    - Example: `["."]`
- `recursive` — whether to discover files recursively
    - Example: `true`
- `config_file` — optional path to a Bandit config file
    - Example: `"bandit.yaml"`
- `agg_type` — aggregation type: `"vuln"`, `"file"`, `"baseline"`
    - Example: `"vuln"`
- `sev_level` — severity level filter: `"LOW"`, `"MEDIUM"`, `"HIGH"`
    - Example: `"LOW"`
- `conf_level` — confidence level filter: `"LOW"`, `"MEDIUM"`, `"HIGH"`
    - Example: `"LOW"`

Returns:
- JSON string with fields `success`, `output`, `error`
- `output` contains:
    - `result` — discovered issues
    - `skipped` — skipped files

Example:
```json
{
    "success": true,
    "output": {
        "result": [],
        "skipped": []
    },
    "error": ""
}
```

## run_semgrep
Runs Semgrep — a SAST tool for code analysis.  
Always appends `p/security-audit` and `p/secrets` to the provided configs.

Arguments:
- `targets` — list of targets to analyze
    - Example: `["."]`
- `configs` — list of Semgrep configs
    - Example: `["p/python", "p/rust"]`
- `timeout` — per-file analysis timeout in seconds
    - Example: `5`

Returns:
- JSON string with fields `success`, `output`, `error`
- `output` — parsed JSON report from Semgrep

Example:
```json
{
    "success": true,
    "output": {
        "results": [],
        "errors": []
    },
    "error": ""
}
```

## run_nuclei
Runs Nuclei — a DAST tool for checking an HTTP target against templates.  
Use when you need to scan a URL for known vulnerabilities/templates.

Arguments:
- `target_url` — target URL. Only HTTP protocol is allowed
    - Example: `"http://127.0.0.1:8080"`
- `template` — path to a template from the `agent` folder, `.yaml` file. If not specified, Nuclei's built-in templates are used.
    - Example: `"my-template.yaml"`
- `severity` — severity of threats to search for. Default is `"info,low,medium,high,critical"`
    - Example: `"medium,high,critical"`

Returns:
- JSON string with fields `success`, `output`, `error`
- `output` — list of parsed JSON lines from Nuclei output

Example:
```json
{
    "success": true,
    "output": [
        {
            "template-id": "example",
            "info": {
                "name": "Example finding",
                "severity": "medium"
            }
        }
    ],
    "error": ""
}
```

## run_zap

Runs OWASP ZAP in Docker, scans the specified URL, and returns a JSON report.  
Use when you need DAST analysis of a web application via ZAP.

Arguments:
- `url` — target URL
    - Example: `"http://host.docker.internal:3000"`
- `spider_timeout` — spider scan timeout in seconds, default `600`
    - Example: `600`
- `ascan_timeout` — active scan timeout in seconds, default `1800`
    - Example: `1800`
- `do_ascan` — whether to perform an active scan, default `false`
    - Example: `false`

Returns:
- JSON string with fields `success`, `output`, `error`
- `output` contains:
    - `target` — target
    - `ascan_done` — whether active scan was performed
    - `alerts_count` — number of alerts
    - `alerts` — list of alerts

Example:
```json
{
    "success": true,
    "output": {
        "target": "http://host.docker.internal:3000",
        "ascan_done": false,
        "alerts_count": 0,
        "alerts": []
    },
    "error": ""
}
```