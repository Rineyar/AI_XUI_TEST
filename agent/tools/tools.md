# Available Tools
File tools operate in the agent's `./workspace`. Pass paths relative to that workspace (for example, `"src/main.py"`), not absolute `/workspace/...` paths. Docker sandbox tools operate in separate containers with their own `/workspace` directories.

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

## run_bandit
Runs Bandit — a SAST tool for analyzing Python code security.  
Use when you need to find potential vulnerabilities in Python code.

Arguments:
- `targets` — optional list of files/directories relative to the agent workspace; default `["."]`
    - Example: `["src"]`
- `recursive` — whether to discover files recursively; default `true`
    - Example: `true`
- `config_file` — optional path to a local Bandit config file, relative to the agent workspace
    - Example: `"bandit.yaml"`
- `agg_type` — aggregation type: `"vuln"`, `"file"`, `"baseline"`; default `"vuln"`
    - Example: `"vuln"`
- `sev_level` — minimum severity: `"LOW"`, `"MEDIUM"`, `"HIGH"`; default `"LOW"`
    - Example: `"LOW"`
- `conf_level` — minimum confidence: `"LOW"`, `"MEDIUM"`, `"HIGH"`; default `"LOW"`
    - Example: `"LOW"`

Returns:
- JSON string with fields `success`, `output`, `error`
- `output` contains:
    - `result` — discovered issues
    - `skipped` — skipped files
- File paths in findings and skipped-file entries are expressed relative to the agent workspace.

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
- `targets` — optional list of files/directories relative to the agent workspace; default `["."]`
    - Example: `["src"]`
- `configs` — optional list of Semgrep Registry rulesets (`p/...`) and/or local configuration files relative to the agent workspace
    - Example: `["p/python", "rules/custom.yaml"]`
- `timeout` — per-file analysis timeout in seconds; default `5`
    - Example: `5`

Returns:
- JSON string with fields `success`, `output`, `error`
- `output` — parsed JSON report from Semgrep; finding paths are relative to the agent workspace

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
Runs Nuclei — a DAST scanner using preinstalled detection templates.  
Use to check an authorized local HTTP/HTTPS target for known security issues.

Arguments:
- `target_url` — target HTTP or HTTPS URL (required; the guard restricts DAST targets to local/private addresses)
    - Example: `"http://127.0.0.1:8080"`
- `template` — optional file or directory **relative to `NUCLEI_TEMPLATES_DIR`** (`/agent/tools/nuclei-templates`); YAML files and template directories are supported. Omit to select the entire installed template collection.
    - Example: `"http/technologies"`
- `severity` — comma-separated severity levels; default `"info,low,medium,high,critical"`
    - Example: `"medium,high,critical"`
- `timeout` — optional maximum duration of the full scan, in seconds; default `120`
    - Example: `120`

Returns:
- JSON string with fields `success`, `output`, `error`
- `output` — list of JSON findings produced by Nuclei (may be empty)
- When the scan fails or exceeds its time limit, `success=false` and `error` explains why.

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

Runs OWASP ZAP in a temporary Docker container, executes a ZAP Automation Framework plan against the specified URL, returns a JSON report, and attempts to remove the container and the generated plan YAML file afterward (including after errors).  
Use when you need DAST analysis of an authorized local web application via ZAP. Docker access must be available to the agent.

Arguments:
- `target_url` — target HTTP or HTTPS URL, reachable from the agent/ZAP network (the guard restricts DAST targets to local/private addresses)
    - Example: `"http://127.0.0.1:5000"`
- `plan_template` — path to the Automation Framework YAML plan template; the `{{TARGET}}` placeholder is replaced with `target_url`. Default: `"plan.yaml.tpl"`
    - Example: `"plan.yaml.tpl"`
- `timeout` — maximum time to wait for the automation plan to finish, in seconds; default `1800`
    - Example: `1800`

Returns:
- JSON string with fields `success`, `output`, `error`
- `output` contains:
    - `target` — scan target
    - `plan_id` — ID of the launched ZAP automation plan
    - `alerts_count` — number of alerts
    - `alerts` — list of alerts

Example:
```json
{
    "success": true,
    "output": {
        "target": "http://127.0.0.1:5000",
        "plan_id": "1",
        "alerts_count": 0,
        "alerts": []
    },
    "error": ""
}
```

# Docker Sandbox Tools

Docker sandbox tools allow you to create isolated containers, transfer files, execute shell commands, and retrieve results.

Each container has its own `/workspace` directory, separate from the agent's local `./workspace`.

Use the container ID returned by `create_container` for subsequent operations. Remove containers when they are no longer needed.

## create_container

Creates and starts a new Docker sandbox container using the Alpine Linux image. The container remains running until removed.

Use when you need an isolated environment to execute commands or work with files.

Arguments:
- None.

Returns:
- `String` — ID of the created container.

Example:
`"a1b2c3d4e5f6"`

## remove_container

Stops and removes a Docker sandbox container.

Use when you have finished working with a container and no longer need its contents.

Arguments:
- `id` — ID of the container to remove.
    - Example: `"a1b2c3d4e5f6"`

Returns:
- Success with no additional data, or an error if removal fails.

Warning: Removing a container deletes its writable filesystem, including files that have not been downloaded.

## execute_command

Executes a shell command inside an existing, running Docker sandbox container.

Commands are executed using `sh -c` with `/workspace` as the working directory.

Use when you need to inspect files, run programs, install dependencies, or execute tests.

Arguments:
- `id` — ID of the target container.
- `command` — shell command to execute.
    - Example: `"ls -la"`
    - Example: `"python3 --version"`
    - Example: `"cd project && ls"`

Returns:
- Command output, including stdout and stderr.
- Exit code of the executed command.
    - `0` usually indicates success.
    - A nonzero value indicates failure.

The command runs inside the container, not on the agent's host filesystem.

## send_path

Uploads an existing file or directory from the agent workspace into a Docker sandbox container.

Files and directories are automatically packaged into a TAR archive and extracted inside the container.

Use when you need to transfer existing source code, projects, configuration files, or other data.

Arguments:
- `id` — ID of the target container.
- `local_path` — source file or directory relative to the agent's workspace.
    - Example: `"project"`
    - Example: `"main.py"`
- `remote_path` — destination directory relative to `/workspace` inside the container.
    - Example: `""` — container workspace root.
    - Example: `"src"` — `/workspace/src`.

Returns:
- Success with no additional data, or an error.

The source filename or directory name is preserved. The destination directory must already exist.

Example:
- `local_path = "project"`
- `remote_path = ""`
- Result: `/workspace/project/` inside the container.

## send_text

Creates or overwrites a UTF-8 text file directly inside a Docker sandbox container.

The file content is transferred directly without creating an intermediate file in the agent workspace.

Use when you need to write source code, scripts, configuration files, or other text directly into a container.

Arguments:
- `id` — ID of the target container.
- `path` — destination directory relative to `/workspace` inside the container.
    - Example: `""`
    - Example: `"src"`
- `filename` — name of the file to create.
    - Example: `"main.py"`
- `content` — UTF-8 text to write into the file.
    - Example: `"print('Hello, world!')"`

Returns:
- Success with no additional data, or an error.

The destination directory must already exist.

Example:
- `path = "src"`
- `filename = "main.py"`
- Result: `/workspace/src/main.py`.

## download_path

Downloads a file or directory from a Docker sandbox container into the agent workspace.

Data is transferred as a TAR archive and automatically extracted into the destination directory.

Use when you need to retrieve generated files, test reports, logs, or other results.

Arguments:
- `id` — ID of the source container.
- `remote_path` — source file or directory relative to `/workspace` inside the container.
    - Example: `"report.txt"`
    - Example: `"results"`
- `local_path` — destination directory relative to the agent workspace.
    - Example: `""`
    - Example: `"reports"`

Returns:
- Success with no additional data, or an error.

Example:
- `remote_path = "report.txt"`
- `local_path = "reports"`
- Result: `./workspace/reports/report.txt` on the agent side.

## read_container_file

Reads the contents of a UTF-8 text file directly from a Docker sandbox container.

The file is transferred and decoded in memory without saving it to the agent workspace.

Use when you need to inspect source code, configuration files, logs, or command-generated text.

Arguments:
- `id` — ID of the source container.
- `filename` — path to a text file relative to `/workspace` inside the container.
    - Example: `"main.py"`
    - Example: `"logs/output.txt"`

Returns:
- `String` — UTF-8 contents of the requested file.
- An error if the file cannot be downloaded or decoded.

Example:

`print('Hello, world!')`

This tool is intended for text files, not binary data.
