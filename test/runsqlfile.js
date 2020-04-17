var mysql = require("mysql");
var fs = require("fs");
var path = require("path");


module.exports = function run(filename, pool, done) {
  console.log("runsqlfile: reading file " + filename);
  let sql = fs.readFileSync(path.join(__dirname, filename), "utf8");
  pool.getConnection((err, connection) => {
    if (err) {
      console.log("runsqlfile: error connecting");
      done();
    } else {
      console.log("runsqlfile: connected");
      connection.query(sql, (err, rows) => {
        connection.release();
        if (err) {
          console.log(err);
          done();
        } else {
          console.log("runsqlfile: run ok");
          done();
        }
      });
    }
  });
};