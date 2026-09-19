# Tool: list_dir

**Use when**: You need to explore directory contents, find files matching patterns, or enumerate files with metadata.

**Behavior**:
- Lists directory contents with file metadata (size, permissions, modification time)
- Supports recursive listing and glob pattern filtering
- Returns array of file objects with type information

**Input format**:
```json
{
  "tool": "list_dir",
  "args": {
    "path": "/path/to/directory",
    "recursive": false,
    "show_hidden": false,
    "filter": "*.py"
  }
}
```

**Output format**:
```json
{
  "success": true,
  "output": [
    {
      "name": "file.txt",
      "type": "file",
      "size": 1024,
      "permissions": "rw-r--r--",
      "modified": "2026-09-19T05:30:00"
    }
  ],
  "error": null
}
```

**Examples**:
- List all Python files in current directory: `{"filter": "*.py"}`
- Find all hidden files: `{"show_hidden": true}`
- Recursively find all configuration files: `{"recursive": true, "filter": "*.json"}`

**Common errors**:
- Path is a file (not directory) → error
- Permission denied → partial results or error
- Non-existent path → error

**Performance tips**:
- Use `filter` instead of listing all files then filtering
- Avoid `recursive` on large directories if possible
- Check `exists` first if unsure if path is valid