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

type Prep struct {
	Id int
	Tx *sql.Tx
}

func errorHandler(err error) {
	if err != nil {
		panic(err.Error())
	}
}

func handlePrepare(conn net.Conn, password string) Prep {
	p := make([]byte, 2048)
	_, err := conn.Read(p)

	if err != nil {
		fmt.Println("Error reading:", err.Error())
		conn.Write([]byte(strconv.Itoa(3))) // 3 = Error reading
		conn.Close()
		return Prep{3, nil}
	}

	message := string(p[:2048])
	fmt.Println(message)
	temp := strings.Split(message, " ")
	user_id, _ := strconv.Atoi(temp[0])
	amount, _ := strconv.Atoi(temp[1])

	db, err := sql.Open("mysql", "haavasma:CaMGxsUt@tcp(127.0.0.1:3306)/haavasma")
	if err != nil {
		conn.Write([]byte(strconv.Itoa(4))) // 4 = Error connecting to database
		conn.Close()
		return Prep{4, nil}
	}
	defer db.Close()

	tx, err := db.Begin()
	if err != nil {
		conn.Write([]byte(strconv.Itoa(7))) //7 = could not start transaction
		conn.Close()
		fmt.Println(err.Error())
		errorHandler(err)
		return Prep{7, nil}
	}

	res, err := tx.Exec("INSERT INTO order (order_id, user_id, amount) VALUES (DEFAULT, ?, ?)", user_id, amount)
	fmt.Println(res.RowsAffected())

	if err != nil {
		tx.Rollback()
		conn.Write([]byte(strconv.Itoa(8))) // 8 = Could not lock row
		conn.Close()
		return Prep{8, nil}
	}
	conn.Write([]byte("1")) // 1 = OK PREP
	conn.Close()
	return Prep{1, tx}
}

func handleCommit(conn net.Conn, tx *sql.Tx) {
	p := make([]byte, 1024)

	_, err := conn.Read(p)

	if err != nil {
		conn.Write([]byte("10"))
	}

	err = tx.Commit()
	if err != nil {
		conn.Write([]byte(strconv.Itoa(10))) // Could not Commit
		conn.Close()
		return
	}

	conn.Write([]byte(strconv.Itoa(2))) // 2 = OK COMMIT
	conn.Close()
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
	var id int
	for {
		conn, err := socket.Accept()
		if err != nil {
			fmt.Println("Error accepting: ", err.Error())
			os.Exit(1)
		}
		if !prepared {
			prep := handlePrepare(conn, password)
			id = prep.Id
			tx = prep.Tx
			prepared = true
		} else if id == 1 && prepared {
			handleCommit(conn, tx)
			prepared = false
		}
	}
}
