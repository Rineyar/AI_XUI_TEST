# Main Project Vulnerabilities

| Vulnerability | Location | Impact |
|---|---|---|
| Public credential exposure | `public/js/`, `public/config.json`, `public/asset-manifest.json`, HTML and sitemap comments | Client-accessible files reveal configuration and credential-shaped values. The repository describes provider credentials as synthetic demo data. |
| Local privilege escalation through leaked application credentials | `server.js`: `/api/auth/generate-token` | Credentials shipped to browsers are accepted by the local API to issue admin or superadmin tokens. |
| Client-side encryption with an exposed key | `public/js/main.js` | The encrypted configuration and its decryption key are distributed together, allowing recovery of privileged application credentials. |
| Hardcoded JWT signing secret | `server.js` | Anyone who obtains the signing key can forge tokens accepted by the local server. The APIM subscription key is also publicly exposed. |
| User enumeration | `/api/login` | An incorrect password for an existing email reveals the user's name, numeric profile ID, and role. |
| IDOR / broken object authorization | `/api/profile?id=`, `/api/account/info`, `/api/claims/:policyId` | An authenticated caller can access another customer's profile, account information, or claims without ownership checks. |
| Password reset without ownership verification | `PUT /api/profile/password` with an `id` | An authenticated caller can change another customer's password without providing the current password. |
| Password material disclosure | `/api/profile/password?id=`, other profile/customer responses | The dedicated endpoint exposes hashes and salts. Other handlers exclude only `password`, leaving hash and salt fields in responses. |
| Weak password storage | `data/db.js` | Passwords are stored as plaintext; additional password hashes use fast SHA-256 instead of a dedicated password hashing algorithm. |
| Internal configuration disclosure | `/api/admin/internal-config` | A token with source `sp` exposes the database connection string, administrator password, and APIM key. Leaked client-side credentials can obtain such a token. |
| Unauthenticated WebSocket access | `/ws/notifications` | The server reads but does not verify the supplied token before sending notifications and an internal key. |
| Tokens in URLs and logs | Browser WebSocket code and `server.js` | The token is placed in the WebSocket URL, and its prefix is written to the server log. |

Customer, policy, and claim records are held in memory. Password changes and token revocations disappear when the server restarts. No external database is required.
