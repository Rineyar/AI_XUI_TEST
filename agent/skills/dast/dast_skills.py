__tool_meta__ = {
    "run_nuclei": {
        "description": (
            "Run Nuclei by using URL. Return list of found vulnerabilities with template_id, name, severity, matched_at."
        )
    },
    "run_zap": {
        "description": (
            "Run OWASP ZAP in Docker, scan URL with Spider. Return alerts with name, risk, url, param, evidence."
        )
    },
}

SPEC_NUCLEI: dict[str, Any] = {
    "name": "run_nuclei",
    "description": __tool_meta__["run_nuclei"]["description"],
    "parameters": {
        "type": "object",
        "properties": {
            "target_url": {
                "type": "string",
                "description": "URL of target, must starts with http:// or https://",
            },
            "template": {
                "type": "string",
                "description": "Way to .yaml inside of ./agent/.",
            },
            "severity": {
                "type": "string",
                "description": "List: info,low,medium,high,critical",
            },
        },
        "required": ["target_url"],
        "additionalProperties": False,
    },
}

SPEC_ZAP: dict[str, Any] = {
    "name": "run_zap",
    "description": __tool_meta__["run_zap"]["description"],
    "parameters": {
        "type": "object",
        "properties": {
            "url": {
                "type": "string",
                "description": "URL of target, must starts with http:// or https://",
            },
            "spider_timeout": {
                "type": "integer",
                "description": "Timeout of spider, seconds.",
                "default": 600,
            },
        },
        "required": ["url"],
        "additionalProperties": False,
    },
}

DAST_SKILLS: dict[str, dict[str, Any]] = {
    "run_nuclei": {"fn": run_nuclei, "spec": SPEC_NUCLEI, "kind": "dast"},
    "run_zap":    {"fn": run_zap,    "spec": SPEC_ZAP,    "kind": "dast"},
}