// Labs 15-16: path traversal and command injection (extras surface).
const express = require("express");
const fs = require("fs");
const path = require("path");
const { requireLogin } = require("../middleware/auth");
const router = express.Router();

const IMG_DIR = path.join(__dirname, "..", "public", "img");

// Lab 15: Path traversal — "product image" download endpoint.
// VULN (A01/CWE-22): the ?file= parameter is joined to the images directory,
// so ../ escapes the folder: /download?file=../../../../etc/passwd
// SAFE: path.basename() the input, or resolve + prefix-check, or serve by id.
router.get("/download", (req, res) => {
  const name = req.query.file || "";
  const full = path.join(IMG_DIR, name);
  fs.readFile(full, (err, data) => {
    if (err) return res.status(404).render("error", { message: "File not found", error: {} });
    res.type(path.extname(full) || "application/octet-stream").send(data);
  });
});


// Lab 16: Command injection — "network diagnostics" panel.
// VULN (A05/CWE-78): the host string is concatenated into a shell command.
//   POST /tools/ping  host=127.0.0.1; cat /etc/passwd
// SAFE: execFile with an argv array (no shell), or validate against a strict
// hostname regex (^[a-zA-Z0-9.-]+$) and a fixed argv.
const { exec } = require("child_process");

router.post("/tools/ping", requireLogin, (req, res) => {
  const host = req.body.host || "127.0.0.1";
  exec(`ping -c 2 "${host}"`, { timeout: 6000 }, (err, stdout) => {
    res.render("ping", { output: (stdout || String(err)).slice(0, 4000), host });
  });
});

router.get("/tools/ping", requireLogin, (req, res) => {
  res.render("ping", { output: null, host: "" });
});
module.exports = router;