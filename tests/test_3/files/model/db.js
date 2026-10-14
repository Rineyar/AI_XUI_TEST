// Data access layer. Every query that touches user input lives here so the
// vulnerable spots are easy to find and to diff against the fix.

const { Pool } = require("pg");
const config = require("../config");

const pool = new Pool({ connectionString: config.databaseUrl });

// VULN (A05 Injection): username and password are concatenated straight into SQL.
// Classic login bypass: ' OR '1'='1
// SAFE: parameterized query — pool.query("SELECT * FROM users WHERE username=$1 AND password=$2", [u, p])
function login(username, password) {
  const q = "SELECT * FROM users WHERE username = '" + username + "' AND password = '" + password + "'";
  return pool.query(q);
}

function userById(id) {
  return pool.query("SELECT * FROM users WHERE id = $1", [id]);
}

// VULN (A05 Injection): search terms are interpolated into ILIKE patterns.
// SAFE: pool.query("SELECT * FROM products WHERE name ILIKE $1 OR description ILIKE $2", [`%${q}%`, `%${q}%`])
function searchProducts(q) {
  const sql =
    "SELECT * FROM products WHERE name ILIKE '%" + q + "%' OR description ILIKE '%" + q + "%'";
  return pool.query(sql);
}

function listProducts() {
  return pool.query("SELECT * FROM products ORDER BY id");
}

function productById(id) {
  return pool.query("SELECT * FROM products WHERE id = $1", [id]);
}

function productReviews(productId) {
  return pool.query(
    "SELECT r.id, r.rating, r.body, r.created_at, u.username FROM reviews r JOIN users u ON u.id = r.user_id WHERE r.product_id = $1 ORDER BY r.id DESC",
    [productId]
  );
}

function addReview(productId, userId, rating, body) {
  return pool.query(
    "INSERT INTO reviews (product_id, user_id, rating, body) VALUES ($1, $2, $3, $4) RETURNING id",
    [productId, userId, rating, body]
  );
}

// VULN (A06 Insecure Design): the whole checkout amount arrives from the client.
// Tamper the form and any product costs what you want.
// SAFE: re-read the price from products by id and multiply server-side.
function createOrder(userId, item) {
  return pool.query(
    "INSERT INTO orders (user_id, product_id, product_name, quantity, amount_cents, address, card_number, cvv) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id",
    [userId, item.product_id, item.product_name, item.quantity, item.amount_cents, item.address, item.card_number, item.cvv]
  );
}

function featuredProducts() {
  return pool.query("SELECT * FROM products WHERE featured ORDER BY id LIMIT 4");
}

function categories() {
  return pool.query(
    "SELECT category, COUNT(*)::int AS n FROM products GROUP BY category ORDER BY category"
  );
}

function productsByCategory(category) {
  return pool.query("SELECT * FROM products WHERE category = $1 ORDER BY id", [category]);
}

// VULN (A04 Cryptographic Failures): PAN and CVV kept in clear text.
// SAFE: store only last4 + a tokenized reference; let the PSP keep the PAN.
function logPaymentAttempt(userId, cardNumber, cvv, outcome) {
  return pool.query(
    "INSERT INTO payment_attempts (user_id, card_number, cvv, outcome) VALUES ($1, $2, $3, $4)",
    [userId, cardNumber, cvv, outcome]
  );
}

// VULN (A01 Broken Access Control): caller passes any id, ownership never checked.
// SAFE: pool.query("... WHERE id = $1 AND user_id = $2", [orderId, req.session.userId])
function orderById(orderId) {
  return pool.query(
    "SELECT o.*, u.username FROM orders o JOIN users u ON u.id = o.user_id WHERE o.id = $1",
    [orderId]
  );
}

function ordersForUser(userId) {
  return pool.query("SELECT * FROM orders WHERE user_id = $1 ORDER BY id DESC", [userId]);
}

// VULN (A05 Injection): sort column is interpolated, ORDER BY cannot be a bound
// parameter. Whitelist instead.
// SAFE: const cols = { price: 'amount_cents', date: 'created_at' }; use cols[req.query.sort]
function ordersForUserSorted(userId, sort) {
  return pool.query(`SELECT * FROM orders WHERE user_id = $1 ORDER BY ${sort} DESC`, [userId]);
}

// VULN (A06 Insecure Design): every profile field is copied onto the user row,
// including role and balance.
// SAFE: pick only username/email into the UPDATE, never role/balance/api_token.
function updateProfile(userId, fields) {
  const sets = [];
  const values = [userId];
  for (const [k, v] of Object.entries(fields)) {
    values.push(v);
    sets.push(`${k} = $${values.length}`);
  }
  return pool.query(`UPDATE users SET ${sets.join(", ")} WHERE id = $1`, values);
}

function setApiToken(userId, token) {
  return pool.query("UPDATE users SET api_token = $1 WHERE id = $2", [token, userId]);
}

module.exports = {
  pool,
  login,
  userById,
  searchProducts,
  listProducts,
  productById,
  productReviews,
  addReview,
  createOrder,
  logPaymentAttempt,
  orderById,
  ordersForUser,
  ordersForUserSorted,
  updateProfile,
  setApiToken,
  featuredProducts,
  categories,
  productsByCategory,
};