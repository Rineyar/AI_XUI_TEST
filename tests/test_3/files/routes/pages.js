// Home, contact, and category browsing.

const express = require("express");
const router = express.Router();
const db = require("../model/db");

router.get("/home", async (req, res, next) => {
  try {
    const [featured, cats] = await Promise.all([db.featuredProducts(), db.categories()]);
    res.render("home", { featured: featured.rows, categories: cats.rows, error: null });
  } catch (err) { next(err); }
});

// VULN (A09): the contact message is logged raw — newlines forge log lines.
// VULN (A05, stored): the message renders unescaped on the contact page
// "recent messages" board (only in DEBUG mode, which compose ships with).
// SAFE: strip control chars before logging, escape anything rendered,
// never render internal tickets to end users.
router.post("/contact", async (req, res) => {
  const { email, message } = req.body || {};
  console.log(`[contact] from=${email} message=${message}`); // VULN: log injection
  let ticket = null;
  try {
    const { rows } = await db.pool.query(
      "INSERT INTO contact_messages (email, message) VALUES ($1, $2) RETURNING id",
      [String(email || "").slice(0, 200), String(message || "").slice(0, 4000)]
    );
    ticket = rows[0].id;
  } catch (err) { /* contact storage is best-effort in the lab */ }
  res.render("contact", { sent: true, ticket, error: null, recent: [] });
});

router.get("/contact", (req, res) => res.render("contact", { sent: false, ticket: null, error: null, recent: [] }));

// Category browsing: products filtered by category slug.
router.get("/categories/:slug", async (req, res, next) => {
  try {
    const { rows } = await db.productsByCategory(req.params.slug);
    res.render("products", { products: rows, q: null, error: null, heading: req.params.slug });
  } catch (err) { next(err); }
});

module.exports = router;
