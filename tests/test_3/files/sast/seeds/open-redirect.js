// SEED — do not run. Scanner bait: CWE-601, open redirect.
const express = require("express");
const app = express();

app.get("/login", (req, res) => {
  const next = req.query.returnurl || "/";
  res.redirect(next); // absolute URLs redirect off-site
});

app.get("/out", (req, res) => {
  // same flaw, 302 via Location header
  res.location(req.query.to).sendStatus(302);
});

app.listen(3000);