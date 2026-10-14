// SEED — do not run. Scanner bait: CWE-943, NoSQL operator injection.
const { MongoClient } = require("mongodb");

async function findUser(body) {
  const mongo = new MongoClient("mongodb://localhost:27017");
  await mongo.connect();
  // body comes from JSON request; operators ride along
  const doc = await mongo.db().collection("users").findOne(body);
  await mongo.close();
  return doc;
}

async function findProduct(queryParams) {
  const mongo = new MongoClient("mongodb://localhost:27017");
  await mongo.connect();
  const filter = { name: queryParams.name, price: queryParams.price };
  const docs = await mongo.db().collection("products").find(filter).toArray();
  await mongo.close();
  return docs;
}

module.exports = { findUser, findProduct };