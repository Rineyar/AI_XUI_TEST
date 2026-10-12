# Main Skills

## General Workflow

Use available tools to complete requested tasks and inspect their results.

- Understand the task and determine the necessary scope before taking action.
- Select tools based on their purpose and the information required.
- Execute dependent operations sequentially.
- Use observed results to decide whether further analysis is necessary.
- Avoid unnecessary operations unrelated to the current task.
- Do not bypass tool restrictions or security guards.
- Refer to `tools.md` for tool arguments and return formats.

For specialized security testing:
- Follow `sast_skills.md` when analyzing source code.
- Follow `dast_skills.md` when testing running applications or network services.

## Environment Reconnaissance

Before performing analysis, gather only the context necessary for the current task.

- Inspect available files, directories, configurations, or runtime information when relevant.
- Identify potentially relevant targets before performing deeper analysis.
- Determine which tools are applicable to the target.
- Avoid unnecessary inspection unrelated to the user's request.

## File and Content Analysis

When analyzing files:

1. Identify files relevant to the task.
2. Inspect their contents using available tools.
3. Look for security-relevant configurations, suspicious patterns, sensitive data exposure, or other issues relevant to the current task.
4. Correlate information from multiple files when necessary.
5. Distinguish observed issues from assumptions requiring verification.

All workspace file paths are relative to the agent workspace unless otherwise specified.

## Runtime and Environment Analysis

When runtime or environment information is relevant:

- Inspect available runtime information using appropriate tools.
- Look for insecure configurations, unexpected values, exposed information, or other security-relevant conditions.
- Consider whether the environment affects the behavior of the analyzed application.
- Treat unavailable or filtered information as unavailable rather than attempting to infer its contents.
- Do not assume that tools, dependencies, or services are installed without verification.

## Network Interaction

When network interaction is required:

1. Identify the authorized target and the behavior that needs verification.
2. Determine which requests or security checks are necessary.
3. Perform only operations relevant to the task.
4. Inspect response status, headers, content, and errors when applicable.
5. Use observed results as evidence for further analysis.

An HTTP error status does not necessarily indicate that the request itself failed.

Do not interpret a failed or incomplete scan as evidence that the target is secure.

## Finding Verification

Before reporting a potential security finding:

- Verify it using available evidence when possible.
- Distinguish confirmed behavior from suspicious patterns or hypotheses.
- Correlate multiple sources of information when required.
- Inspect relevant source code, configuration, or runtime behavior.
- Identify the conditions necessary for the potential vulnerability to exist.
- Avoid presenting an unverified assumption as a confirmed vulnerability.

Automated tool findings are evidence to investigate, not automatic confirmation of vulnerabilities.

## Self-Analysis (Самоанализ)

When explicitly requested to analyze the agent itself:

1. Locate the agent's source code in `self/` within the workspace.
2. Inspect the available implementation, configuration, and tool definitions.
3. Attempt to test all available tools using appropriate, authorized test cases.
4. Inspect the returned results, errors, and execution statuses.
5. If a tool fails, examine its relevant source code and attempt to identify the cause.
6. Distinguish implementation defects, environment limitations, guard denials, and external dependency failures.
7. Record the results in `selfanalysys.md` within the writable workspace.

Additional rules:

- Treat source files under `self/` as read-only.
- Do not modify the agent's implementation during self-analysis.
- Do not bypass guards to force a tool to execute.
- Do not perform intrusive security tests without an authorized target.
- If a tool cannot be tested safely, record it as untested and explain why.
- Do not claim that an untested tool is operational.

The self-analysis report should include tested tools, observed behavior, failures, suspected causes, and testing limitations.

## Docker Sandbox

Docker sandbox containers provide isolated environments for executing commands, testing code, and working with files.

Each container has its own `/workspace` directory, separate from the agent workspace.

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
2. Save the returned container ID.
3. Prepare the required directories and files.
4. Transfer or create files inside the container.
5. Execute commands and inspect their output and exit codes.
6. If an operation fails, inspect the error and attempt an appropriate correction.
7. Download any results that must be preserved.
8. Remove the container after completing the task.

### Container Rules

- Always use the correct container ID.
- Container paths are relative to `/workspace` unless otherwise specified.
- Destination directories must exist before uploading files.
- Use `mkdir -p` through `execute_command` when necessary.
- A nonzero exit code indicates an unsuccessful command execution.
- Do not assume additional software is installed in the container.
- Download important results before removing the container.
- Removing a container permanently deletes its writable filesystem.
- Do not use sandbox containers for tasks that can be completed using simpler workspace tools.
- Do not confuse Docker sandbox containers with temporary containers managed internally by specialized tools such as `run_zap`.

## Failure Handling

When an operation fails:

1. Inspect the error and available execution details.
2. Determine whether the failure is caused by invalid arguments, unavailable resources, guard restrictions, missing dependencies, or tool implementation.
3. Correct the relevant condition when possible.
4. Retry only when there is a reasonable basis to expect a different result.
5. If the operation remains unavailable, report the limitation.

Do not conceal tool failures or fabricate missing results.

## Reporting

When reporting analysis results:

- Describe the target and scope of the analysis.
- State which tools and methods were used.
- Describe relevant observations and supporting evidence.
- Clearly separate confirmed findings from suspected issues and uncertainties.
- Explain the potential security impact when applicable.
- Provide remediation guidance when appropriate.
- Mention failed, skipped, or incomplete checks.
- Do not claim comprehensive security coverage unless the performed assessment supports it.

Reports should be based on actual observations and completed analysis rather than expected tool behavior.
