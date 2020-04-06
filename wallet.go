package main

import (
	"database/sql"
	"encoding/binary"
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

type Prep struct {
	Id int
	Tx *sql.Tx
}

func errorHandler(err error) {
	if err != nil {
		panic(err.Error())
	}
}

func handlePrepare(conn net.Conn) Prep {

	buf := make([]byte, 1024)

	_, err := conn.Read(buf)

	data := binary.BigEndian.Uint64(buf)
	fmt.Println(data)

	if err != nil {
		fmt.Println("Error reading:", err.Error())
	}

	message := string(buf[:1024])
	temp := strings.Split(message, " ")
	user_id, _ := strconv.Atoi(temp[0])
	price, _ := strconv.Atoi(temp[1])

	fmt.Println(user_id, price)

	if err != nil {
		fmt.Println("Error reading:", err.Error())
		conn.Write([]byte(strconv.Itoa(3))) // 3 = Error reading
		conn.Close()
		return Prep{3, nil}
	}

	db, err := sql.Open("mysql", "dilawar:passord123@tcp(127.0.0.1:3306)/wallet_service")
	if err != nil {
		conn.Write([]byte(strconv.Itoa(4))) // 4 = Error connecting to database
		conn.Close()
		return Prep{4, nil}
	}

	defer db.Close()

	results, err := db.Query("SELECT * FROM wallet WHERE user_id=?", user_id)
	if err != nil {
		conn.Write([]byte(strconv.Itoa(5))) // 5 = NO user with that user_id
		conn.Close()
		return Prep{5, nil}
	}

	var wallet Wallet
	for results.Next() {

		err = results.Scan(&wallet.User_id, &wallet.Balance)
		if err != nil {
			conn.Write([]byte(strconv.Itoa(6))) // 6 = Wrong format on wallet object
			conn.Close()
			return Prep{6, nil}
		}
	}
	fmt.Println("Wallet :", wallet)

	tx, err := db.Begin()
	if err != nil {
		conn.Write([]byte(strconv.Itoa(7))) // Could not start transaction
		conn.Close()
		return Prep{7, nil}
	}

	res, err := tx.Exec("UPDATE wallet SET balance=? WHERE user_id=?", wallet.Balance-price, user_id)
	fmt.Println(res.RowsAffected())

	if wallet.Balance-price >= 0 {
		if err != nil {
			tx.Rollback()
			conn.Write([]byte(strconv.Itoa(8))) // 8 = Could not lock row
			conn.Close()
			return Prep{8, nil}
		}

		conn.Write([]byte(strconv.Itoa(1))) // 1 = OK PREP
		conn.Close()
		return Prep{1, tx}

	} else {
		tx.Rollback()
		conn.Write([]byte(strconv.Itoa(9))) // 9 = balance < price
		conn.Close()
		return Prep{9, nil}
	}
}

func handleCommit(conn net.Conn, tx *sql.Tx) {

	buf := make([]byte, 1024)

	_, err := conn.Read(buf)

	if err != nil {
		fmt.Println("Error reading:", err.Error())
	}

	err = tx.Commit()
	if err != nil {
		conn.Write([]byte(strconv.Itoa(10))) // Could not COMMIT
		conn.Close()
	}

	conn.Write([]byte(strconv.Itoa(2))) // 2 = OK COMMIT
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
	var id int
	for {
		conn, err := l.Accept()
		if err != nil {
			fmt.Println("Error accepting: ", err.Error())
			os.Exit(1)
		}
		if !prepared {
			prep := handlePrepare(conn)
			id = prep.Id
			tx = prep.Tx
			prepared = true
		} else if id == 1 && prepared {
			handleCommit(conn, tx)
			prepared = false
		}
	}
}
