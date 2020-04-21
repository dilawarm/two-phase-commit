package main

import (
	"database/sql"
	"encoding/binary"
	"fmt"
	"io/ioutil"
	"net"
	"os"
	"strings"

	"../micro"

	_ "github.com/go-sql-driver/mysql"
)

const CONN_PORT = "3332"

var list micro.List
var host = ""

type Wallet struct {
	User_id int `json:"user_id"`
	Balance int `json:"balance"`
}

func handlePrepare(conn net.Conn, password string) micro.Prep {
	// Read user id and price
	buf := make([]byte, 4)
	_, err := conn.Read(buf)
	data := binary.BigEndian.Uint32(buf[:4])
	user_id := int(data)
	if err != nil {
		return micro.Prep{0, nil, 0} // Error reading data
	}
	_, err = conn.Read(buf)
	data = binary.BigEndian.Uint32(buf[:4])
	price := int(data)
	if err != nil {
		return micro.Prep{0, nil, 0} // Error reading data
	}
	// Check if the user is busy
	list.Mux.Lock()
	if list.List[user_id] {
		list.Mux.Unlock()
		return micro.Prep{3, nil, user_id}
	}
	list.List[user_id] = true
	list.Mux.Unlock()

	if err != nil {
		return micro.Prep{3, nil, user_id}
	}
	// Connect to db
	db, err := sql.Open("mysql", password+"/wallet_service")
	if err != nil {
		return micro.Prep{4, nil, user_id}
	}

	defer db.Close()

	// Get user balance
	results, err := db.Query("SELECT * FROM wallet WHERE user_id=?", user_id)
	if err != nil {
		return micro.Prep{9, nil, user_id}
	}

	var wallet Wallet
	for results.Next() {

		err = results.Scan(&wallet.User_id, &wallet.Balance)
		if err != nil {
			return micro.Prep{10, nil, user_id} //
		}
	}
	if wallet.User_id == 0 { // No user
		return micro.Prep{11, nil, user_id}
	}

	// Start transaction
	tx, err := db.Begin()
	if err != nil {
		return micro.Prep{5, tx, user_id}
	}

	// Update balance
	_, err = tx.Exec("UPDATE wallet SET balance=? WHERE user_id=?", wallet.Balance-price, user_id)

	if wallet.Balance-price >= 0 {
		if err != nil {
			tx.Rollback()
			return micro.Prep{6, tx, user_id}
		}
		return micro.Prep{1, tx, user_id}
	} else {
		tx.Rollback()
		return micro.Prep{12, tx, user_id} //  = Balance too low.
	}
}

func main() {
	// Hashmap of users to lock other threads out from modifying the same user
	list = micro.List{List: make(map[int]bool)}
	// Read database config
	data, err := ioutil.ReadFile("../.config")
	if err != nil {
		os.Exit(1)
	}
	password := strings.TrimSpace(string(data))
	data, err = ioutil.ReadFile("../addresses")
	if err != nil {
		os.Exit(1)
	}
	host = strings.Split(string(data), " ")[1]
	// Listen for incomming requests
	l, err := net.Listen(micro.CONN_TYPE, host+":"+CONN_PORT)
	if err != nil {
		os.Exit(1)
	}

	defer l.Close()
	fmt.Println("Wallet microservice listening on " + host + ":" + CONN_PORT)
	for {

		conn, err := l.Accept()
		if err != nil {
			os.Exit(1)
		}
		go prepareAndCommit(conn, password)
	}
}

func prepareAndCommit(conn net.Conn, password string) {
	// Attempts to prepare transaction
	prep := handlePrepare(conn, password)
	tx := prep.Tx
	user_id := prep.User_id
	b := make([]byte, 2)
	binary.LittleEndian.PutUint16(b, uint16(prep.Id))
	// Write status to coordinator
	conn.Write(b)
	// Read response from coordinator an handle it
	micro.HandleCommit(conn, tx, user_id, list, prep.Id)
}
