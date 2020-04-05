package main

import (
	"database/sql"
	"fmt"
	"net"
	"os"
	"strconv"
	"strings"

	_ "github.com/go-sql-driver/mysql"
)

const (
	CONN_HOST = "localhost"
	CONN_PORT = "3333"
	CONN_TYPE = "tcp"
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

func handlePrepare(conn net.Conn) *sql.Tx {

	buf := make([]byte, 2048)

	_, err := conn.Read(buf)

	if err != nil {
		fmt.Println("Error reading:", err.Error())
	}

	message := string(buf)
	temp := strings.Split(message, " ") // temp[0] = transaction_id, temp[1] = user_id, temp[2] = price.
	fmt.Println(temp)
	transaction_id, _ := strconv.Atoi(temp[0])
	user_id, _ := strconv.Atoi(temp[1])
	price := 50

	fmt.Println(transaction_id, user_id, price)

	if err != nil {
		fmt.Println("Error reading:", err.Error())
	}

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
	fmt.Println(res.RowsAffected())
	errorHandler(err)

	if wallet.Balance-price >= 0 {
		if err != nil {
			tx.Rollback()
			panic(err.Error())
		}

		conn.Write([]byte("OK PREP " + strconv.Itoa(transaction_id)))
		conn.Close()
		return tx

	} else {
		tx.Rollback()
		panic(err.Error())
	}
}

func handleCommit(conn net.Conn, tx *sql.Tx) {

	buf := make([]byte, 1024)

	_, err := conn.Read(buf)

	if err != nil {
		fmt.Println("Error reading:", err.Error())
	}

	temp := strings.Split(string(buf), " ") // temp[0] = "Commit", temp[1] = transaction_id
	transaction_id, _ := strconv.Atoi(temp[1])

	errorHandler(tx.Commit())

	conn.Write([]byte("OK Commit " + strconv.Itoa(transaction_id)))
	conn.Close()
}

func main() {
	l, err := net.Listen(CONN_TYPE, CONN_HOST+":"+CONN_PORT)
	if err != nil {
		fmt.Println("Error listening:", err.Error())
		os.Exit(1)
	}

	defer l.Close()
	fmt.Println("Listening on " + CONN_HOST + ":" + CONN_PORT)
	prepared := false
	var tx *sql.Tx
	for {
		conn, err := l.Accept()
		if err != nil {
			fmt.Println("Error accepting: ", err.Error())
			os.Exit(1)
		}
		if !prepared {
			tx = handlePrepare(conn)
			prepared = true
		} else {
			go handleCommit(conn, tx)
			prepared = false
		}
	}
}
