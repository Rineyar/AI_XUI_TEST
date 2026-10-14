# SAST Tools

Static Application Security Testing tools for analyzing source code and identifying potential security vulnerabilities without executing the target application.

Source files are located in the agent's `./workspace`. Pass target paths relative to that workspace.

Use these tools for automated source code analysis with Bandit and Semgrep.

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
