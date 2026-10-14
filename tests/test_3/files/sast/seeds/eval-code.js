// SEED — do not run. Scanner bait: CWE-95, code injection.
const express = require("express");
const app = express();
app.use(express.json());

function computeExpression(expr) {
  return eval(expr); // user JSON reaches eval
}

app.post("/calc", (req, res) => {
  res.json({ result: computeExpression(req.body.expression) });
});

app.get("/greet/:name", (req, res) => {
  // less obvious: Function constructor is eval in disguise
  const fn = new Function("name", "return 'hi ' + " + req.params.name);
  res.send(fn("world"));
});

app.listen(3000);