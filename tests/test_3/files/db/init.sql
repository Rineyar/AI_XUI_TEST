-- NodeBazaar schema. Created on boot by db/init.js; seed data in db/seed-data.js.

CREATE TABLE IF NOT EXISTS users (
  id          SERIAL PRIMARY KEY,
  username    VARCHAR(50)  NOT NULL UNIQUE,
  password    VARCHAR(100) NOT NULL,
  email       VARCHAR(200) NOT NULL,
  role        VARCHAR(20)  NOT NULL DEFAULT 'customer',
  api_token   VARCHAR(300),
  balance     INTEGER      NOT NULL DEFAULT 5000,
  created_at  TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS products (
  id          SERIAL PRIMARY KEY,
  name        VARCHAR(120) NOT NULL,
  description TEXT         NOT NULL,
  long_description TEXT    NOT NULL DEFAULT '',
  price_cents INTEGER      NOT NULL,
  image       VARCHAR(200) NOT NULL,
  stock       INTEGER      NOT NULL DEFAULT 25,
  category    VARCHAR(40)  NOT NULL DEFAULT 'classic',
  featured    BOOLEAN      NOT NULL DEFAULT false,
  tags        TEXT[]       NOT NULL DEFAULT '{}'
);

CREATE TABLE IF NOT EXISTS orders (
  id           SERIAL PRIMARY KEY,
  user_id      INTEGER     REFERENCES users(id),
  product_id   INTEGER     NOT NULL REFERENCES products(id),
  product_name VARCHAR(120) NOT NULL,
  quantity     INTEGER     NOT NULL DEFAULT 1,
  amount_cents INTEGER     NOT NULL,
  address      TEXT        NOT NULL,
  card_number  VARCHAR(30) NOT NULL,
  cvv          VARCHAR(5)  NOT NULL,
  status       VARCHAR(20) NOT NULL DEFAULT 'paid',
  created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS reviews (
  id         SERIAL PRIMARY KEY,
  product_id INTEGER NOT NULL REFERENCES products(id),
  user_id    INTEGER NOT NULL REFERENCES users(id),
  rating     INTEGER NOT NULL DEFAULT 5,
  body       TEXT    NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS payment_attempts (
  id         SERIAL PRIMARY KEY,
  user_id    INTEGER,
  order_id   INTEGER,
  card_number VARCHAR(30) NOT NULL,
  cvv        VARCHAR(5)  NOT NULL,
  outcome    VARCHAR(20) NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_orders_user   ON orders(user_id);

CREATE TABLE IF NOT EXISTS contact_messages (
  id         SERIAL PRIMARY KEY,
  email      VARCHAR(200) NOT NULL,
  message    TEXT         NOT NULL,
  created_at TIMESTAMPTZ  NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_reviews_prod  ON reviews(product_id);
