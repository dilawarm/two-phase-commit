package main

import (
	"database/sql"
	"encoding/binary"
	"fmt"
	"io/ioutil"
	"net"
	"os"
	"strconv"

	"../micro"
	_ "github.com/go-sql-driver/mysql"
)

const CONN_PORT = "3334"

type Order struct {
	User_id int `json:"user_id"`
	Amount  int `json:"amount"`
}

func handlePrepare(conn net.Conn, password string) micro.Prep {
	p := make([]byte, 16)
	_, err := conn.Read(p)

	if err != nil {
		fmt.Println("Error reading:", err.Error())
		conn.Write([]byte(strconv.Itoa(3))) // 3 = Error reading
		conn.Close()
		return micro.Prep{3, nil, 0}
	}

	data := binary.BigEndian.Uint32(p[:4])
	user_id := int(data)

	data = binary.BigEndian.Uint32(p[4:])
	amount := int(data)

	fmt.Println(user_id, amount)

	db, err := sql.Open("mysql", "dilawarm:"+password+"@tcp(localhost:3306)/order_service")
	if err != nil {
		return micro.Prep{4, nil, user_id}
	}
	defer db.Close()

	tx, err := db.Begin()
	if err != nil { //7 = could not start transaction
		return micro.Prep{7, tx, user_id}
	}

	res, err := tx.Exec("INSERT INTO `order` (order_id, user_id, amount) VALUES (DEFAULT, ?, ?)", user_id, amount)
	if err != nil {
		tx.Rollback() // 8 = Could not lock row
		return micro.Prep{8, tx, user_id}
	}
	fmt.Println(res.RowsAffected())

	micro.PreparedList.Mux.Lock()
	for _, n := range micro.PreparedList.List {
		if user_id == n {
			micro.PreparedList.Mux.Unlock()
			return micro.Prep{11, nil, 0}
		}
	}
	micro.PreparedList.List = append(micro.PreparedList.List, user_id)

	micro.PreparedList.Mux.Unlock()

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
	b := make([]byte, 2)
	fmt.Println(prep.Id)
	binary.LittleEndian.PutUint16(b, uint16(prep.Id))
	conn.Write(b)
	micro.HandleCommit(conn, tx, user_id)
	conn.Close()
}
