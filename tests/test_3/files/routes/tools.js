// "Admin tools": URL fetcher (SSRF) and regex validator (ReDoS).

const express = require("express");
const router = express.Router();
const { requireLogin } = require("../middleware/auth");
const config = require("../config");

router.get("/tools", requireLogin, (req, res) => {
  res.render("tools", { result: null, regexResult: null, error: null });
});

// VULN (A01/A02 SSRF): the URL comes from the user and is fetched server-side
// with no allowlist. From the app container:
//   http://imds:8090/latest/meta-data/  -> cloud metadata
//   http://vault:8091/secrets/pipeline  -> "vault" secret, needs IMDS token
//   http://169.254.169.254/             -> real metadata if this were a VM
// SAFE: scheme+host allowlist (or egress proxy), block link-local ranges,
// resolve DNS after validation and pin the IP.
router.post("/tools/fetch", requireLogin, async (req, res, next) => {
  try {
    const response = await fetch(req.body.url, {
      headers: req.body.token ? { Authorization: "Bearer " + req.body.token } : {},
      signal: AbortSignal.timeout(5000),
    });
    const text = await response.text();
    res.render("tools", {
      result: { url: req.body.url, status: response.status, body: text.slice(0, 4000) },
      regexResult: null, error: null,
    });
  } catch (err) {
    res.render("tools", { result: null, regexResult: null, error: "fetch failed: " + err.message });
  }
});

// VULN (A05/ReDoS): catastrophic backtracking regex over user input; a single
// request pins the event loop:
//   pattern: ^(\d+)*$   input: 1111111111111111111111111111111X
// SAFE: bound the input length, use non-backtracking engines (RE2), or drop
// the nested quantifier.
router.post("/tools/regex", requireLogin, (req, res) => {
  const pattern = req.body.pattern || "";
  const input = (req.body.input || "").slice(0, 5000);
  try {
    const re = new RegExp(pattern);
    const match = re.test(input);
    res.render("tools", { result: null, regexResult: { pattern, input, match }, error: null });
  } catch (err) {
    res.render("tools", { result: null, regexResult: null, error: "bad regex: " + err.message });
  }
});

// Kept simple for the SSRF lab: this helper is what the "vault check" calls.
router.get("/tools/vault-check", requireLogin, async (req, res) => {
  try {
    // VULN: token requested over plain HTTP from a user-influenced host.
    const imds = await fetch(config.imdsUrl + "/latest/api/token", { method: "PUT" }).then(r => r.text());
    const secrets = await fetch(config.vaultUrl + "/secrets/pipeline", {
      headers: { Authorization: "Bearer " + imds },
    }).then(r => r.text());
    res.type("text").send(secrets);
  } catch (err) {
    res.status(502).send("vault check failed: " + err.message);
  }
});

module.exports = router;