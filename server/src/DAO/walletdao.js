// @flow

const Dao = require("./dao.js");

module.exports = class WalletDao extends Dao {
    getAll(callback: function) {
        super.query(
            "SELECT * FROM wallet",
            [],
            callback
        );
    }
}