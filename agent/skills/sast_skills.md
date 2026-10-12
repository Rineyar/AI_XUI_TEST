# SAST Skill

## Purpose

Static Application Security Testing (SAST) identifies potential security vulnerabilities by analyzing source code without executing the application.

Use SAST to investigate insecure coding patterns, dangerous operations, configuration weaknesses, and potential vulnerabilities.

Automated scanner findings must be reviewed against the actual source code before drawing conclusions.

## Available Tools

- `directory_contents` — inspect the structure of the codebase.
- `find_files` — locate source files and relevant configurations.
- `read_file` — inspect source code and verify findings.
- `run_bandit` — analyze Python code for potential security issues.
- `run_semgrep` — analyze supported programming languages using security rules.

Refer to `tools.md` for exact arguments and return formats.

## General Workflow

### 1. Explore the Codebase

Before running security scanners:

- Identify the project structure.
- Determine the programming languages and frameworks.
- Locate relevant source code and configuration files.
- Identify potential entry points and security-sensitive components.
- Determine the appropriate scope for analysis.

Pay particular attention to authentication, authorization, input processing, file operations, command execution, database access, and external communications.

### 2. Run Automated Security Analysis

**Bandit**

If Python source code is present, use `run_bandit` to identify potential Python security weaknesses.

Examine detected issues, severity levels, confidence levels, affected files, and skipped files.

**Semgrep**

Use `run_semgrep` to analyze the relevant source code.

The default security rulesets `p/security-audit` and `p/secrets` are always included.

Additional configurations may be selected when relevant to the technologies being analyzed.

Use workspace-relative paths when specifying scan targets and local configuration files.

### 3. Analyze Scanner Results

Review the reports produced by both scanners.

For each relevant finding:

- Identify the affected file and source location.
- Understand the reported vulnerability.
- Examine the reported severity and confidence.
- Identify duplicate or overlapping findings.
- Determine whether the issue requires further investigation.

Do not assume that every scanner finding represents a real vulnerability.

### 4. Verify Findings Against Source Code

Use `read_file` to inspect the code associated with reported findings.

Investigate:

- The potentially vulnerable operation.
- The origin and handling of relevant data.
- Existing input validation and security controls.
- The surrounding execution context.
- Conditions required for exploitation.

Determine whether the scanner finding is supported by the source code.

If evidence is insufficient, classify the finding as unverified rather than confirmed.

### 5. Perform Additional Manual Analysis

Do not rely exclusively on automated scanner results.

Inspect security-sensitive components for weaknesses that automated tools may overlook, including:

- Authentication and authorization logic.
- Trust boundaries and input validation.
- Injection vulnerabilities.
- Unsafe file and path handling.
- Insecure cryptographic operations.
- Sensitive information exposure.
- Dangerous system command execution.
- Security-related configuration mistakes.

Focus on realistic vulnerabilities supported by observable code behavior.

Avoid speculative findings without technical evidence.

### 6. Correlate Findings

Combine results from automated scanning and source code inspection.

- Merge duplicate findings.
- Compare related findings from different scanners.
- Distinguish confirmed, suspected, and rejected findings.
- Assess potential impact based on the actual code context.
- Identify relevant vulnerability classifications, such as CWE, when supported by the evidence.

Scanner severity should not automatically be treated as the final risk assessment.

## Failure Handling

If a scanner fails:

- Inspect the returned error information.
- Identify whether the failure is caused by configuration, unavailable rules, invalid targets, or execution errors.
- Review skipped and unscanned files.
- Do not interpret incomplete scans as successful security assessments.
- Do not repeatedly retry failed operations without addressing the cause.

A successful scan with zero findings does not guarantee the absence of vulnerabilities.

## Reporting

Prepare a security assessment containing:

- Analyzed project and scope.
- Identified languages and technologies.
- Tools and configurations used.
- Completed, failed, and incomplete checks.
- Confirmed and suspected security findings.
- Affected files and line numbers.
- Supporting technical evidence.
- Severity and potential impact.
- Recommended remediation measures.
- Limitations of the assessment.

Clearly distinguish automated scanner observations from findings verified through source code analysis.

Do not claim that a vulnerability has been confirmed unless sufficient supporting evidence is available.
