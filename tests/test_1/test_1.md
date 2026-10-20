# Main Project Vulnerabilities

Examples target the local demo at `http://localhost:5000`.

## 1. SQL Injection

**Location:** `app.py`, `/accounts` and `/api/accounts`.
User input is inserted directly into SQL, allowing query conditions to be changed and confidential accounts to be exposed.

**Example:** Enter the following in the account search field:

```sql
' OR 1=1 --
```

The `/accounts` query then returns all accounts, including confidential ones.

**Fix:** Use parameterized queries:

```python
cursor.execute(
    "SELECT * FROM accounts WHERE customer_name LIKE ? AND is_confidential = 0",
    (f"%{search_term}%",),
)
```

## 2. OS Command Injection

**Location:** `/system-diagnostics` and `/api/ping`.
The `host` parameter is inserted into a command executed with `shell=True`. Additional commands run with the application's OS privileges. The current Dockerfile runs the application as root inside the container.

**Example:** Enter this in the diagnostics host field:

```text
127.0.0.1; whoami
```

The shell executes `whoami`, revealing the OS username, even if `ping` is missing.

**Fix:** Validate the host, pass arguments as a list with `shell=False`, and run as a non-root user:

```python
import ipaddress

host = str(ipaddress.ip_address(target_host))
result = subprocess.run(
    ["ping", "-c", "2", host],
    shell=False, capture_output=True, text=True, timeout=10,
)
```

## 3. Missing Authentication and Authorization

**Location:** Account, transfer, and diagnostics routes.
Anyone who can reach the service can view accounts, submit transfers, and run diagnostics. Sender account ownership is not checked.

**Example:** Submit a transfer without logging in (PowerShell):

```powershell
curl.exe -X POST http://localhost:5000/transfer -d "sender_account=ACC-10001&recipient_name=DemoVendor&recipient_iban=DE1234567890&amount=1&currency=USD"
```

**Fix:** Require login, restrict diagnostics by role, and verify sender account ownership.

## 4. Insufficient Transfer Validation

**Location:** `/transfer`.
Account existence, account status, available funds, and supported currency are not checked. The `amount <= 0` check also accepts positive infinity and does not reject `NaN` before database execution.

**Examples:**

- `amount=999999999` for `ACC-10001` can produce a negative balance.
- `sender_account=DOES-NOT-EXIST` can record a transfer without debiting any account.
- `amount=inf` can store an infinite amount and corrupt the balance.

**Fix:** Validate all fields, require a finite positive monetary amount, and debit atomically only when a valid account has sufficient funds.

## 5. Missing CSRF Protection

**Location:** POST handlers for `/transfer` and `/system-diagnostics`.
Forms have no CSRF tokens or server-side origin checks. A third-party page can attempt to submit an unwanted request from a visitor's browser. Whether it reaches localhost depends on browser network restrictions; these routes currently do not require login either.

**Example:** A third-party page could contain this form:

```html
<form action="http://localhost:5000/transfer" method="POST">
  <input type="hidden" name="sender_account" value="ACC-10001">
  <input type="hidden" name="recipient_name" value="DemoVendor">
  <input type="hidden" name="recipient_iban" value="DE1234567890">
  <input type="hidden" name="amount" value="1">
  <input type="hidden" name="currency" value="USD">
  <button type="submit">Submit</button>
</form>
```

**Fix:** Require CSRF tokens for state-changing browser requests and validate their origin.

## 6. Internal Error Disclosure

**Location:** `/accounts`, `/api/accounts`, and `/transfer`.
Responses expose SQLite errors. `/api/accounts` also returns the SQL query in `executed_query`.

**Example:** Open this URL:

```text
http://localhost:5000/api/accounts?search=%27
```

The unmatched quote causes an SQL error, and the JSON response reveals query details.

**Fix:** Log detailed errors on the server and return generic messages to clients.

## 7. Public Fallback Secret Key

**Location:** `app.secret_key` in `app.py` and `SECRET_KEY` in `compose.yaml`.
Fixed fallback keys are publicly visible. If one is used, someone who knows it can forge Flask session cookies, including flash messages. This alone does not grant an administrator role: the application has no role-based authentication.

**Example:** Without a `SECRET_KEY` override, Compose uses:

```text
apexfin-local-demo-key
```

**Fix:** Supply a randomly generated secret through the environment and remove public fallback keys.

The in-memory demo database still executes SQL, so SQL injection and transfer logic vulnerabilities remain. Diagnostics commands execute in the OS regardless of database mode. Publishing the Compose port only on localhost limits exposure but does not fix these issues.
