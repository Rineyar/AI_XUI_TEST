// NodeBazaar entrypoint.
// VULN (A02): morgan's combined access log includes URLs with tokens and the
// register POST carries the password in the body (body logging off here, but
// the auth route prints it — see routes/auth.js).
// SAFE: log to stdout, redact query strings carrying tokens.

const express = require("express");
const session = require("express-session");
const morgan = require("morgan");
const path = require("path");
const crypto = require("crypto");

const config = require("./config");
const { initDb } = require("./db/init");

async function main() {
  await initDb();

  const app = express();
  app.set("views", path.join(__dirname, "views"));
  app.set("view engine", "ejs");

  // Layout support for EJS: render the view, then wrap it in the layout as
  // `body`. Keeps the stack dependency-free (ejs dropped app-level layouts).
  app.use((req, res, next) => {
    const renderFile = require("ejs").renderFile;
    res.render = function (view, options, callback) {
      const self = this;
      const opts = options || {};
      const done = callback || function (err, str) {
        if (err) return req.next(err);
        self.req.app.emit("render", str);
        self.type("html");
        self.send(str);
      };
      const viewsDir = self.app.get("views");
      renderFile(path.join(viewsDir, view + ".ejs"), { ...res.locals, ...opts },
        (err, body) => {
          if (err) return done(err);
          renderFile(path.join(viewsDir, "layout.ejs"), { ...res.locals, ...opts, body },
            done);
        });
    };
    next();
  });
  app.use(morgan("combined"));
  app.use(express.json());
  app.use(express.urlencoded({ extended: true }));
  app.use(require("cookie-parser")());
  app.use(express.static(path.join(__dirname, "public")));

  // VULN (A07): hardcoded secret, cookie valid for a year, no secure flag.
  // SAFE: secret from env, maxAge of a few hours, secure+httpOnly+sameSite.
  app.use(
    session({
      name: config.sessionName,
      secret: config.sessionSecret,
      resave: false,
      saveUninitialized: false,
      cookie: { secure: false, httpOnly: true, maxAge: config.cookieMaxAgeMs },
    })
  );

  // Expose session bits to every template.
  app.use((req, res, next) => {
    res.locals.logged = !!(req.session && req.session.logged);
    res.locals.username = (req.session && req.session.username) || "";
    res.locals.role = (req.session && req.session.role) || "";
    next();
  });

  // CSRF: not implemented anywhere on purpose (lab 06b). Every state-changing
  // route relies only on the session cookie.

  app.use(require("./routes/pages"));
  app.use(require("./routes/extras"));
  app.use(require("./routes/shop"));
  app.use(require("./routes/auth"));
  app.use(require("./routes/orders"));
  app.use(require("./routes/account"));
  app.use(require("./routes/tools"));
  app.use("/api/v1", require("./routes/api/v1"));

  // 404
  app.use((req, res) => res.status(404).render("error", { message: "Not found", error: {} }));

  // VULN (A02/A10): with debug on, the raw error object (stack + SQL text)
  // reaches the template. SAFE: log server-side with an id, render a generic page.
  app.use((err, req, res, _next) => {
    console.error("[error]", err.status || 500, err.message);
    res.status(err.status || 500).render("error", {
      message: err.message,
      error: config.debug ? err : {},
    });
  });

  app.listen(config.port, () =>
    console.log(`NodeBazaar running at http://127.0.0.1:${config.port}`)
  );
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
