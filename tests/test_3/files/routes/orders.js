// Checkout and orders. Cookie session for the web, IDOR + price tampering labs.

const express = require("express");
const config = require("../config");
const db = require("../model/db");
const { requireLogin } = require("../middleware/auth");
const router = express.Router();

router.get("/checkout", (req, res) => {
  // Guests can buy; login is offered, not forced. Identity is only needed
  // for the account area — a shop that walls its catalog behind a login
  // loses customers.
  res.render("checkout", { error: null, result: null });
});

// VULN (A06 Insecure Design): amount_cents and product_name come from the form.
// Change amount_cents to 1 and a keyboard costs one cent.
// VULN (A01): no CSRF token — another site can POST a purchase on your session.
// VULN (A04): PAN and CVV are stored verbatim (see model/db.logPaymentAttempt).
// SAFE: derive amount from the product row server-side, add a CSRF token,
// tokenize the card via the payment provider.
router.post("/checkout", async (req, res, next) => {
  try {
    const b = req.body;
    if (!b.card_number || !b.cvv || !b.address) {
      return res.render("checkout", { error: "Missing card/address fields", result: null });
    }
    const amount = parseInt(b.amount_cents, 10);
    const item = {
      product_id: parseInt(b.product_id, 10),
      product_name: b.product_name,
      quantity: parseInt(b.quantity, 10) || 1,
      amount_cents: isNaN(amount) ? 0 : amount,
      address: b.address,
      card_number: b.card_number,
      cvv: b.cvv,
    };
    // Guests can buy: userId is null when no session. Identity only gates
    // the account area, not the purchase.
    const buyerId = req.session.logged ? req.session.userId : null;
    const { rows } = await db.createOrder(buyerId, item);
    await db.logPaymentAttempt(buyerId, b.card_number, b.cvv, "paid");
    res.render("checkout", { error: null, result: { orderId: rows[0].id, amount: item.amount_cents } });
  } catch (err) { next(err); }
});

router.get("/orders", requireLogin, async (req, res, next) => {
  try {
    // VULN (A05): sort is interpolated into ORDER BY (cannot be a bound param).
    // SAFE: whitelist: { date: 'created_at', price: 'amount_cents' }[sort]
    const sort = req.query.sort || "created_at";
    const { rows } = await db.ordersForUserSorted(req.session.userId, sort);
    res.render("orders", { orders: rows, sort });
  } catch (err) {
    if (config.debug && err) return next(err);
    try {
      const { rows } = await db.ordersForUser(req.session.userId);
      res.render("orders", { orders: rows, sort: "created_at" });
    } catch (e) { next(e); }
  }
});

// VULN (A01 BOLA/IDOR): any logged-in user can read any order id.
// Bob's order is /orders/1 — alice can open it.
// SAFE: add "AND user_id = $2" with the session user id.
router.get("/orders/:id", requireLogin, async (req, res, next) => {
  try {
    const { rows } = await db.orderById(req.params.id);
    res.render("order_detail", { order: rows[0] });
  } catch (err) { next(err); }
});

module.exports = router;
