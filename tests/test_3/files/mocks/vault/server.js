// Fake secrets vault for the SSRF lab. Requires the IMDS-issued token.

const http = require("http");

const SECRETS = {
  pipeline: "PIPELINE_SECRET=VN-d03nt3st-th1s-1s-f4k3-42",
  db: "DATABASE_MASTER_KEY=fake-master-key-0001",
};

http
  .createServer((req, res) => {
    const auth = req.headers.authorization || "";
    if (!auth.startsWith("Bearer IMDS-TOKEN-")) {
      res.writeHead(401).end("missing or wrong bearer token");
      return;
    }
    const name = (req.url.match(/\/secrets\/([a-z]+)/) || [])[1];
    if (name && SECRETS[name]) {
      res.writeHead(200, { "content-type": "text/plain" }).end(SECRETS[name]);
      return;
    }
    res.writeHead(404).end("unknown secret");
  })
  .listen(8091, () => console.log("mock vault on :8091"));