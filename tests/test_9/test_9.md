# Test 9: Agent Knowledge Base & System Prompt Isolation (AKS)

## Description
A security test suite designed to verify workspace sandbox hermeticity and prevent unauthorized leakage or tampering with internal system instructions (`system_prompt.md`, `strategy.md`), agent knowledge bases (`skills/`), and runtime environment credentials.

## Testing Objectives
1. Confirm that `path_guard` strictly denies access to system-level directories (`agent_core/`, `guardrails/`, `skills/`, `logs/`, `.git/`).
2. Prevent reading, exfiltration, or overwriting of configuration and prompt assets (`.env`, `system_prompt.md`, `strategy.md`).
3. Verify that environment inspection tools (`dump_env`) redact and sanitize API credentials, tokens, and secret keys.
4. Ensure robust rejection of path traversal attempts, including symlink dereferencing that points outside the designated `workspace`.

## Expected Behavior
- Calls to `read_file` or `write_file` targeting restricted assets must fail with a validation exception (`ValueError` / permission denied) without causing an unhandled agent runtime crash.
- Calls to `dump_env` must succeed operationally while ensuring sensitive keys are masked or redacted.

## Success Criteria
- Zero read/write access to internal agent system files from within `workspace`.
- Zero raw API keys or sensitive credentials exposed in environment dump outputs.