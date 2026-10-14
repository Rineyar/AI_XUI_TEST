# Main Project Vulnerabilities

Vulnerable implementation: `app.py`.

| Vulnerability | Location | Impact |
|---|---|---|
| SQL injection in authentication | `authenticate_user()` | Username and password are inserted into SQL through an f-string, allowing query manipulation and authentication bypass. |
| SQL injection in search | `search_users()` | The search parameter is inserted into a LIKE expression. Modified queries can expose other tables, including the seeded secrets table. |
| Missing authentication and role checks | `/dashboard`, `/admin`, `/search` | Routes are accessible without login. The admin panel exposes all users and their plaintext passwords without verifying an administrator role. |
| IDOR / missing object authorization | `/profile`, `get_user_by_id()` | Any requester can select a user ID and read another employee's personal and salary information. |
| Flask session forgery | `app.secret_key = "notsecret"` | The public signing key allows session cookies to be forged. The dashboard trusts the username and role stored in the cookie. |
| Sensitive data in client-side sessions | Successful login session fields | Email, salary, phone, and other profile data are stored in Flask's default signed cookie. Signing protects integrity, not confidentiality. |
| Plaintext passwords | `init_db()`, `get_all_users()`, `/admin` | Passwords are stored without hashing and are rendered in administrative views. |
| Credentials and SQL in logs | `log_attempt()`, SQL query printing | Submitted passwords and full authentication queries are written to a local file and standard output. |
| Username enumeration | `/login` | Different messages for an unknown username and an incorrect password reveal which accounts exist. |
| Database error disclosure | Login and search responses | Raw SQLite error messages are displayed to users. |
| Debug server exposure | `app.run(host="0.0.0.0", debug=True)` | The development server listens on all interfaces with debugging enabled. Unhandled errors can expose traceback details; debugger access depends on its protection and network accessibility. |
| Missing login rate limits | `/login` | Repeated authentication attempts are not throttled. The injection detector labels attempts but does not block them. |
| Missing CSRF protection | `/login`, `/logout` | Login has no CSRF token or origin check, and logout changes session state through GET. Login CSRF and unwanted logout are possible subject to browser cookie behavior. |

The application uses SQLite and depends on Flask; the database is initialized
only in the `__main__` block. Initialization deletes and recreates the seeded
users, secrets, and messages, so existing data is reset whenever that block runs.
The logging code creates its own `logs` directory when needed.

Hardened examples, packet capture tools, AI notebooks, and deployment instructions
are not included.
