//var axios = require("axios");
const fetch = require("node-fetch");
const bodyParser = require('body-parser')

var mysql = require("mysql");
const runsqlfile = require("./runsqlfile.js");

var pool1 = mysql.createPool({
    connectionLimit: 1,
    host: "localhost",
    user: "haavasma",
    password: "pwd2741",
    database: "wallet_service",
    debug: false,
    multipleStatements: true
});

var pool2 = mysql.createPool({
    connectionLimit: 1,
    host: "localhost",
    user: "haavasma",
    password: "pwd2741",
    database: "order_service",
    debug: false,
    multipleStatements: true
});

beforeAll(done => {
    /*runsqlfile("data-dumps/wallet-dump.sql", pool1, () => {
        runsqlfile("data-dumps/order-dump.sql", pool2, done);
        console.log("put up testData");
    });*/
});

afterAll(()=>{
    pool1.end();
    pool2.end();
});

test("legge inn ordre som funker", done=>{
    /*axios.post("http://localhost:3000/purchase", {
        "account":1,
        "amount":100,
        "user_id":1,
        "amount_of_items":5,
        "items": [1,2,3,4,5]
    }).then(response => {
        expect(response.data).toBe("message received")
    });*/
    let orcResponse = "";
    const data = {
        "account":1,
        "amount":100,
        "user_id":1,
        "amount_of_items":5,
        "items": [1,2,3,4,5]
    }
    fetch("http://localhost:3000/purchase", {
        method: "POST", 
        headers: {
            'Content-Type': 'application/json',
          },
        body: JSON.stringify(data),
      }).then((response) => /*response.json()*/ {
          console.log(response.text());
          expect(response.data).toBe("success");
          //done();
       }).catch((error) => {
        console.error('Error:', error);
      });
}, 10000);
