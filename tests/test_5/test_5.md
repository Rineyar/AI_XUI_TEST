# Main Project Vulnerabilities

Examples use the local container at `http://localhost:8080`.

| Vulnerability | Location | Example and impact |
|---|---|---|
| SQL injection | `src/sql-injection.php`, `src/sql-injection_2.php` | `/sql-injection.php?id=1%20OR%201=1` returns all employee records because input is concatenated into SQL. |
| Reflected XSS | `src/cross-site-scripting-xss.php` | `/cross-site-scripting-xss.php?name=%3Cscript%3Ealert(1)%3C/script%3E` executes the supplied script because output is not escaped. |
| Path traversal | `src/path-traversal.php` | `/path-traversal.php?file=../html/sql-injection.php` reads source code outside the uploads directory. This example reads files; it does not execute them as PHP. |
| Missing object access checks | `src/idor.php` | Changing `/idor.php?invoice_id=1` to `invoice_id=2` selects another customer record without authentication or ownership checks. Despite the parameter name, the query reads customers, not invoices. |
| Insecure file upload | `src/insecure-file-upload.php` | Uploading a PHP file containing `<?php echo 'Upload marker'; ?>` can make it executable through `/uploads/<filename>.php` in the provided container. File type and content are not validated. |
| Open redirect | `src/open-redirect.php` | `/open-redirect.php?url=https://example.com` redirects to an arbitrary external destination. |
| Insecure deserialization | `src/insecure-deserialization.php` | The decoded value `O:8:"stdClass":0:{}` is accepted by `unserialize()`. Arbitrary object types are allowed; code execution would additionally require a usable gadget class, which is not demonstrated here. |
| XML external entities (XXE) | `src/xxe.php` | POSTing the XML below causes external entity expansion and can disclose a local file. |
| LDAP injection | `src/ldap-injection.php` | `username=*` produces `(uid=*)`, broadening the LDAP query. Reproduction requires a reachable LDAP server and appropriate bind/query settings. |
| Hardcoded credentials | `src/sensitive-data-exposure.php` | The source contains a database administrator password and an API-key-shaped value. Its MySQL connection requires a separate server. |
| Information disclosure | `src/security-misconfiguration.php` | Opening the page calls `phpinfo()`, exposing PHP configuration and environment details. Error display is also enabled. |
| Weak cryptography | `src/weak-cryptography.php` | Passwords are hashed using fast MD5/SHA-1, and a token uses `rand(0, 9999)`, providing at most 10,000 possible values. These values are computed but not used in an implemented login flow. |
| Session fixation and unverified identity | `src/session-fixation.php` | `?session_id=demo-session-123` sets a caller-chosen session ID, and POST `username=admin` stores that identity without checking credentials. No session ID regeneration occurs. |
| Loose comparison | `src/type-juggling.php` | `/type-juggling.php?token=0e123` prints `Access granted`: the numeric string compares equal to zero. PHP-version-dependent examples involving arbitrary non-numeric strings may behave differently. |
| Unsafe type coercion | `src/type-juggling_2.php` | `/type-juggling_2.php?user_id=1abc` becomes integer `1` and is accepted as a valid ID. No actual user lookup is performed. |
| Missing CSRF token | `src/csrf.php` | A POST with `email=demo@example.com` is accepted without a CSRF token. This snippet only echoes the value; it does not persist an email change or implement an authenticated account. |
| Remote file inclusion pattern | `src/remote-file-inclusion.php` | User-controlled `page` is passed to `include()`. Remote HTTP inclusion requires `allow_url_include=On`, which is disabled by default in the provided container. Local paths can still be supplied. |
| Command injection and obsolete APIs | `src/basic-collection.php` | The `cmd` cookie is appended to a shell command. However, the earlier `mysql_query()` call fails on PHP 8.3, so this command path is not reached without adapting the legacy code. `split()` is also unavailable. |

Example XML for `xxe.php` (send as the form field `xml`):

```xml
<!DOCTYPE root [<!ENTITY marker SYSTEM "file:///etc/hostname">]>
<root>&marker;</root>
```

Typical fixes include parameterized SQL, escaped HTML output, ownership checks,
allowlisted file paths and upload types, validated redirect destinations,
restricted deserialization, disabled external entity expansion, escaped LDAP
filters, protected secrets, dedicated password hashing, secure random values,
session regeneration, strict input validation, and CSRF protection.

`src/tool-examples/` contains additional scanner-oriented snippets with unsafe
SQL, output, include, and other patterns. Some are legacy examples rather than
working PHP 8.3 endpoints. The SQLite database is included; MySQL and LDAP
servers are not provided by Compose.
