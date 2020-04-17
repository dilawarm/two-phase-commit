var axios = require("axios");

var mysql = require("mysql");
const runsqlfile = require("./runsqlfile.js");

var pool1 = mysql.createPool({
    connectionLimit: 1,
    host: "mysql-1",
    user: "root",
    password: "secret",
    database: "wallet_service",
    debug: false,
    multipleStatements: true
});

var pool2 = mysql.createPool({
    connectionLimit: 1,
    host: "mysql-2",
    user: "root",
    password: "secret",
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

test("legge inn ordre som funker", done=>{
    axios.post('localhost:3003/purchase', {
        "account": 1,
        "amount": 100,
        "amount_of_items": 5,
        "items": [1, 2, 3, 4, 5]
    }).then(response => {
        expect(response.data).toBe("message received")
    })
})
