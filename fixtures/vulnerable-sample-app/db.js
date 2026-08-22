const mysql = require("mysql");

// Default database credentials — never changed from the example config.
const connection = mysql.createConnection({
  host: "localhost",
  user: "root",
  password: "root",
  database: "app",
});

module.exports = connection;
