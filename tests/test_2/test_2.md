# Main Project Vulnerabilities

Examples target the local application at `http://localhost:8000`. The actual query parameter is `website`, although the README uses `url`.

## 1. Server-Side Request Forgery (SSRF)

**Location:** `api/app.py`, `load_url()`.
The application passes user-controlled URLs to `requests.get()` without restricting destinations. A caller can make the container contact internal services and return their responses. Redirects are followed automatically, allowing an external URL to redirect to an internal address.

**Example:** Request the application's own home page through the server:

```text
http://localhost:8000/load?website=http://127.0.0.1:8000/
```

The result is `Content loading service`. Here, `127.0.0.1` refers to the container, not the caller's computer.

**Fix:** Allow only approved destinations and HTTP/HTTPS schemes; validate resolved IP addresses and block loopback, private, and link-local destinations. Disable redirects or validate every redirect destination. Restrict outbound network access.

## 2. Untrusted HTML Served from the Application Origin

**Location:** `return response.text` in `load_url()`.
Flask returns the fetched content as HTML by default. A caller can supply a URL serving arbitrary HTML or JavaScript, which a visitor's browser then executes under the application's origin.

**Example:** A controlled page containing this harmless marker:

```html
<script>document.body.textContent = "Script executed";</script>
```

When that page is loaded through `/load?website=...`, its script executes on the local application's origin.

**Fix:** Return fetched text as `text/plain`, or use an isolated sandbox if rendering remote HTML is required.

## 3. Missing Access Controls and Resource Limits

**Location:** `/load`.
No login or rate limit is required. Responses are fully downloaded into memory without a size limit. The five-second timeout limits connection/read waiting, not the total download duration; a slow stream can occupy workers for much longer.

**Example:** Several concurrent requests to a slow endpoint can occupy all four Gunicorn workers. A URL returning a very large body can exhaust available memory.

**Fix:** Require authorization where appropriate, limit request rates and concurrent fetches, stream responses with a maximum size, and enforce an overall deadline.

## 4. Internal Error Disclosure

**Location:** `except Exception as e: return str(e)`.
Raw exception messages are returned to callers, revealing connection failures, destination details, and networking behavior. Errors also return HTTP 200 by default.

**Example:**

```text
http://localhost:8000/load?website=http://127.0.0.1:1/
```

If the port is closed, the response exposes the underlying connection error.

**Fix:** Return a generic error with an appropriate status code, such as 502, and keep diagnostic details in server logs.

## 5. Sensitive Data in Logs

**Location:** `log_request()` and URL logging in `load_url()`.
Complete query strings and request bodies are logged without redaction. URLs can contain access tokens or other sensitive values.

**Example:**

```text
/load?website=https://example.com/?token=DEMO_SECRET
```

The token appears in application logs even if the upstream request fails.

**Fix:** Redact sensitive parameters, avoid logging complete bodies, and restrict log access and retention.

## 6. Broad Network Exposure and Root Execution

**Location:** `docker-compose.yml` and `api/Dockerfile`.
The `8000:8000` mapping publishes the service on all host interfaces by default, subject to firewall rules. The Dockerfile defines no non-root user, so the application runs as root inside the container. This increases the potential impact of a separate compromise; it does not itself provide host root access.

**Example:** If the host firewall permits it, another machine can access `http://<host-ip>:8000/load` and use the same unrestricted fetch endpoint.

**Fix:** Bind to `127.0.0.1:8000:8000` for a local lab, run as a non-root user, and restrict container privileges and network access.
