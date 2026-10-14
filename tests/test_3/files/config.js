// NodeBazaar configuration.
// Values can be configured through environment variables.
// VULN (A02): real deployments would never ship defaults like these.

const config = {
  port: process.env.PORT || 3000,
  // VULN (A07): session secret hardcoded in source control.
  // SAFE: read from env/secret manager and rotate: process.env.SESSION_SECRET
  sessionSecret: process.env.SESSION_SECRET || "nodebazaar-dev-secret-do-not-use-in-prod",
  sessionName: "nodebazaar.sid",
  // VULN (A07): session cookie lives ~1 year and works over plain HTTP.
  // SAFE: maxAge of a few hours, secure: true, httpOnly: true, sameSite: 'lax'
  cookieMaxAgeMs: 1000 * 60 * 60 * 24 * 365,
  // VULN (A07): anyone can mint tokens with this public "secret".
  // SAFE: strong random secret from env: process.env.JWT_SECRET
  jwtSecret: process.env.JWT_SECRET || "jwt-super-secret",
  databaseUrl: process.env.DATABASE_URL || "postgres://nodebazaar:nodebazaar@127.0.0.1:5432/nodebazaar",
  mongoUrl: process.env.MONGO_URL || "mongodb://127.0.0.1:27017/nodebazaar",
  // Internal services only reachable from the Docker network (SSRF lab).
  imdsUrl: process.env.IMDS_URL || "http://imds:8090",
  vaultUrl: process.env.VAULT_URL || "http://vault:8091",
  // VULN (A02/A10): stack traces and SQL errors rendered to the user.
  // SAFE: false in every real deployment, errors go to logs with a correlation id.
  debug: (process.env.DEBUG_ERRORS || "1") === "1",
};

module.exports = config;
