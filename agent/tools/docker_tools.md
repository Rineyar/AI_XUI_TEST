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
