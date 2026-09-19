# Tool: run_file

**Use when**: You need to execute a script or binary, capture its output, and analyze results (security testing, vulnerability scanning, etc.).

**Behavior**:
- Direct execution (never uses shell=True)
- Captures stdout, stderr, and exit code
- Enforces timeout (default 30s, max 300s)
- Output limited to 1MB

**Input format**:
```json
{
  "tool": "run_file",
  "args": {
    "filename": "/usr/bin/python3",
    "args": ["--version"],
    "timeout": 5,
    "stdin": "input data",
    "env": {"VAR": "value"}
  }
}
```

**Output format**:
```json
{
  "success": true,
  "output": {
    "exit_code": 0,
    "stdout": "Python 3.12.0\n",
    "stderr": "",
    "timed_out": false
  },
  "error": null
}
```

**Examples**:
- Run Python script: `{"filename": "/usr/bin/python3", "args": ["script.py"]}`
- Execute binary with args: `{"filename": "/bin/ls", "args": ["-la", "/tmp"]}`
- Custom environment: `{"env": {"PATH": "/custom/bin"}}`

**Security**:
- NEVER uses shell=True (prevents injection)
- Cannot execute from world-writable dirs
- Timeout enforced automatically
- Output truncated at 1MB

**Common errors**:
- File not found → error
- Permission denied → error
- Timeout exceeded → `"timed_out": true`
- Exit code non-zero → still returns output (check exit_code)