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

func errorHandler(err error) {
	if err != nil {
		panic(err.Error())
	}
}

func main() {
	// Disse verdiene skal servicen få fra Coordinator via TCP
	// TODO: Sette opp TCP server for å hente verdier fra Coordinator

	user_id := 1
	price := 50
	transaction_id := 69

	db, err := sql.Open("mysql", "dilawar:passord123@tcp(127.0.0.1:3306)/wallet_service")
	errorHandler(err)

	defer db.Close()

	results, err := db.Query("SELECT * FROM wallet WHERE user_id=?", user_id)
	errorHandler(err)

	var wallet Wallet
	for results.Next() {

		err = results.Scan(&wallet.User_id, &wallet.Balance)
		if err != nil {
			panic(err.Error())
		}
	}
	fmt.Println("Wallet :", wallet)

	tx, err := db.Begin()
	errorHandler(err)
	res, err := tx.Exec("UPDATE wallet SET balance=? WHERE user_id=?", wallet.Balance-price, user_id)
	errorHandler(err)

	if wallet.Balance-price >= 0 {
		fmt.Println("OK PREP", transaction_id) // skal sendes til Coordinator

		if err != nil {
			tx.Rollback()
			panic(err.Error())
		}

		rows, err := res.RowsAffected()
		errorHandler(err)

		fmt.Println("Rows :", rows)

		errorHandler(tx.Commit())

		fmt.Println("OK COMMIT", transaction_id) // skal sendes til Coordinator

	} else {
		tx.Rollback()
		panic(err.Error())
	}

	fmt.Println("Done.")
}
