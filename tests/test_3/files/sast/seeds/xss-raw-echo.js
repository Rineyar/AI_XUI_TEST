// SEED — do not run. Scanner bait: CWE-79, reflected XSS.
const express = require("express");
const app = express();

app.get("/greet", (req, res) => {
  const name = req.query.name || "guest";
  res.send("<h1>Hello " + name + "</h1>");
});

function renderPage(template, data) {
  // template placeholders replaced without escaping
  return template.replace(/\{\{(\w+)\}\}/g, (_, key) => String(data[key]));
}

app.get("/profile", (req, res) => {
  const html = renderPage("<p>Welcome, {{user}}!</p>", { user: req.query.user });
  res.type("html").send(html);
});

app.listen(3000);