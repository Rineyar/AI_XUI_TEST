# Main Project Vulnerabilities

| Vulnerability | Location | Impact |
|---|---|---|
| SQL injection | `model/db.js`: login, product search, order sorting, profile field names | User-controlled values or identifiers alter SQL queries. |
| NoSQL injection | `routes/shop.js`: `/api/tags-filter`, `/api/meta` | MongoDB operator objects change the intended filter. |
| IDOR / broken object authorization | `routes/orders.js`, `routes/api/v1.js` | A logged-in user can access another user's orders by changing an ID or `user_id`. |
| Mass assignment | `routes/auth.js`, `routes/account.js`, `model/db.js` | Registration accepts `role`; profile updates accept sensitive fields such as role and balance. |
| Stored XSS | Reviews in `views/product_detail.ejs` | Review HTML is rendered without escaping. Scripts can read API tokens from localStorage. |
| SSRF | `routes/tools.js`: `/tools/fetch` | User-supplied URLs let the server access internal services, including the mock metadata service and vault. |
| Excessive access to internal secrets | `routes/tools.js`: `/tools/vault-check` | Any logged-in user can retrieve the mock vault response; an admin role is not required. |
| Checkout price tampering | `routes/orders.js`, `model/db.js` | The order amount comes from the browser instead of the product price stored in the database. |
| Missing CSRF protection | State-changing browser routes | No CSRF tokens or origin checks prevent unwanted requests; exploitability depends on browser cookie rules. |
| Weak authentication secrets | `config.js`, `middleware/auth.js` | The public JWT secret allows signed token forgery. JWTs and session cookies last approximately one year. |
| Plaintext sensitive data | `db/init.js`, `model/db.js`, `routes/auth.js` | Passwords, card numbers, and CVVs are stored without protective handling. |
| Secret disclosure and log injection | `routes/auth.js`, `routes/pages.js`, `server.js` | Passwords and URLs are logged; raw user-controlled text can forge log entries. |
| Verbose errors | Login and search handlers, `config.js` | Database errors are exposed to users; debug behavior is enabled by default. |
| Prototype pollution | `model/merge.js`, `/preferences` | Recursive merging accepts special keys such as `__proto__`, allowing shared object prototypes to be modified. |
| Regular expression denial of service | `routes/tools.js`: `/tools/regex` | A user-supplied backtracking expression can block the Node.js event loop. |
| Path traversal | `routes/extras.js`: `/download` | Relative path segments can escape `public/img` and read files accessible to the process. |
| OS command injection | `routes/extras.js`: `/tools/ping` | User-controlled host input reaches a shell command and can execute additional commands. |
| Open redirect | `routes/auth.js`: login `returnurl` | A successful login can redirect users to an external destination. |
| Unsafe CI configuration | `azure-pipelines.yml` | PR-controlled branch values enter shell scripts; deployment secrets are referenced during PR processing. Actual access depends on Azure project permissions. |

`sast/seeds/` contains separate vulnerable code snippets, including eval-based code injection, unsafe deserialization, and weak cryptography. They are not loaded by the storefront. Their individual descriptions are listed in `sast/ground-truth.yaml`.

The metadata and vault services contain simulated data. PostgreSQL and MongoDB are required to run the storefront; there is no database fallback.
