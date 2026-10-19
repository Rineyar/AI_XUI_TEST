// SEED — do not run. Scanner bait: CWE-89, SQL injection.
const { Pool } = require("pg");
const pool = new Pool({ connectionString: "postgres://u:p@localhost/db" });

function queryUser(username) {
  return pool.query("SELECT * FROM users WHERE name = '" + username + "'");
}

function queryOrder(id) {
  return pool.query(`SELECT * FROM orders WHERE id = ${id}`);
}

module.exports = { queryUser, queryOrder };