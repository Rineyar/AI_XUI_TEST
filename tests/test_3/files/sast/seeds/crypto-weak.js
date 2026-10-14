// SEED — do not run. Scanner bait: CWE-327, weak crypto.
const crypto = require("crypto");

function hashPassword(password) {
  return crypto.createHash("md5").update(password).digest("hex");
}

function makeToken(userId) {
  // non-random token: Math.random is not cryptographically secure
  return Math.random().toString(36).slice(2) + userId;
}

function encryptCard(number) {
  const cipher = crypto.createCipheriv("des-ecb", Buffer.from("8bytekey"), null);
  return Buffer.concat([cipher.update(number, "utf8"), cipher.final()]).toString("hex");
}

module.exports = { hashPassword, makeToken, encryptCard };