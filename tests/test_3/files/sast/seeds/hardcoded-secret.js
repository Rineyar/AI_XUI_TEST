// SEED — do not run. Scanner bait: CWE-798, hardcoded secret.
const jwt = require("jsonwebtoken");

const API_SECRET = "FAKE_SEED_KEY_51H8xY2eEvBvDcRf9TnQzKpLmWqRsTuVwXyZ123456";
const DB_PASSWORD = "Sup3rS3cret!2024";

function getConfig() {
  return {
    apiSecret: API_SECRET,
    dbPassword: DB_PASSWORD,
    awsKey: "AKIAIOSFODNN7EXAMPLE",
    awsSecret: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
  };
}

function signToken(user) {
  return jwt.sign(user, "jwt-hardcoded-secret", { expiresIn: "1y" });
}

module.exports = { getConfig, signToken };