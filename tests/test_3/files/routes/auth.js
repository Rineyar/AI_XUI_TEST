// Login, logout, register. Session-based auth for the HTML shop.

const express = require("express");
const router = express.Router();
const db = require("../model/db");

router.get("/login", (req, res) => {
  res.render("login", {
    returnurl: req.query.returnurl || "",
    error: req.query.error || null,
  });
});

// VULN (A05): login goes straight into concatenated SQL. Try
//   username: ' OR '1'='1
// VULN (A09): the username is logged raw, so CRLF/carriage tricks forge log lines.
// VULN (A01): the login error message echoes the raw SQL error back to the user.
// SAFE: parameterized query, structured logging of a sanitized user id,
// generic "invalid credentials" message.
router.post("/login/auth", async (req, res) => {
  const user = req.body.username;
  const password = req.body.password;
  console.log(`[auth] login attempt user=${user} password=${password}`); // VULN (A09): password in logs
  try {
    const { rows } = await db.login(user, password);
    if (rows.length === 0) throw new Error("User not found");
    const u = rows[0];
    req.session.logged = true;
    req.session.userId = u.id;
    req.session.username = u.username;
    req.session.role = u.role;
    // VULN (A01/A10): redirect target taken from the request, unvalidated.
    // SAFE: only allow relative paths starting with '/' and not '//'.
    const target = req.body.returnurl && req.body.returnurl !== "" ? req.body.returnurl : "/";
    res.redirect(target);
  } catch (err) {
    // VULN (A02/A10): raw DB error (often the SQL text) flows into the redirect.
    res.redirect("/login?returnurl=" + encodeURIComponent(req.body.returnurl || "") +
      "&error=" + encodeURIComponent(err.message));
  }
});

router.get("/logout", (req, res) => {
  req.session.destroy(() => res.redirect("/login"));
});

// VULN (A06 Insecure Design): the register form accepts any field and the
// handler forwards the body to the model, so role=admin mass-assigns an admin.
// SAFE: pick { username, password, email } explicitly.
router.post("/register", async (req, res) => {
  try {
    const fields = req.body; // whole body trusted
    const { rows } = await db.pool.query(
      `INSERT INTO users (username, password, email, role) VALUES ($1, $2, $3, $4) RETURNING id`,
      [fields.username, fields.password, fields.email || "", fields.role || "customer"]
    );
    req.session.logged = true;
    req.session.userId = rows[0].id;
    req.session.username = fields.username;
    req.session.role = fields.role || "customer";
    res.redirect("/");
  } catch (err) {
    res.render("register", { error: err.message });
  }
});

router.get("/register", (req, res) => {
  res.render("register", { error: null });
});

module.exports = router;