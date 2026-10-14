// SEED — do not run. Scanner bait: CWE-918, SSRF.
const express = require("express");
const app = express();
app.use(express.json());

async function proxyFetch(url) {
  return fetch(url, { signal: AbortSignal.timeout(3000) });
}

app.post("/preview", async (req, res) => {
  try {
    const r = await proxyFetch(req.body.url);
    const text = await r.text();
    res.type("text").send(text.slice(0, 2000));
  } catch (e) {
    res.status(502).send("fetch failed");
  }
});

app.listen(3000);