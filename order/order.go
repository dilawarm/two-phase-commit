package main

import (
	"database/sql"
	"fmt"
	"io/ioutil"
	"net"
	"os"
	"strconv"
	"strings"

	_ "github.com/go-sql-driver/mysql"
)

const (
	CONN_HOST = "localhost"
	CONN_PORT = "3334"
	CONN_TYPE = "tcp"
)

type Order struct {
	User_id int `json:"user_id"`
	Amount  int `json:"amount"`
}

func errorHandler(err error) {
	if err != nil {
		panic(err.Error())
	}
}

func handlePrepare(conn net.Conn, password string) *sql.Tx {
	p := make([]byte, 2048)
	_, err := conn.Read(p)

	if err != nil {
		fmt.Println("Error reading: ", err.Error())
	}
	message := string(p[:2048])
	temp := strings.Split(message, " ")
	user_id, _ := strconv.Atoi(temp[0])
	amount, _ := strconv.Atoi(temp[1])

	if err != nil {
		fmt.Println("Error reading:", err.Error())
	}

	db, err := sql.Open("mysql", "haavasma:"+password+"@tcp(127.0.0.1:3306)/haavasma")
	errorHandler(err)
	defer db.Close()

	var order Order
	order.Amount = amount
	order.User_id = user_id

	tx, err := db.Begin()
	errorHandler(err)
	res, err := tx.Exec("INSERT INTO order (order_id, user_id, amount) VALUES (DEFAULT, ?, ?)", user_id, amount)
	fmt.Println(res.RowsAffected())
	errorHandler(err)

	if err != nil {
		tx.Rollback()
		panic(err.Error())
	}
	conn.Write([]byte("1"))
	conn.Close()
	return tx
}

func main() {
	data, err := ioutil.ReadFile("../.config")
	if err != nil {
		fmt.Println("File reading error", err)
		os.Exit(1)
	}
	fmt.Println("Contents of file:")
	fmt.Println(string(data))
	password := string(data)

	socket, err := net.Listen(CONN_TYPE, CONN_HOST+":"+CONN_PORT)
	if err != nil {
		fmt.Println("Error listening: ", err.Error())
		os.Exit(1)
	}
	defer socket.Close()
	fmt.Println("Listening on " + CONN_HOST + ":" + CONN_PORT)
	prepared := false
	var tx *sql.Tx
	for {
		conn, err := socket.Accept()
		if err != nil {
			fmt.Println("Error accepting: ", err.Error())
			os.Exit(1)
		}
		if !prepared {
			tx = handlePrepare(conn, password)
			prepared = true
		}
	}
}
