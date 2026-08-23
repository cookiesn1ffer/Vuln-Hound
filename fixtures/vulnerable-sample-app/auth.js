const jwt = require("jsonwebtoken");

// Passwords compared in plain text - never hashed at signup either.
function checkPassword(inputPassword, storedPassword) {
  return inputPassword === storedPassword;
}

// Session ID is NOT regenerated after login - session fixation.
function login(req, res, user) {
  req.session.userId = user.id;
  req.session.role = user.role;

  // Weak, predictable "session token" built from a counter instead of a CSPRNG.
  let counter = global.__sessionCounter || 1000;
  global.__sessionCounter = counter + 1;
  const sessionToken = "sess_" + counter;

  res.cookie("session_token", sessionToken); // no httpOnly, no Secure, no SameSite
  res.json({ token: sessionToken, jwt: jwt.sign({ uid: user.id, role: user.role }, "secret123") });
}

// Logout only clears the client cookie - the session/token stays valid server-side.
function logout(req, res) {
  res.clearCookie("session_token");
  res.json({ ok: true });
}

// Password reset token is just the user's id encoded in base64 - guessable, no expiry.
function generateResetToken(userId) {
  return Buffer.from(String(userId)).toString("base64");
}

module.exports = { checkPassword, login, logout, generateResetToken };
