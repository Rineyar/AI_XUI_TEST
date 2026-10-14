// Broken JSON API under /api/v1. JWT auth only — separate from the cookie
// session on purpose: XSS steals this token from localStorage (lab 05).

const express = require("express");
const router = express.Router();
const db = require("../../model/db");
const { requireApiToken } = require("../../middleware/auth");
const jwt = require("jsonwebtoken");
const config = require("../../config");

// VULN (A07): the login issues a token with no expiry bound server-side and a
// guessable HMAC secret. alg=none forged tokens also verify (see middleware).
// SAFE: strong secret, short expiry, algorithms: ['HS256'], aud/iss pinned.
router.post("/auth/token", (req, res) => {
  const { username, password } = req.body || {};
  db.pool
    .query("SELECT * FROM users WHERE username = $1 AND password = $2", [username, password])
    .then(({ rows }) => {
      if (rows.length === 0) return res.status(401).json({ error: "bad credentials" });
      const u = rows[0];
      const token = jwt.sign({ id: u.id, username: u.username, role: u.role }, config.jwtSecret, {
        expiresIn: "365d",
      });
      res.json({ token });
    })
    .catch(() => res.status(500).json({ error: "server error" }));
});

router.get("/me", requireApiToken, async (req, res, next) => {
  try {
    const { rows } = await db.userById(req.apiUser.id);
    if (rows.length === 0) return res.status(404).json({ error: "no user" });
    const { password, ...safe } = rows[0];
    res.json(safe);
  } catch (err) { next(err); }
});

router.get("/orders", requireApiToken, async (req, res, next) => {
  try {
    // VULN (A01 BOLA): ?user_id= reads anyone's orders with any valid token.
    // SAFE: filter by req.apiUser.id and re-check role server-side.
    const userId = req.query.user_id || req.apiUser.id;
    const { rows } = await db.ordersForUser(userId);
    res.json({ orders: rows });
  } catch (err) { next(err); }
});

router.get("/products", requireApiToken, async (req, res, next) => {
  try {
    const { rows } = await db.listProducts();
    res.json({ products: rows });
  } catch (err) { next(err); }
});

module.exports = router;