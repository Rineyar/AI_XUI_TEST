# Unsafe Tools

Low-level tools for direct filesystem access, shell command execution, HTTP communication, and environment inspection.

These tools operate with the permissions of the agent process and are not restricted to the agent's `./workspace`. They do not apply the normal workspace path guards.

Use them when direct system access is required. Their availability depends on the `unsafe_tools` feature.

## unsafe_read_text_file

Reads a text file from the filesystem.

Arguments:
- `path` — path to the file, absolute or relative to the agent's current working directory
    - Example: `"/etc/hostname"`

Returns:
- File contents as a UTF-8 string.
- An error if the file cannot be read or contains invalid UTF-8.

## unsafe_read_bin_file

Reads a file as raw binary data.

Arguments:
- `path` — path to the file
    - Example: `"/tmp/data.bin"`

Returns:
- Raw file contents as a byte array.
- An error if the file cannot be read.

## unsafe_write_text_file

Writes text directly to a filesystem path.

Arguments:
- `path` — destination file path
    - Example: `"/tmp/output.txt"`
- `text` — text to write
    - Example: `"Hello world"`

Returns:
- No value on success.
- An error if the file cannot be written.
- Creates the file if it does not exist and overwrites existing content.
- Parent directories must already exist.

## unsafe_write_bin_file

Writes raw binary data directly to a filesystem path.

Arguments:
- `path` — destination file path
    - Example: `"/tmp/output.bin"`
- `data` — byte array to write

Returns:
- No value on success.
- An error if the file cannot be written.
- Creates or overwrites the destination file.

## unsafe_execute

Executes a shell command directly in the agent's runtime environment using `sh -c`.

Arguments:
- `command` — shell command to execute
    - Example: `"pwd && ls -la"`

Returns:
- `stdout` — standard output as a string
- `stderr` — standard error as a string
- `exitcode` — process exit code, or `null` if unavailable
- An execution error if the shell process cannot be started or waited for.
- Nonzero exit codes are returned normally and do not automatically indicate a tool execution error.
- Output is captured in memory. No command timeout is enforced by the underlying function.

## unsafe_http_get

Performs an HTTP or HTTPS GET request using the agent's network environment.

Arguments:
- `url` — full HTTP or HTTPS URL
    - Example: `"http://127.0.0.1:8080/health"`
- `headers` — optional dictionary of HTTP request headers
    - Example: `{"Accept": "application/json"}`
- `timeout` — optional request timeout in seconds, default `15`
    - Example: `10`

Returns:
- `status` — HTTP response status code
- `body` — response body as a string
- An error if the request cannot be completed.
- HTTP 4xx/5xx statuses do not automatically indicate a tool execution error.
- HTTP redirects are handled according to the client's default redirect policy.

## unsafe_http_post

Performs an HTTP or HTTPS POST request with a text body.

Arguments:
- `url` — full HTTP or HTTPS URL
    - Example: `"http://127.0.0.1:8080/api"`
- `data` — request body as a string
    - Example: `"{\"message\":\"hello\"}"`
- `headers` — optional dictionary of HTTP request headers
    - Example: `{"Content-Type": "application/json"}`
- `timeout` — optional request timeout in seconds, default `15`
    - Example: `10`

Returns:
- `status` — HTTP response status code
- `body` — response body as a string
- An error if the request cannot be completed.
- The request body is sent as provided. JSON serialization is not performed automatically.
- HTTP 4xx/5xx statuses do not automatically indicate a tool execution error.

## unsafe_directory_contents

Lists the immediate contents of a filesystem directory.

Arguments:
- `path` — directory path, absolute or relative
    - Example: `"/tmp"`

Returns:
- Array of file and directory names as strings.
    - Example: `["file.txt", "data", "output.log"]`
- An error if the directory cannot be read.
- Does not recursively traverse subdirectories.
- Does not guarantee the order of returned entries.

## unsafe_dump_env

Returns environment variables from the current agent process.

Arguments:
- none

Returns:
- Dictionary containing environment variable names and values.
- Selected internal configuration variables and API keys are excluded using an explicit blacklist.
- The blacklist does not guarantee that every sensitive variable is removed.
- The result describes the agent process environment, not the environment of a separate Docker sandbox.

## Usage Rules

- These tools operate directly in the agent runtime, not in an isolated Docker sandbox.
- Filesystem access is limited by operating-system permissions rather than workspace guards.
- Prefer regular guarded tools when they provide the required functionality.
- Use unrestricted operations only when appropriate for the requested task.
- Inspect exit codes, error messages, and HTTP statuses instead of assuming success.
- Do not expose credentials, tokens, or other sensitive environment values in reports.
- Do not assume that commands, dependencies, files, or network endpoints are available.
- Do not confuse these tools with Docker sandbox tools or specialized security scanners.
