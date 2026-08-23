const express = require("express");
const jwt = require("jsonwebtoken");
const multer = require("multer");
const db = require("./db");
const { corsMiddleware, errorHandler } = require("./config");

const app = express();
app.use(express.json());
app.use(corsMiddleware);

// Files are accepted with no type/size validation and stored directly under
// the web root, so an uploaded script would be served back as executable.
const upload = multer({ dest: "./public/uploads" });

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

// Mass assignment: the entire request body is written onto the user record,
// so a client can set fields like `role` or `isVerified` that were never
// meant to be user-controlled.
app.put("/api/profile", (req, res) => {
  db.query("UPDATE users SET ? WHERE id = " + req.body.id, req.body, (err) => {
    res.json({ ok: true });
  });
});

// Function-level authorization failure: checks that a token exists at all,
// but never checks that the caller is actually an admin.
app.post("/api/admin/delete-user", (req, res) => {
  jwt.verify(req.headers.authorization, "secret123", (err, decoded) => {
    if (err) return res.status(401).json({ error: "unauthorized" });
    db.query("DELETE FROM users WHERE id = " + req.body.userId, () => {
      res.json({ ok: true });
    });
  });
});

// Unvalidated file upload: no type/size/content check, saved into the
// public web root under the attacker-chosen original filename.
app.post("/api/upload", upload.single("file"), (req, res) => {
  res.json({ path: "/uploads/" + req.file.originalname });
});

app.use(errorHandler);

app.listen(3000);
