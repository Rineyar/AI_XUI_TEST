// Creates the schema and seeds demo data on app boot.
// Idempotent: skips seeding when users already exist.

const fs = require("fs");
const path = require("path");
const { Pool } = require("pg");
const config = require("../config");
const { products, users } = require("./seed-data");

async function initDb() {
  const pool = new Pool({ connectionString: config.databaseUrl });
  await pool.query(fs.readFileSync(path.join(__dirname, "init.sql"), "utf8"));

  const { rows } = await pool.query("SELECT COUNT(*)::int AS n FROM users");
  if (rows[0].n === 0) {
    // Passwords stored in plain text on purpose: part of the A04/A07 story.
    for (const u of users) {
      await pool.query(
        "INSERT INTO users (username, password, email, role) VALUES ($1, $2, $3, $4)",
        [u.username, u.password, u.email, u.role]
      );
    }
    for (const p of products) {
      await pool.query(
        "INSERT INTO products (name, description, long_description, price_cents, image, stock, category, featured, tags) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        [p.name, p.description, p.long_description, p.price_cents, p.image, p.stock, p.category, p.featured, p.tags]
      );
    }
    // A couple of reviews so product pages look alive.
    await pool.query(
      "INSERT INTO reviews (product_id, user_id, rating, body) VALUES (1, 1, 5, 'Sounds like a haunted typewriter. Five stars.'), (2, 2, 4, 'The duck solved a race condition. Still floats weird.')"
    );
    await pool.query(
      "INSERT INTO orders (user_id, product_id, product_name, quantity, amount_cents, address, card_number, cvv, status) VALUES (2, 2, 'Rubber Duck Pro', 1, 999, '221B Baker Street', '4242424242424242', '123', 'paid')"
    );
  }

  // Mongo: product metadata collection used by the tag filter.
  const { MongoClient } = require("mongodb");
  const mongo = new MongoClient(config.mongoUrl);
  try {
    await mongo.connect();
    const col = mongo.db().collection("product_meta");
    const existing = await col.countDocuments();
    if (existing === 0) {
      await col.insertMany(
        products.map((p, i) => ({
          product_id: i + 1,
          origin: ["shenzhen", "barcelona", "berlin", "kyiv"][i % 4],
          restock_eta_days: (i % 5) + 1,
          warehouse: ["eu-1", "eu-2", "us-1"][i % 3],
        }))
      );
    }
  } finally {
    await mongo.close();
  }

  await pool.end();
}

module.exports = { initDb };

if (require.main === module) {
  initDb()
    .then(() => console.log("Database ready."))
    .catch((err) => {
      console.error(err);
      process.exit(1);
    });
}
