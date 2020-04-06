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

	db, err := sql.Open("mysql", "haavasma:"+password+"@tcp(127.0.0.1:3306)/haavasma")
	if err != nil {
		conn.Write([]byte(strconv.Itoa(4))) // 4 = Error connecting to database
		return Prep{4, nil}
	}
	defer db.Close()

	tx, err := db.Begin()
	if err != nil {
		conn.Write([]byte(strconv.Itoa(7))) //7 = could not start transaction
		return Prep{7, tx}
	}

	res, err := tx.Exec("INSERT INTO order (order_id, user_id, amount) VALUES (DEFAULT, ?, ?)", user_id, amount)
	fmt.Println(res.RowsAffected())

	if err != nil {
		tx.Rollback()
		conn.Write([]byte(strconv.Itoa(8))) // 8 = Could not lock row
		return Prep{8, tx}
	}
	conn.Write([]byte("1")) // 1 = OK PREP
	return Prep{1, tx}
}

func handleCommit(conn net.Conn, tx *sql.Tx) {
	buf := make([]byte, 2048)

	_, err := conn.Read(buf)

	if err != nil {
		fmt.Println("Error reading:", err.Error())
	}
	message := string(buf[:2048])
	temp := strings.Split(message, " ")
	id, _ := strconv.Atoi(temp[0])
	if id == 1 {
		err = tx.Commit()
		if err != nil {
			conn.Write([]byte(strconv.Itoa(10))) // Could not COMMIT
		}
		conn.Write([]byte(strconv.Itoa(2))) // 2 = OK COMMIT
	} else if tx != nil {
		tx.Rollback()
	} else {
		fmt.Println("do nothing, transaction never started")
	}
}

func main() {

	data, err := ioutil.ReadFile("../.config")
	if err != nil {
		fmt.Println("File reading error", err)
		os.Exit(1)
	}
	password := string(data)

	socket, err := net.Listen(CONN_TYPE, CONN_HOST+":"+CONN_PORT)
	if err != nil {
		fmt.Println("Error listening: ", err.Error())
		os.Exit(1)
	}
	defer socket.Close()
	fmt.Println("Listening on " + CONN_HOST + ":" + CONN_PORT)

	for {
		conn, err := socket.Accept()
		if err != nil {
			fmt.Println("Error accepting: ", err.Error())
			os.Exit(1)
		}
		go prepareAndCommit(conn, password)
	}
}

func prepareAndCommit(conn net.Conn, password string) {
	prep := handlePrepare(conn, password) // skriver her til Coordinator
	tx := prep.Tx
	handleCommit(conn, tx)
	conn.Close()
}
