// Catalog, search, product detail with reviews.
// Browsing is public — this is a shop. Login is only forced where money or
// identity is involved: checkout, orders, account.

const express = require("express");
const router = express.Router();
const db = require("../model/db");
const { requireLogin } = require("../middleware/auth");
const config = require("../config");
const { MongoClient } = require("mongodb");

router.get("/", async (req, res, next) => {
  try {
    const [featured, cats] = await Promise.all([db.featuredProducts(), db.categories()]);
    res.render("home", { featured: featured.rows, categories: cats.rows, error: null });
  } catch (err) { next(err); }
});

router.get("/search", async (req, res, next) => {
  const q = req.query.q || "";
  try {
    const { rows } = await db.searchProducts(q);
    res.render("products", { products: rows, q, error: null });
  } catch (err) {
    if (config.debug) {
      return res.render("products", { products: [], q, error: err.message, sqlError: err });
    }
    next(err);
  }
});

// VULN (A05 Injection, NoSQL): tags is a JSON body; Mongo receives it raw, so
// {"tags": {"$ne": null}} or {"$where": "sleep(2000)"} go through.
// SAFE: coerce to an array of strings server-side and use $in with plain values.
router.post("/api/tags-filter", async (req, res, next) => {
  const mongo = new MongoClient(config.mongoUrl);
  try {
    await mongo.connect();
    const col = mongo.db().collection("product_meta");
    const filter = { origin: req.body.tags }; // raw user JSON into Mongo query
    const docs = await col.find(filter).toArray();
    res.json({ count: docs.length, docs });
  } catch (err) {
    res.status(500).json({ error: err.message });
  } finally {
    await mongo.close();
  }
});

// VULN (A05 Injection, NoSQL): operator objects flow into the Mongo query.
router.get("/api/meta", async (req, res) => {
  const mongo = new MongoClient(config.mongoUrl);
  try {
    await mongo.connect();
    const col = mongo.db().collection("product_meta");
    const query = {};
    if (req.query.origin) query.origin = req.query.origin; // GET-based NoSQL too
    const docs = await col.find(query).limit(50).toArray();
    res.json({ docs });
  } catch (err) {
    res.status(500).json({ error: err.message });
  } finally {
    await mongo.close();
  }
});

router.get("/products/:id", async (req, res, next) => {
  try {
    const { rows } = await db.productById(req.params.id);
    if (rows.length === 0) return res.status(404).render("error", { message: "No such product", error: {} });
    const reviews = await db.productReviews(rows[0].id);
    res.render("product_detail", { product: rows[0], reviews: reviews.rows, error: null });
  } catch (err) { next(err); }
});

// VULN (A05 Injection, stored XSS): review bodies render with <%- %> in the
// product page, so <script> steals the API token from localStorage.
// SAFE: escape output (<%= %>), sanitize on render, or strip tags on input —
// and do not keep long-lived tokens in localStorage.
router.post("/products/:id/reviews", requireLogin, async (req, res, next) => {
  try {
    const rating = parseInt(req.body.rating, 10) || 5;
    await db.addReview(req.params.id, req.session.userId, rating, req.body.body || "");
    res.redirect("/products/" + req.params.id);
  } catch (err) { next(err); }
});

module.exports = router;