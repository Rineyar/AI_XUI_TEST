# DAST Tools

Dynamic Application Security Testing tools for identifying potential vulnerabilities in running web applications and services.

Targets are specified using HTTP or HTTPS URLs and must be reachable from the corresponding scanning environment.

Use these tools for network-based security testing with Nuclei and OWASP ZAP. Active scanning may generate intrusive requests and must only be performed against authorized targets.

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

Runs OWASP ZAP in a temporary Docker container, scans the specified URL, returns a JSON report, and attempts to remove the container afterward (including after errors).  
Use when you need DAST analysis of an authorized local web application via ZAP. Docker access must be available to the agent.

Arguments:
- `url` — target HTTP or HTTPS URL, reachable from the agent/ZAP network (the guard restricts DAST targets to local/private addresses)
    - Example: `"http://127.0.0.1:3000"`
- `spider_timeout` — spider scan timeout in seconds; default `600`
    - Example: `600`
- `ascan_timeout` — active scan timeout in seconds; default `1800`
    - Example: `1800`
- `do_ascan` — whether to perform an active scan; default `false`
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
        "target": "http://127.0.0.1:3000",
        "ascan_done": false,
        "alerts_count": 0,
        "alerts": []
    },
    "error": ""
}
```