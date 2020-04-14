// @flow

let express = require("express");
let mysql = require("mysql");
let bodyParser: function = require("body-parser");
let app = express();
let server = app.listen(8082, () => console.log("Listening on port 8082"));

app.use(bodyParser.json()); // for å tolke JSON i body

const WalletDao = require("./DAO/walletdao.js");
let config: {host: string, user: string, password: string, database: string} = require("./config")

app.use(function(req, res, next) {
  res.header("Access-Control-Allow-Origin", "http://35.223.240.171/"); // update to match the domain you will make the request from
  res.header("Access-Control-Allow-Headers", "Origin, X-Requested-With, Content-Type, Accept, x-access-token");
  res.header("Access-Control-Request-Headers", "x-access-token");
  res.setHeader("Access-Control-Allow-Methods", "PUT, POST, GET, OPTIONS, DELETE");
  next();
});

let pool = mysql.createPool({
	connectionLimit: 2,
	host: config.host,
	user: config.user,
	password: config.password,
	database: config.database,
	debug: false
});

let walletDao = new WalletDao(pool);

app.post("/purchase", (req, res) => {
	console.log("Fikk post request fra klient.");
	console.log(req.body);
})