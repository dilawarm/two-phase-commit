//var axios = require("axios");
const fetch = require("node-fetch");
const bodyParser = require('body-parser')
require('request').debug = true

var mysql = require("mysql");
const runsqlfile = require("./runsqlfile.js");

let config = require("./config")


var pool1 = mysql.createPool({
    connectionLimit: 1,
    host: config.host,
    user: config.user,
    password: config.password,
    database: "wallet_service",
    debug: false,
    multipleStatements: true
});

var pool2 = mysql.createPool({
    connectionLimit: 1,
    host: config.host,
    user: config.user,
    password: config.password,
    database: "order_service",
    debug: false,
    multipleStatements: true
});

beforeAll(done => {
    runsqlfile("data-dumps/wallet-dump.sql", pool1, () => {
        runsqlfile("data-dumps/order-dump.sql", pool2, done);
        console.log("put up testData");
    });
});

afterAll(()=>{
    pool1.end();
    pool2.end();
});


test("valid order", done=>{
    let orcRes = "";
    const data = {
        "account":1,
        "amount":100,
        "user_id":1,
        "items": [1,2,3,4,5]
    }
    fetch("http://localhost:3000/purchase", {
        method: "POST", 
        headers: {
            'Content-Type': 'application/json',
          },
        body: JSON.stringify(data),
      })
      .then(response => response.text())
      .then(data => {
          expect(data).toBe("success");
          done();
      })
      .catch((error) => {
        console.error('Error:', error);
      });
});


test("invalid user id", done=>{
    let orcRes = "";
    const data = {
        "account":69,
        "amount":100,
        "user_id":1,
        "items": [1,2,3,4,5]
    }
    fetch("http://localhost:3000/purchase", {
        method: "POST", 
        headers: {
            'Content-Type': 'application/json',
          },
        body: JSON.stringify(data),
      })
      .then(response => response.text())
      .then(data => {
          expect(data).toBe("User does not exist");
          done();
      })
      .catch((error) => {
        console.error('Error:', error);
      });
});

test("Price is greater than balance", done=>{
    let orcRes = "";
    const data = {
        "account":1,
        "amount":10000,
        "user_id":1,
        "items": [1,2,3,4,5]
    }
    fetch("http://localhost:3000/purchase", {
        method: "POST", 
        headers: {
            'Content-Type': 'application/json',
          },
        body: JSON.stringify(data),
      })
      .then(response => response.text())
      .then(data => {
          expect(data).toBe("Balance too low");
          done();
      })
      .catch((error) => {
        console.error('Error:', error);
      });
});

test("not enough in stock", done=>{
    let orcRes = "";
    const data = {
        "account":1,
        "amount":10,
        "user_id":1,
        "items": [1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1]
    }
    fetch("http://localhost:3000/purchase", {
        method: "POST", 
        headers: {
            'Content-Type': 'application/json',
          },
        body: JSON.stringify(data),
      })
      .then(response => response.text())
      .then(data => {
          expect(data).toBe("Not in stock");
          done();
      })
      .catch((error) => {
        console.error('Error:', error);
      });
});