# AI XUI Testing Agent - Tools Reference

## Overview

All tools are standalone Python executables that communicate via JSON stdin/stdout. Each tool follows a unified contract for consistency and reliability.

## Communication Protocol

### Request Format (stdin)
```json
{
  "tool": "tool_name",
  "args": {
    "param1": "value1",
    "param2": "value2"
  }
}
```

### Response Format (stdout)
```json
{
  "success": true,
  "output": "result data",
  "error": null
}
```

### Error Response
```json
{
  "success": false,
  "output": null,
  "error": "detailed error message"
}
```

---

## File Operations

### 1. read_file

**Purpose**: Read contents of a text file with automatic encoding detection and size limits.

**Location**: `agent/tools/tools_py/read_file.py`

**Arguments**:
- `filename` (string, required): Path to file (absolute or relative)
- `offset` (integer, optional): Line number to start reading from (1-indexed, default: 1)
- `limit` (integer, optional): Maximum number of lines to read (default: 1000, max: 5000)

**Output**: File contents as string

**Example**:
```json
// Request
{"tool": "read_file", "args": {"filename": "/etc/passwd", "limit": 10}}

// Response
{
  "success": true,
  "output": "root:x:0:0:root:/root:/bin/bash\n...",
  "error": null
}
```

**Security**:
- Files larger than 1MB are truncated
- Binary files return error
- Cannot read from /proc, /sys, or device files

**Complexity**: Simple (already exists, rewrite to contract)

---

### 2. write_file

**Purpose**: Write or append text content to a file.

**Location**: `agent/tools/tools_py/write_file.py`

**Arguments**:
- `filename` (string, required): Path to file (absolute or relative)
- `content` (string, required): Text content to write
- `mode` (string, optional): "write" (default) or "append"
- `encoding` (string, optional): Text encoding (default: "utf-8")

**Output**: Number of bytes written

**Example**:
```json
// Request
{"tool": "write_file", "args": {"filename": "/tmp/test.txt", "content": "Hello world"}}

// Response
{
  "success": true,
  "output": 11,
  "error": null
}
```

**Security**:
- Cannot overwrite system files (/etc, /usr, /bin, etc.)
- Creates parent directories automatically
- Respects file permissions (fails if no write access)

**Complexity**: Simple (already exists, rewrite to contract)

---

### 3. list_dir

**Purpose**: List contents of a directory with file metadata.

**Location**: `agent/tools/tools_py/list_dir.py`

**Arguments**:
- `path` (string, required): Directory path (default: current directory)
- `recursive` (boolean, optional): Include subdirectories (default: false)
- `show_hidden` (boolean, optional): Show hidden files (default: false)
- `filter` (string, optional): Glob pattern to filter results (e.g., "*.py")

**Output**: Array of objects with file metadata:
```json
{
  "name": "filename.txt",
  "type": "file|directory|symlink",
  "size": 1024,
  "permissions": "rw-r--r--",
  "modified": "2026-09-19T05:30:00"
}
```

**Example**:
```json
// Request
{"tool": "list_dir", "args": {"path": "/var/log", "filter": "*.log"}}

// Response
{
  "success": true,
  "output": [
    {"name": "auth.log", "type": "file", "size": 524288, "permissions": "rw-r-----", "modified": "2026-09-19T05:30:00"},
    {"name": "syslog", "type": "file", "size": 1048576, "permissions": "rw-r-----", "modified": "2026-09-19T05:30:00"}
  ],
  "error": null
}
```

**Complexity**: Medium

---

### 4. exists

**Purpose**: Check if a file or directory exists and determine its type.

**Location**: `agent/tools/tools_py/exists.py`

**Arguments**:
- `path` (string, required): Path to check

**Output**: Object with existence info:
```json
{
  "exists": true,
  "type": "file|directory|symlink|socket|device",
  "readable": true,
  "writable": false
}
```

**Example**:
```json
// Request
{"tool": "exists", "args": {"path": "/etc/passwd"}}

// Response
{
  "success": true,
  "output": {
    "exists": true,
    "type": "file",
    "readable": true,
    "writable": false
  },
  "error": null
}
```

**Complexity**: Simple

---

## System Inspection

### 5. get_env

**Purpose**: Retrieve environment variables.

**Location**: `agent/tools/tools_py/get_env.py`

**Arguments**:
- `name` (string, optional): Variable name (omit to get all variables)

**Output**: 
- If name specified: string value or null
- If no name: object with all environment variables

**Example**:
```json
// Request (single variable)
{"tool": "get_env", "args": {"name": "HOME"}}

// Response
{
  "success": true,
  "output": "/root",
  "error": null
}

// Request (all variables)
{"tool": "get_env", "args": {}}

// Response
{
  "success": true,
  "output": {
    "HOME": "/root",
    "USER": "root",
    "PATH": "/usr/local/sbin:/usr/local/bin:...",
    "..."
  },
  "error": null
}
```

**Security**:
- Filters sensitive variables (API_KEY, SECRET, PASSWORD, TOKEN)
- Returns "[FILTERED]" for sensitive values

**Complexity**: Simple

---

### 6. run_file

**Purpose**: Execute a script or binary with output capture and timeout control.

**Location**: `agent/tools/tools_py/run_file.py`

**Arguments**:
- `filename` (string, required): Path to executable
- `args` (array of strings, optional): Command-line arguments
- `timeout` (integer, optional): Timeout in seconds (default: 30, max: 300)
- `stdin` (string, optional): Input to pass to stdin
- `env` (object, optional): Environment variables to set

**Output**:
```json
{
  "exit_code": 0,
  "stdout": "standard output",
  "stderr": "standard error",
  "timed_out": false
}
```

**Example**:
```json
// Request
{"tool": "run_file", "args": {"filename": "/usr/bin/python3", "args": ["--version"], "timeout": 5}}

// Response
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

**Security**:
- **NEVER** uses `shell=True` (direct execution only)
- Output limited to 1MB (truncated if exceeded)
- Cannot execute from world-writable directories (/tmp)
- Timeout enforced via subprocess timeout

**Complexity**: Complex

---

### 7. connectivity

**Purpose**: Test TCP connectivity to a remote host and port.

**Location**: `agent/tools/tools_py/connectivity.py`

**Arguments**:
- `host` (string, required): Hostname or IP address
- `port` (integer, required): Port number (1-65535)
- `timeout` (integer, optional): Connection timeout in seconds (default: 5, max: 30)

**Output**:
```json
{
  "reachable": true,
  "latency_ms": 23.5,
  "error": null
}
```

**Example**:
```json
// Request
{"tool": "connectivity", "args": {"host": "google.com", "port": 443, "timeout": 5}}

// Response
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

**Security**:
- Uses TCP connect (not ICMP ping)
- Prevents port scanning by limiting concurrent connections

**Complexity**: Simple

---

## Network Operations

### 8. http_request

**Purpose**: Perform HTTP/HTTPS requests.

**Location**: `agent/tools/tools_py/http_request.py`

**Arguments**:
- `url` (string, required): Target URL (http or https)
- `method` (string, optional): HTTP method (default: "GET")
- `headers` (object, optional): Request headers
- `body` (string, optional): Request body (for POST/PUT/PATCH)
- `timeout` (integer, optional): Request timeout in seconds (default: 30, max: 120)
- `follow_redirects` (boolean, optional): Follow redirects (default: true, max: 5)

**Output**:
```json
{
  "status_code": 200,
  "status_text": "OK",
  "headers": {},
  "body": "response body",
  "elapsed_ms": 150
}
```

**Example**:
```json
// Request
{"tool": "http_request", "args": {"url": "https://api.github.com", "method": "GET", "timeout": 10}}

// Response
{
  "success": true,
  "output": {
    "status_code": 200,
    "status_text": "OK",
    "headers": {
      "content-type": "application/json",
      "..."
    },
    "body": "{\"current_user_url\": \"https://api.github.com/user\", ...}",
    "elapsed_ms": 150
  },
  "error": null
}
```

**Security**:
- Response body limited to 10MB (truncated if exceeded)
- Cannot access private IP ranges (10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16, 127.0.0.0/8)
- Prevents SSRF by blocking internal endpoints

**Complexity**: Medium

---

## Removed Tools

The following tools were removed as they were test-only and not part of requirements:

- ~~`sum_i32`~~ — Test tool, removed
- ~~`sum_i64`~~ — Test tool, removed  
- ~~`sub_i64`~~ — Test tool, removed

---

## Implementation Status

| Tool | Status | Priority | Complexity |
|------|--------|----------|------------|
| exists | ✅ Ready | Day 1-2 | Simple |
| list_dir | ✅ Ready | Day 1-2 | Medium |
| get_env | ✅ Ready | Day 1-2 | Simple |
| write_file | ✅ Ready | Day 3-4 | Simple |
| http_request | ✅ Ready | Day 3-4 | Medium |
| connectivity | ✅ Ready | Day 3-4 | Simple |
| run_file | ✅ Ready | Day 5-6 | Complex |
| read_file | ✅ Ready | Day 5-6 | Simple |

**All tools follow the unified JSON contract for seamless integration.**
