# Tool: get_env

**Use when**: You need to inspect environment variables for configuration, debugging, or security analysis.

**Behavior**:
- Returns single variable value or all variables
- Automatically filters sensitive values (API_KEY, SECRET, PASSWORD, TOKEN, etc.)
- Fast synchronous operation

**Input format**:
```json
{
  "tool": "get_env",
  "args": {
    "name": "HOME"
  }
}
```

**Output format**:
```json
{
  "success": true,
  "output": "/root",
  "error": null
}
```

**Examples**:
- Get HOME directory: `{"name": "HOME"}`
- Get PATH to see available binaries: `{"name": "PATH"}`
- Get all environment: `{}` (returns filtered object)
- Check for API key presence: `{"name": "OPENAI_API_KEY"}`

**Sensitive variables (filtered)**:
- API_KEY, SECRET, PASSWORD, TOKEN
- AUTH, CREDENTIAL, PRIVATE_KEY
- AWS_SECRET, GCP_KEY, etc.

**Common errors**:
- Variable not set → `"output": null`
- Empty name with no permission → error

**Security analysis**:
- Exposed secrets in environment = security vulnerability
- Missing environment variables may indicate misconfiguration
- PATH manipulation can indicate compromise