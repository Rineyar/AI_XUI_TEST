# InsecureShield Claims Portal

An Express application with static frontend files and an in-memory customer, policy, and claim store. Vulnerabilities are summarized in [test_4.md](test_4.md).

Credential-shaped provider values are synthetic demo data. Some exposed credentials are accepted by this application's own local authentication endpoints.

## Run

From the `test_4` directory:

```powershell
docker compose up --build -d
```

Open http://localhost:3000. Stop with `docker compose down`.
View logs with `docker compose logs -f web`.
No external database is required. Password changes and token revocations are
stored in memory and are lost when the application restarts.
Automatic browser opening is disabled inside the container.

## Run without Docker

Requires Node.js 18 or newer and npm.

```powershell
npm ci
npm start
```

Open http://localhost:3000. Stop with Ctrl+C. No external database is required.
The server automatically attempts to open the portal in a browser.

## Files

- `server.js`: HTTP API, authentication, and WebSocket server.
- `data/db.js`: in-memory records and initial data.
- `public/`: frontend code, styles, and exposed configuration files.
- `package.json`, `package-lock.json`: dependencies and startup command.
- `Dockerfile`, `docker-compose.yml`, `.dockerignore`: container setup.
- `test_4.md`: vulnerability descriptions.
- `LICENSE`: copyright and license notices.
