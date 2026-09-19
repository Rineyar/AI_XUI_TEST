# AI XUI Testing Agent - Tools Skills

Comprehensive guide to all 8 security testing tools available in the AI XUI Testing Agent.

## Quick Reference

| Tool | Purpose | Complexity | When to Use |
|------|---------|-----------|------------|
| **exists** | Check if file/dir exists | Simple | Before read/write/execute |
| **list_dir** | List directory contents | Medium | Explore file structure |
| **get_env** | Read environment variables | Simple | Check config & secrets |
| **read_file** | Read file contents | Simple | Analyze configs, logs |
| **write_file** | Write/append to files | Simple | Create reports, configs |
| **http_request** | Make HTTP requests | Medium | Test APIs, check endpoints |
| **connectivity** | TCP connectivity test | Simple | Verify network access |
| **run_file** | Execute binaries/scripts | Complex | Run security tools |

## Tool Categories

### 1. File Operations (4 tools)

#### exists
- **When**: Before any file operation
- **Returns**: Existence, type, permissions
- **Example**: `{"path": "/etc/passwd"}`

#### read_file
- **When**: Analyze file contents
- **Returns**: File contents (text only)
- **Example**: `{"filename": "/var/log/auth.log", "limit": 50}`
- **Security**: Check for hardcoded credentials, suspicious patterns

#### write_file
- **When**: Create analysis reports, save findings
- **Returns**: Bytes written
- **Example**: `{"filename": "report.json", "content": "...", "mode": "write"}`
- **Restrictions**: Cannot write to /etc, /usr, /bin, /tmp

#### list_dir
- **When**: Explore directory structure
- **Returns**: Array of files with metadata
- **Example**: `{"path": "/var/log", "filter": "*.log", "recursive": false}`
- **Performance**: Use filter to avoid listing all files

### 2. System Inspection (2 tools)

#### get_env
- **When**: Check environment configuration
- **Returns**: Single var or all vars (filtered)
- **Example**: `{"name": "HOME"}` or `{}`
- **Security**: Filters sensitive variables (API_KEY, SECRET, PASSWORD, TOKEN)

#### run_file
- **When**: Execute security scanning tools
- **Returns**: exit_code, stdout, stderr
- **Example**: `{"filename": "/usr/bin/python3", "args": ["script.py"], "timeout": 30}`
- **Security**: No shell=True, timeout enforced, output limited to 1MB
- **Complexity**: HIGHEST — use with caution

### 3. Network Operations (2 tools)

#### connectivity
- **When**: Test if remote host is reachable
- **Returns**: reachable status, latency
- **Example**: `{"host": "google.com", "port": 443}`
- **Use Case**: Detect network isolation, firewall rules

#### http_request
- **When**: Test APIs, download content
- **Returns**: status_code, headers, body
- **Example**: `{"url": "https://api.github.com", "method": "GET"}`
- **Security**: Blocks private IP ranges (prevents SSRF)

## Execution Flow Example

### Scenario: Audit suspicious process

```
1. get_env (check PATH, HOME, USER)
   ↓
2. exists (/proc/[pid]) → verify process exists
   ↓
3. read_file (/proc/[pid]/cmdline) → check command
   ↓
4. list_dir (/proc/[pid]/fd) → check open files
   ↓
5. run_file (lsof -p [pid]) → detailed analysis
   ↓
6. write_file (report.json) → save findings
```

## Common Patterns

### Reconnaissance
```
exists(path) → list_dir(path) → read_file(file)
```

### Security Audit
```
get_env() → run_file(security_tool) → write_file(report)
```

### Network Check
```
connectivity(host:port) → http_request(url) → analyze response
```

## Error Handling

Every tool returns:
```json
{
  "success": true/false,
  "output": "result or null",
  "error": "error message or null"
}
```

**Always check `success` before using output.**

## Security Guidelines

1. **Privilege Escalation**: Never assume root access. Check with `exists` first.
2. **Timeouts**: Always set reasonable timeouts for `run_file`.
3. **SSRF Prevention**: `http_request` blocks private IPs automatically.
4. **Shell Injection**: `run_file` uses direct execution, never shell=True.
5. **Output Limits**: Large outputs are truncated (1MB for run_file, 10MB for http_request).
6. **Sensitive Data**: Environment variables with KEY/SECRET are filtered by `get_env`.

## Performance Tips

- Use `filter` in `list_dir` instead of listing all and filtering
- Read large files with `offset`/`limit` instead of all at once
- Check `connectivity` before expensive `http_request`
- Batch related operations to minimize tool calls
