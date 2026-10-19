# PHP Vulnerability Examples for Static Analysis

A collection of deliberately vulnerable PHP code examples for use with static analysis tools (SATs), security training, and research.

A brief summary with examples and runtime limitations is available in [test_5.md](test_5.md).

Originally created as part of a Master's program project in IT-Security at the University of Applied Sciences Vienna (FH Technikum Wien).

## Vulnerabilities Covered

| File | Vulnerability |
|------|--------------|
| `src/sql-injection.php` | SQL Injection (PDO, SQLite) |
| `src/sql-injection_2.php` | SQL Injection (PDO, SQLite) |
| `src/cross-site-scripting-xss.php` | Cross-Site Scripting (XSS) |
| `src/basic-collection.php` | XSS, SQL Injection, Command Injection, Deprecated Functions |
| `src/path-traversal.php` | Path Traversal / Local File Inclusion |
| `src/csrf.php` | Cross-Site Request Forgery (CSRF) |
| `src/insecure-file-upload.php` | Insecure File Upload |
| `src/idor.php` | Insecure Direct Object Reference (IDOR) |
| `src/open-redirect.php` | Open Redirect |
| `src/sensitive-data-exposure.php` | Sensitive Data Exposure (Hardcoded Credentials) |
| `src/remote-file-inclusion.php` | Remote File Inclusion (RFI) |
| `src/insecure-deserialization.php` | Insecure Deserialization |
| `src/xxe.php` | XML External Entity (XXE) |
| `src/ldap-injection.php` | LDAP Injection |
| `src/security-misconfiguration.php` | Security Misconfiguration |
| `src/weak-cryptography.php` | Weak Cryptography |
| `src/session-fixation.php` | Session Fixation |
| `src/type-juggling.php` | Type Juggling — Loose Comparison |
| `src/type-juggling_2.php` | Type Juggling — Type Casting |

### Tool-specific Test Files

Files under `src/tool-examples/` contain test code tailored to specific SATs:

| File | Tool |
|------|------|
| `src/tool-examples/progpilot.php` | [Progpilot](https://github.com/designsecurity/progpilot) |
| `src/tool-examples/phpcs-security-audit.php` | [phpcs-security-audit](https://github.com/FloeDesignTechnologies/phpcs-security-audit) |
| `src/tool-examples/php-security-scanner.php` | [php-security-scanner](https://github.com/dustyfresh/PHP-vulnerability-audit-cheatsheet) |

## Usage

### Docker

From the `test_5` directory:

```powershell
docker compose up --build -d
```

Open an example directly, such as:

- http://localhost:8080/sql-injection.php?id=1
- http://localhost:8080/sql-injection_2.php?id=1
- http://localhost:8080/idor.php?invoice_id=1
- http://localhost:8080/insecure-file-upload.php

Stop with `docker compose down`. View logs with `docker compose logs -f web`.
The container uses PHP 8.3 and Apache, includes PDO SQLite, PDO MySQL, LDAP,
and DOM support, and copies the existing SQLite database to the location used
by the examples. Database and upload files persist in Docker volumes.

These are independent code examples, not a complete application. The MySQL
credential example requires a separate MySQL server; the LDAP example requires
a separate LDAP server. Neither is provided by this Compose configuration.
Legacy examples using `mysql_query()` or `split()` do not run on PHP 8.3.
Remote URL inclusion is disabled by PHP's default configuration, so the RFI
example requires a separate configuration change to reproduce remote inclusion.

### Static analysis

```bash
git clone https://github.com/rubennati/vulnerable-php-code-examples.git
```

Run any PHP SAT against the files in `src/`. The SQLite database in `database/` is used by the SQL injection examples.

## Related

Based on research comparing open-source PHP static analysis tools and their effectiveness when combined into a toolchain. The evaluated SATs include Progpilot, phpcs-security-audit, php-security-scanner, OWASP WAP, SonarPHP, RIPS, and others.

**Related projects (FH Technikum Wien, PRJ1/PRJ2):**
- [vulnerability-code-scanner](https://gitlab.com/FHTW-MCS-PRJ/vulnerability-code-scanner) — Laravel-based toolchain combining the three selected SATs
- [PHP-Vulnerability-test-suite](https://gitlab.com/FHTW-MCS-PRJ/PHP-Vulnerability-test-suite) — 42,212 PHP test cases (29,258 safe / 12,954 unsafe)

## Warning

This repository contains intentionally insecure code. Do not deploy or use in production environments.
