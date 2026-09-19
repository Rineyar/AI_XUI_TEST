# Tool: exists

**Use when**: You need to verify if a file or directory exists before performing other operations (read, write, execute).

**Behavior**:
- Returns detailed existence information including type and permissions
- Fast operation — use before expensive file operations
- Can distinguish files, directories, symlinks, sockets, and devices

**Input format**:
```json
{
  "tool": "exists",
  "args": {
    "path": "/path/to/check"
  }
}
```

**Output format**:
```json
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

**Examples**:
- Check if config file exists before reading
- Verify log directory exists before listing contents
- Detect if a path is a symlink (potential security risk)

**Common errors**:
- Path doesn't exist → `"exists": false`
- Permission denied → `"exists": true` but `"readable": false`

**Security notes**:
- Symlinks may point outside expected directory (sanitize before use)
- World-writable directories are potential security risks