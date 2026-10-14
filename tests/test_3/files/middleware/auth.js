// Auth middleware: session-based for the web shop, JWT for /api/v1.

const config = require("../config");

// Web session guard. Session shape: { logged, userId, username, role }.
function requireLogin(req, res, next) {
  if (!req.session.logged) {
    return res.redirect("/login?returnurl=" + encodeURIComponent(req.originalUrl));
  }
  next();
}

function requireAdmin(req, res, next) {
  if (!req.session.logged || req.session.role !== "admin") {
    return res.status(403).render("error", { message: "Admin area", error: {} });
  }
  next();
}

// API guard. VULN (A07): the HMAC secret is public ("jwt-super-secret" in
// config.js / docker-compose), so anyone can mint admin tokens, and the token
// lives for a year. SAFE: pin algorithms: ['HS256'], strong secret from env,
// short expiry.
function requireApiToken(req, res, next) {
  const jwt = require("jsonwebtoken");
  const header = req.headers.authorization || "";
  const token = header.startsWith("Bearer ") ? header.slice(7) : req.query.token;
  if (!token) return res.status(401).json({ error: "missing token" });
  try {
    req.apiUser = jwt.verify(token, config.jwtSecret);
    next();
  } catch (e) {
    return res.status(401).json({ error: "invalid token: " + e.message });
  }
}

module.exports = { requireLogin, requireAdmin, requireApiToken };