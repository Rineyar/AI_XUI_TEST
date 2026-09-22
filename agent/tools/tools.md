# AI XUI Testing Agent - Tools Reference

## Overview

Tools are Python functions embedded into the Rust binary via `include_dir!` (PyO3). At runtime, an embedded Python interpreter executes them directly — Rust calls Python functions with `kwargs` through `serde_pyobject`. There is **no JSON stdin/stdout protocol** and no external processes.

## Architecture

- Python files live in `agent/tools/tools_py/`
- Each module exposes functions via `__all__`
- `py_env.rs` loads all `.py` files at startup, extracts `__all__` functions into a registry
- `tools.rs` wraps each Python function as a Rig tool (`#[rig_tool]`)
- Guardrails (`tools_guard.py`) run **before** tool execution for path/permission checks

---

## Available Tools

### 1. `read_file` (module: `files`)

**Purpose**: Read contents of a text file from the workspace.

**Signature**:
```python
def read_file(*, filename: str) -> str
```

**Parameters**:
- `filename` (string, required): Path relative to workspace root

**Returns**: File contents as string

**Example call from agent**:
```rust
read_file(filename: String) -> Result<String, ToolExecutionError>
```

---

### 2. `write_file` (module: `files`)

**Purpose**: Write text content to a file in the workspace.

**Signature**:
```python
def write_file(*, filename: str, text: str) -> None
```

**Parameters**:
- `filename` (string, required): Path relative to workspace root
- `text` (string, required): Content to write

**Returns**: None (raises on error)

**Example call from agent**:
```rust
write_file(filename: String, text: String) -> Result<(), ToolExecutionError>
```

---

### 3. `http_request` (module: `http_request`)

**Purpose**: Perform HTTP/HTTPS requests.

**Signature**:
```python
def make_request(*, url: str, req_type: str = "get",
                 post_data: dict | None = None,
                 get_params: dict | None = None) -> str
```
Returns JSON string: `{"success": bool, "output": ..., "error": ...}`

**Parameters**:
- `url` (string, required): Target URL (http/https)
- `req_type` (string, optional): "get" or "post" (default: "get")
- `post_data` (object, optional): JSON body for POST
- `get_params` (object, optional): Query parameters for GET

**Returns**: JSON response as string

**Example call from agent**:
```rust
http_request(
    url: String,
    req_type: String,
    post_data: Option<HashMap<String, String>>,
    get_params: Option<HashMap<String, String>>
) -> Result<String, ToolExecutionError>
```

---

### 4. `dump_env` (module: `dump_env`)

**Purpose**: Dump environment variables (with secret filtering).

**Signature**:
```python
def dump_env() -> str
```
Returns JSON string: `{"success": bool, "output": {...}, "error": ...}`

**Parameters**: None

**Returns**: JSON string with filtered environment

**Example call from agent**:
```rust
dump_env() -> Result<String, ToolExecutionError>
```

---

## Test Tools (Rust-native, not Python)

These are compiled into the binary directly via `#[rig_tool]`:

| Tool | Description |
|------|-------------|
| `sum_i32(a: i32, b: i32)` | Add two signed 32-bit integers with overflow check |
| `sum_i64(a: i64, b: i64)` | Add two signed 64-bit integers with overflow check |
| `sub_i64(a: i64, b: i64)` | Subtract two signed 64-bit integers with overflow check |

---

## Security Model

Security is **not** documented per-tool. It is enforced centrally in `agent/guardrails/tools_guard.py`:
- Path traversal prevention (workspace boundary)
- File read/write guards
- Extensible for new tools

Do not add per-tool security sections — they belong in guardrails.