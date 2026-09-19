# Tool: read_file

**Use when**: You need to read and analyze text file contents for security analysis, configuration review, or log inspection.

**Behavior**:
- Reads text files with automatic encoding detection
- Supports pagination (offset/limit for large files)
- Files larger than 1MB are truncated with warning
- Binary files return error

**Input format**:
```json
{
  "tool": "read_file",
  "args": {
    "filename": "/path/to/file.txt",
    "offset": 1,
    "limit": 100
  }
}
```

**Output format**:
```json
{
  "success": true,
  "output": "file contents as string",
  "error": null
}
```

**Examples**:
- Read config file: `{"filename": "/etc/ssh/sshd_config"}`
- Read log (first 50 lines): `{"filename": "/var/log/auth.log", "limit": 50}`
- Read log (skip first 1000, read 100): `{"filename": "/var/log/auth.log", "offset": 1000, "limit": 100}`

**Security analysis**:
- Check for hardcoded credentials in config files
- Analyze logs for suspicious activity (failed logins, errors)
- Review shell scripts for dangerous patterns
- Inspect environment configuration files

**Common errors**:
- File not found → error
- Permission denied → error
- Binary file detected → error
- File too large (>1MB) → truncated with warning

**Performance tips**:
- Use `offset` and `limit` for large log files instead of reading all at once
- Check `exists` first to verify file is readable
- For very large files, read in chunks with multiple calls