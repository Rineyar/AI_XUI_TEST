// Fake cloud instance metadata service (IMDS) for the SSRF lab.
// Behaves like the big clouds: PUT for the token, plain paths for metadata.

const http = require("http");

const TOKEN = "IMDS-TOKEN-3f9a2c";
const metadata = {
  "compute/instance-id": "i-0nodebazaar123456",
  "compute/region": "eu-south-2",
  "iam/security-credentials/": JSON.stringify(
    { AccessKeyId: "AKIAIOSFODNN7EXAMPLE", SecretAccessKey: "wJalrXUtnFEMI/example", Token: TOKEN }
  ),
};

http
  .createServer((req, res) => {
    if ((req.method === "PUT" || req.method === "GET") && req.url === "/latest/api/token") {
      res.writeHead(200).end(TOKEN);
      return;
    }
    if (req.url.startsWith("/latest/meta-data/")) {
      const key = req.url.slice("/latest/meta-data/".length).replace(/\/$/, "");
      if (metadata[key] !== undefined) {
        res.writeHead(200, { "content-type": "application/json" }).end(metadata[key]);
        return;
      }
    }
    if (req.url === "/latest/meta-data/") {
      res.writeHead(200).end(Object.keys(metadata).join("\n"));
      return;
    }
    res.writeHead(404).end("not found");
  })
  .listen(8090, () => console.log("mock IMDS on :8090"));