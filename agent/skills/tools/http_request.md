# Tool: http_request

**Use when**: You need to make HTTP/HTTPS requests to test APIs, download resources, or check web endpoints.

**Behavior**:
- Supports GET, POST, PUT, PATCH, DELETE methods
- Automatic redirect following (default: 5 redirects)
- Request/response headers and body handling
- Response body limited to 10MB

**Input format**:
```json
{
  "tool": "http_request",
  "args": {
    "url": "https://api.example.com/endpoint",
    "method": "GET",
    "headers": {"Content-Type": "application/json"},
    "body": "{\"key\": \"value\"}",
    "timeout": 30,
    "follow_redirects": true
  }
}
```

**Output format**:
```json
{
  "success": true,
  "output": {
    "status_code": 200,
    "status_text": "OK",
    "headers": {"content-type": "application/json"},
    "body": "{\"result\": \"success\"}",
    "elapsed_ms": 150
  },
  "error": null
}
```

**Examples**:
- GET request: `{"url": "https://api.github.com/user", "method": "GET"}`
- POST with JSON: `{"url": "...", "method": "POST", "body": "{\"a\":1}"}`
- Custom headers: `{"headers": {"Authorization": "Bearer token"}}`

**Security**:
- Cannot access private IP ranges (10.x, 172.16.x, 192.168.x, 127.x.x)
- Prevents SSRF attacks by blocking internal endpoints
- Response body truncated at 10MB