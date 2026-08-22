const express = require("express");
const jwt = require("jsonwebtoken");
const db = require("./db");

const app = express();
app.use(express.json());

// Hardcoded secret — should be an env var.
const stripe = require("stripe")("sk_live_51H3xampleSecretKeyDoNotUse00000000");

// SQL injection: raw string concatenation of user input into a query.
app.get("/api/search", (req, res) => {
  const q = req.query.q;
  db.query("SELECT * FROM users WHERE name = '" + q + "'", (err, rows) => {
    res.json(rows);
  });
});

// Reflected XSS: request input echoed straight into the HTML response.
app.get("/api/greet", (req, res) => {
  const name = req.query.name;
  res.send("<h1>Hello, " + name + "!</h1>");
});

// Login endpoint with no rate limiting or lockout of any kind.
app.post("/api/login", (req, res) => {
  const { username, password } = req.body;
  db.query(
    "SELECT * FROM users WHERE username = '" + username + "' AND password = '" + password + "'",
    (err, rows) => {
      if (rows.length > 0) {
        // Weak JWT verification: no algorithm restriction, and the secret is trivial.
        const token = jwt.sign({ user: username }, "secret123");
        res.json({ token });
      } else {
        res.status(401).json({ error: "invalid credentials" });
      }
    }
  );
});

// IDOR: order is fetched by client-supplied ID with no ownership check.
app.get("/api/orders/:id", (req, res) => {
  db.query("SELECT * FROM orders WHERE id = " + req.params.id, (err, rows) => {
    res.json(rows[0]);
  });
});

app.listen(3000);
