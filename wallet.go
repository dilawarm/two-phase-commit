package main

import (
	"database/sql"
	"fmt"

	_ "github.com/go-sql-driver/mysql"
)

type Wallet struct {
	User_id int `json:"user_id"`
	Balance int `json:"balance"`
}

func main() {

	db, err := sql.Open("mysql", "dilawar:passord123@tcp(127.0.0.1:3306)/wallet_service")

	if err != nil {
		panic(err.Error())
	}

	defer db.Close()

	results, err := db.Query("SELECT * FROM wallet")
	if err != nil {
		panic(err.Error())
	}

	for results.Next() {
		var wallet Wallet

		err = results.Scan(&wallet.User_id, &wallet.Balance)
		if err != nil {
			panic(err.Error())
		}

		fmt.Println(wallet)
	}
}
