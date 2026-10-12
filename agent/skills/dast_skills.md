# DAST Skill

## Purpose

Dynamic Application Security Testing (DAST) identifies potential security weaknesses by interacting with a running application.

Use DAST to examine HTTP/HTTPS services within the authorized testing scope.

DAST findings are potential vulnerabilities until supported by sufficient evidence.

## Available Tools

- `http_request` — inspect HTTP responses and check endpoint availability.
- `run_nuclei` — perform template-based security checks.
- `run_zap` — crawl web applications and optionally perform active security testing.

Refer to `tools.md` for exact tool arguments and return formats.

## General Workflow

### 1. Identify the Target

Before testing:

- Determine the target URL and relevant endpoints.
- Check that the target belongs to the authorized testing scope.
- Identify whether the application uses HTTP or HTTPS.
- Do not attempt to bypass guard restrictions.

### 2. Initial HTTP Inspection

Use `http_request` to inspect the application before launching scanners.

Examine:

- HTTP status codes.
- Response headers.
- Response body.
- Redirect responses.
- Server error messages.

An HTTP response such as `403`, `404`, or `500` does not mean that the request failed to execute.

### 3. Template-Based Scanning

Use `run_nuclei` to check for known security weaknesses.

Prefer templates relevant to the identified technology or suspected vulnerability.

Nuclei uses its installed templates from `NUCLEI_TEMPLATES_DIR`. Template paths are relative to that directory, not the agent workspace.

The default scan timeout is 120 seconds.

A timeout or scanner error does not prove that the application is secure.

### 4. Web Application Analysis

Use `run_zap` when application crawling or more comprehensive web testing is required.

Start with passive discovery using Spider (`do_ascan=false`).

Use Active Scan (`do_ascan=true`) only when active testing is explicitly permitted and appropriate for the target.

Active scanning may generate intrusive requests or modify application state.

The ZAP tool manages its temporary Docker container. Do not independently create another ZAP container.

### 5. Analyze Results

For every reported finding:

- Identify the affected endpoint or component.
- Record the scanner and relevant evidence.
- Determine the reported severity.
- Compare findings from different tools when useful.
- Distinguish confirmed vulnerabilities from unverified scanner findings.
- Avoid reporting duplicate findings as separate vulnerabilities.

A successful scan with zero findings does not guarantee the absence of vulnerabilities.

## Failure Handling

If a tool fails:

- Inspect the returned `error` field.
- Distinguish network failures, timeouts, scanner errors, and guard denials.
- Do not interpret incomplete scans as successful security assessments.
- Do not repeatedly retry the same failing operation without changing the relevant conditions.

## Reporting

Summarize the assessment with:

- Tested targets and scope.
- Tools used.
- Completed and incomplete checks.
- Identified potential vulnerabilities.
- Supporting evidence and severity.
- Limitations of the assessment.

Clearly separate observations, suspected vulnerabilities, and confirmed findings.
