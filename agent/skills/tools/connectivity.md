# Tool: connectivity

**Use when**: You need to test if a remote host is reachable on a specific TCP port (basic network connectivity check).

**Behavior**:
- Performs TCP connect to host:port
- Returns reachability status and latency
- No ICMP ping (pure TCP check)

**Input format**:
```json
{
  "tool": "connectivity",
  "args": {
    "host": "example.com",
    "port": 443,
    "timeout": 5
  }
}
```

**Output format**:
```json
{
  "success": true,
  "output": {
    "reachable": true,
    "latency_ms": 23.5,
    "error": null
  },
  "error": null
}
```

**Examples**:
- Check HTTPS port: `{"host": "google.com", "port": 443}`
- Check SSH port: `{"host": "192.168.1.1", "port": 22}`
- Check custom service: `{"host": "localhost", "port": 8080, "timeout": 2}`

**Common errors**:
- Host unreachable → `"reachable": false`, `"latency_ms": null`
- Connection timeout → `"reachable": false`
- Invalid host → error

**Security notes**:
- Used to verify external connectivity before HTTP requests
- Helps identifyfirewall restrictions or network isolation
- Rate limited to prevent port scanning abuse