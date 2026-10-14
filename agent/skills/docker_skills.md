## Docker Sandbox

Docker sandbox tools provide isolated environments for executing commands, testing code, and working with files.

The agent's `./workspace` and each sandbox container's `/workspace` are separate filesystems. Files are not shared automatically and must be transferred using the appropriate tools.

Use Docker sandbox tools when a task requires code execution, isolated testing, additional dependencies, or operations that should not run directly in the agent environment.

### Available Operations

- `create_container` — create an isolated container.
- `execute_command` — execute shell commands inside a running container.
- `send_path` — transfer existing workspace files or directories into a container.
- `send_text` — create text files inside a container.
- `read_container_file` — inspect text files inside a container.
- `download_path` — retrieve files and directories from a container.
- `remove_container` — stop and remove a container.

### Container Workflow

1. Create a container using `create_container`.
2. Save the returned container ID for subsequent operations.
3. Prepare the required directories and dependencies.
4. Transfer workspace files using `send_path` or create files using `send_text`.
5. Execute commands using `execute_command`.
6. Inspect command output, errors, and exit codes.
7. If an operation fails, identify the cause and correct it when possible.
8. Download results that must be preserved using `download_path`.
9. Remove the container using `remove_container` after completing the task.

### Filesystem Rules

- The agent workspace and container workspaces are independent.
- Agent workspace paths must be relative to `./workspace`.
- Container file operations use paths relative to the container's `/workspace` unless otherwise specified.
- Destination directories must exist before transferring files.
- Use `mkdir -p` through `execute_command` when necessary.
- Files created or modified inside a container are not automatically available in the agent workspace.
- Download important results before removing the container.
- Removing a container permanently deletes its writable filesystem unless the data is stored in a persistent volume.

### Execution Rules

- Always use the container ID returned by `create_container`.
- Do not assume additional software or dependencies are installed in a container.
- Check command exit codes and error output before treating an operation as successful.
- A nonzero exit code indicates an unsuccessful command execution.
- Do not repeatedly execute failing commands without identifying a plausible correction.
- Do not use sandbox containers for tasks that can be completed using simpler workspace tools.
- Do not confuse Docker sandbox containers with temporary containers managed internally by specialized tools such as `run_zap`.
- Do not manually manage containers owned by specialized tools unless explicitly required.

### Failure Handling

When a Docker operation fails:

1. Inspect the tool response, command exit code, and available error output.
2. Determine whether the failure is caused by invalid paths, missing dependencies, container state, guard restrictions, or execution errors.
3. Correct the relevant condition when possible.
4. Retry only when there is a reasonable basis to expect a different result.
5. Preserve available diagnostic information before removing the container.
6. Report unresolved failures without fabricating results.

A failed operation does not automatically require recreating the container. Reuse an existing container when it remains operational and the issue can be corrected.
