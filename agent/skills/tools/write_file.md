# Tool: write_file

**Use when**: You need to create or modify a file with text content, or append data to an existing file.

**Behavior**:
- Creates parent directories automatically
- Can write (overwrite) or append mode
- Returns number of bytes written
- Fails if file is not writable

**Input format**:
```json
{
  "tool": "write_file",
  "args": {
    "filename": "/path/to/file.txt",
    "content": "text content",
    "mode": "write",
    "encoding": "utf-8"
  }
}
```

**Output format**:
```json
{
  "success": true,
  "output": 11,
  "error": null
}
```

**Examples**:
- Write configuration: `{"filename": "config.json", "content": "{}", "mode": "write"}`
- Append to log: `{"filename": "app.log", "content": "entry\n", "mode": "append"}`

**Restrictions**:
- Cannot overwrite system files (/etc, /usr, /bin, /sbin, /boot)
- Cannot write to world-writable directories (/tmp, /var/tmp)
- Cannot write files larger than 10MB

**Common errors**:
- Permission denied → error
- Parent directory doesn't exist → auto-created on write mode
- File is a directory → error