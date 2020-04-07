package main

import (
	"database/sql"
	"fmt"
	"io/ioutil"
	"net"
	"os"
	"strconv"
	"strings"

	"../micro"
	_ "github.com/go-sql-driver/mysql"
)

const CONN_PORT = "3334"

type Order struct {
	User_id int `json:"user_id"`
	Amount  int `json:"amount"`
}

func handlePrepare(conn net.Conn, password string) micro.Prep {
	p := make([]byte, 2048)
	_, err := conn.Read(p)

	if err != nil {
		fmt.Println("Error reading:", err.Error())
		conn.Write([]byte(strconv.Itoa(3))) // 3 = Error reading
		conn.Close()
		return micro.Prep{3, nil, 0}
	}

	message := string(p[:2048])
	fmt.Println(message)
	temp := strings.Split(message, " ")
	user_id, _ := strconv.Atoi(temp[0])
	amount, _ := strconv.Atoi(temp[1])

	micro.PreparedList.Mux.Lock()
	for _, n := range micro.PreparedList.List {
		if user_id == n {
			micro.PreparedList.Mux.Unlock()
			return micro.Prep{11, nil, 0}
		}
	}
	micro.PreparedList.List = append(micro.PreparedList.List, user_id)

	micro.PreparedList.Mux.Unlock()

	db, err := sql.Open("mysql", "haavasma:"+password+"@tcp(127.0.0.1:3306)/haavasma")
	if err != nil {
		return micro.Prep{4, nil, user_id}
	}
	defer db.Close()

	tx, err := db.Begin()
	if err != nil { //7 = could not start transaction
		return micro.Prep{7, tx, user_id}
	}

	res, err := tx.Exec("INSERT INTO order (order_id, user_id, amount) VALUES (DEFAULT, ?, ?)", user_id, amount)
	fmt.Println(res.RowsAffected())

	if err != nil {
		tx.Rollback() // 8 = Could not lock row
		return micro.Prep{8, tx, user_id}
	}
	return micro.Prep{1, tx, user_id}
}

func main() {
	micro.PreparedList = micro.List{List: []int{}}
	data, err := ioutil.ReadFile("../.config")
	if err != nil {
		fmt.Println("File reading error", err)
		os.Exit(1)
	}
	password := string(data)

	socket, err := net.Listen(micro.CONN_TYPE, micro.CONN_HOST+":"+CONN_PORT)
	if err != nil {
		fmt.Println("Error listening: ", err.Error())
		os.Exit(1)
	}
	defer socket.Close()
	fmt.Println("Listening on " + micro.CONN_HOST + ":" + CONN_PORT)

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
	micro.HandleCommit(conn, tx, user_id)
	conn.Close()
}
