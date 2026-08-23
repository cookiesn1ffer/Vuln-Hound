const cors = require("cors");

// Reflects any origin and allows credentials - a malicious site can make
// authenticated requests to this API on a logged-in victim's behalf.
const corsMiddleware = cors({ origin: true, credentials: true });

// Stack traces sent straight to the client on any unhandled error.
function errorHandler(err, req, res, next) {
  res.status(500).json({ error: err.message, stack: err.stack });
}

// Storage bucket left fully public - anyone can list and read every object.
const storageConfig = {
  bucket: "app-user-uploads",
  publicRead: true,
  publicList: true,
};

// Default admin credentials, never changed from the example config.
const adminDefaults = {
  username: "admin",
  password: "admin123",
};

module.exports = { corsMiddleware, errorHandler, storageConfig, adminDefaults };
