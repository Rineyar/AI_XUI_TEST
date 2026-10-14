# Simple SSRF Lab

A simple deliberately vulnerable API demonstrating Server-Side Request Forgery (SSRF). The application accepts a URL parameter and makes a server-side request to that URL, returning the response to the client.

Also accessible via [Docker Hub](https://hub.docker.com/r/insecureapps/simple-ssrf).

⚠️ **WARNING: This application is deliberately vulnerable. Do not deploy in environments that you care about.**

## Setup

```bash
# Build and run the container
docker compose up --build -d

# Test the SSRF vulnerability
curl "http://localhost:8000/load?url=http://www.google.com"
```

## How it Works

The application exposes a `/load` endpoint that accepts a `url` parameter. Any URL provided will be fetched server-side using Python's requests library, and the response will be returned to the client. This is a classic SSRF vulnerability that allows an attacker to:

- Make requests to internal services
- Bypass firewall restrictions
- Potentially access sensitive data

## Structure
```
.
├── docker-compose.yml
└── api
    ├── Dockerfile
    ├── app.py
    └── requirements.txt
```

## Using this in a separate environment

You can host this with docker compose without cloning the repository. Just create a `docker-compose.yml` file with the following content:

```yaml
services:
  api:
    image: insecureapps/simple-ssrf:latest
    ports:
      - "8000:8000"
    environment:
      - WORKER_COUNT=4
```


## References
- [OWASP SSRF Guide](https://owasp.org/www-community/attacks/Server_Side_Request_Forgery)
- [OWASP SSRF Preventioin Cheatsheet](https://cheatsheetseries.owasp.org/cheatsheets/Server_Side_Request_Forgery_Prevention_Cheat_Sheet.html)
