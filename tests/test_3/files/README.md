# NodeBazaar

A storefront with intentionally vulnerable application code. Vulnerability descriptions are in [test_3.md](test_3.md).

## Run

```powershell
docker compose up --build -d
```

Open http://localhost:8888. Stop with `docker compose down`.
View logs with `docker compose logs -f web`.

Compose starts PostgreSQL, MongoDB, the application, and the internal metadata/vault services.
The application waits for both databases to become ready, then `db/init.js`
creates the PostgreSQL tables, primary/foreign keys and indexes from `db/init.sql`,
inserts initial accounts/products/reviews/orders, and populates MongoDB's
`product_meta` collection. Existing populated databases are not reseeded.

Database files are stored in the `pgdata` and `mongodata` volumes and survive
container recreation. The database and internal service ports are not published.

Optional: copy `.env.example` to `.env` to override the host port, PostgreSQL
password, and session/JWT keys. Defaults use demo credentials to preserve the
documented vulnerabilities. PostgreSQL initializes its password only when the
data volume is first created; changing `.env` does not update an existing database
password. If a custom PostgreSQL password contains URL-special characters,
the password in `DATABASE_URL` must be URL-encoded.

## Demo data

| Username | Password | Role |
|---|---|---|
| alice | alice123 | customer |
| bob | bob123 | customer |
| admin | admin123 | admin |

Payments are simulated. Use the test card `4242424242424242` and CVV `123`.

## Files

- `routes/`, `model/`, `middleware/`, `views/`, `public/`: application code and assets.
- `db/`: schema and initial data.
- `mocks/`: internal services reached by SSRF handlers.
- `sast/seeds/`, `sast/ground-truth.yaml`: standalone vulnerable snippets and their descriptions.
- `azure-pipelines.yml`: vulnerable CI configuration.
- Docker files, package manifests, and configuration: application startup and dependencies.
- `LICENSE`: copyright and license notices.

Use locally with simulated data.
