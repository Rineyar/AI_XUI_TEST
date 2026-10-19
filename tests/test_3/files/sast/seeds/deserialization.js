// SEED — do not run. Scanner bait: CWE-502, unsafe deserialization.
const express = require("express");
const { exec } = require("child_process");
const app = express();
app.use(express.text({ type: "application/x-www-form-urlencoded" }));

// node-serialize-style instant RCE pattern: untrusted payload to an
// deserializer that can instantiate functions
function loadState(payload) {
  const unserialize = require("node-serialize");
  return unserialize.unserialize(payload);
}

app.post("/session", (req, res) => {
  const state = loadState(req.body);
  res.json(state);
});

// bonus: user input into exec (command injection, CWE-78)
app.get("/ping", (req, res) => {
  exec("ping -c 1 " + req.query.host, { timeout: 3000 }, (err, out) => {
    res.type("text").send(out || String(err));
  });
});

app.listen(3000);