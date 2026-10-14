// Account, preferences (mass assignment + prototype pollution), API token.

const express = require("express");
const router = express.Router();
const db = require("../model/db");
const { requireLogin } = require("../middleware/auth");
const { merge } = require("../model/merge");
const jwt = require("jsonwebtoken");
const config = require("../config");

router.get("/account", requireLogin, async (req, res, next) => {
  try {
    const { rows } = await db.userById(req.session.userId);
    const u = rows[0];
    res.render("account", {
      user: u,
      token: u.api_token || null,
      prefs: JSON.stringify(global.userPrefs[req.session.username] || {}, null, 2),
      error: null,
      saved: req.query.saved === "1",
    });
  } catch (err) { next(err); }
});

// VULN (A06 Insecure Design): every profile field is copied onto the row,
// including role and balance. Send role=admin in the form and become admin.
// SAFE: build the update from an explicit { username, email } allowlist.
router.post("/account", requireLogin, async (req, res, next) => {
  try {
    await db.updateProfile(req.session.userId, req.body);
    if (req.body.role) req.session.role = req.body.role;
    res.redirect("/account?saved=1");
  } catch (err) { next(err); }
});

global.userPrefs = global.userPrefs || {};

// VULN (A08 Software/Data Integrity Failures): preferences are applied with a
// hand-rolled deep merge, so JSON keys like __proto__ reach Object.prototype:
//   curl -X POST /preferences -d '{"__proto__":{"isAdmin":true}}'
// After that, every fresh object literal exposes isAdmin: true.
// SAFE: block __proto__/constructor/prototype keys before merging.
router.post("/preferences", requireLogin, (req, res) => {
  const prefs = global.userPrefs[req.session.username] || {};
  merge(prefs, req.body || {});
  global.userPrefs[req.session.username] = prefs;
  res.json({ ok: true, prefs });
});

// The API token the UI keeps in localStorage (XSS lab target).
// VULN (A07): token signed with a weak, public "secret" and no expiry.
// SAFE: short-lived tokens, strong secret from env, aud/iss checks.
router.post("/account/token", requireLogin, async (req, res, next) => {
  try {
    const { rows } = await db.userById(req.session.userId);
    const u = rows[0];
    const token = jwt.sign(
      { id: u.id, username: u.username, role: u.role },
      config.jwtSecret,
      { expiresIn: "365d" }
    );
    await db.setApiToken(u.id, token);
    res.redirect("/account");
  } catch (err) { next(err); }
});

module.exports = router;