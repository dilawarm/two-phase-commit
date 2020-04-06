package main

import (
	"database/sql"
	"fmt"
	"io/ioutil"
	"net"
	"os"
	"strconv"
	"strings"
	"sync"

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
	Id      int
	Tx      *sql.Tx
	User_id int
}

type List struct {
	list []int
	mux  sync.Mutex
}

var preparedList List

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
		return Prep{3, nil, 0}
	}

	message := string(p[:2048])
	fmt.Println(message)
	temp := strings.Split(message, " ")
	user_id, _ := strconv.Atoi(temp[0])
	amount, _ := strconv.Atoi(temp[1])

	preparedList.mux.Lock()
	for _, n := range preparedList.list {
		if user_id == n {
			preparedList.mux.Unlock()
			return Prep{11, nil, 0}
		}
	}
	preparedList.list = append(preparedList.list, user_id)

	preparedList.mux.Unlock()

	db, err := sql.Open("mysql", "haavasma:"+password+"@tcp(127.0.0.1:3306)/haavasma")
	if err != nil {
		return Prep{4, nil, user_id}
	}
	defer db.Close()

	tx, err := db.Begin()
	if err != nil { //7 = could not start transaction
		return Prep{7, tx, user_id}
	}

	res, err := tx.Exec("INSERT INTO order (order_id, user_id, amount) VALUES (DEFAULT, ?, ?)", user_id, amount)
	fmt.Println(res.RowsAffected())

	if err != nil {
		tx.Rollback() // 8 = Could not lock row
		return Prep{8, tx, user_id}
	}
	return Prep{1, tx, user_id}
}

func handleCommit(conn net.Conn, tx *sql.Tx, user_id int) {
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
	preparedList.mux.Lock()
	for i := 0; i < len(preparedList.list); i++ {
		if user_id == preparedList.list[i] {
			preparedList.list[i] = preparedList.list[len(preparedList.list)-1] // Copy last element to index i.
			preparedList.list[len(preparedList.list)-1] = 0                    // Erase last element (write zero value).
			preparedList.list = preparedList.list[:len(preparedList.list)-1]
		}
	}
	preparedList.mux.Unlock()
}

func main() {
	preparedList = List{list: []int{}}
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
	user_id := prep.User_id
	conn.Write([]byte(strconv.Itoa(prep.Id)))
	handleCommit(conn, tx, user_id)
	conn.Close()
}
