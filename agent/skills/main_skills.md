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

### Purpose

Self-analysis is a functional diagnostic procedure intended to verify that the agent, its tools, runtime environment, and integrations operate correctly.

Self-analysis is NOT a security vulnerability assessment of the agent's own source code.

### Workflow

When explicitly requested to perform self-analysis:

1. Locate the agent's source code in `self/` within the workspace.
2. Inspect the available implementation, tool definitions, configurations, and relevant dependencies.
3. Attempt to execute all available tools using appropriate and authorized test cases.
4. Inspect actual tool results, errors, execution statuses, and observable effects.
5. Verify that each tool performs its intended operation, rather than relying solely on reported success.
6. If a tool fails or behaves incorrectly, examine its relevant source code and configuration to identify the probable cause.
7. Distinguish between implementation defects, environment limitations, guard denials, missing dependencies, and external service failures.
8. Record the test results and diagnostic findings in `selfanalysys.md` within the writable workspace.

### Testing Rules

- Treat all source files under `self/` as read-only.
- Do not modify the agent's implementation during self-analysis.
- Create temporary test files only in writable locations.
- Use isolated test environments when execution or file modification is necessary.
- Do not bypass runtime guards or security restrictions.
- Do not perform intrusive security testing without an authorized target.
- Do not assume a tool works merely because it is registered or callable.
- Verify actual results whenever possible.
- If a tool cannot be tested safely or its dependencies are unavailable, mark it as untested and explain why.
- Do not claim that an untested or failed tool is operational.
- Clean up temporary test resources and containers when finished.

### Tool Evaluation

Classify each tested tool as:

- **PASS** — The tool executed successfully and its expected behavior was verified.
- **FAIL** — The tool failed, produced incorrect results, or did not perform its intended operation.
- **BLOCKED** — Execution was denied by a guard or another security restriction.
- **UNTESTED** — Testing could not be completed due to unavailable dependencies, missing test conditions, or safety constraints.

A successful function call alone is insufficient for PASS if the expected operation cannot be verified.

### Reporting

Write the results to `selfanalysys.md`.

The report should include:

- Tested tools and their status.
- Test operations performed.
- Expected and observed behavior.
- Relevant errors and execution details.
- Probable causes of failures, supported by available evidence.
- Environment and dependency limitations.
- Recommendations for correcting identified problems.
- A summary of operational and non-operational tools.

Do not report suspected causes as confirmed defects without sufficient evidence.

The primary objective is to determine which agent capabilities work correctly, which do not, and why.
