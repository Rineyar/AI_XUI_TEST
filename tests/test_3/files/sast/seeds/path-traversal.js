// SEED — do not run. Scanner bait: CWE-22, path traversal.
const fs = require("fs");
const path = require("path");

function readFileFor(baseDir, userPath) {
  const full = path.join(baseDir, userPath); // ../ escapes baseDir
  return fs.readFileSync(full, "utf8");
}

function avatarHandler(req, res) {
  const name = req.query.file;
  res.type("image/png").send(fs.readFileSync("/srv/app/public/img/" + name));
}

module.exports = { readFileFor, avatarHandler };