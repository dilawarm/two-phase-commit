package main

import (
	"database/sql"
	"encoding/binary"
	"fmt"
	"io/ioutil"
	"net"
	"os"

	"../micro"

	_ "github.com/go-sql-driver/mysql"
)

const CONN_PORT = "3333"

type Wallet struct {
	User_id int `json:"user_id"`
	Balance int `json:"balance"`
}

func handlePrepare(conn net.Conn, password string) micro.Prep {
	buf := make([]byte, 16)

	_, err := conn.Read(buf)

	if err != nil {
		fmt.Println("Error reading:", err.Error())
		return micro.Prep{10, nil, 0}
	}

	data := binary.BigEndian.Uint32(buf[:4])
	user_id := int(data)
	data = binary.BigEndian.Uint32(buf[4:])
	price := int(data)
	//fmt.Printf(user_id, price)
	fmt.Println(user_id, price)
	/*message := string(buf[:2048])
	temp := strings.Split(message, " ")
	//user_id, _ := strconv.Atoi(temp[0])
	price, _ := strconv.Atoi(temp[1])*/

	fmt.Println(user_id, price)

	if err != nil {
		fmt.Println("Error reading:", err.Error())
		return micro.Prep{3, nil, user_id}
	}

	db, err := sql.Open("mysql", "dilawar:"+password+"@tcp(127.0.0.1:3306)/wallet_service")
	if err != nil {
		//conn.Write([]byte(strconv.Itoa(4))) // 4 = Error connecting to database
		return micro.Prep{4, nil, user_id}
	}

	defer db.Close()

	results, err := db.Query("SELECT * FROM wallet WHERE user_id=?", user_id)
	if err != nil {
		//conn.Write([]byte(strconv.Itoa(5))) // Query went wrong
		return micro.Prep{5, nil, user_id}
	}

	var wallet Wallet
	for results.Next() {

		err = results.Scan(&wallet.User_id, &wallet.Balance)
		if err != nil {
			//conn.Write([]byte(strconv.Itoa(6))) // 6 = Wrong format on wallet object
			return micro.Prep{6, nil, user_id}
		}
	}
	fmt.Println("Wallet :", wallet)
	if wallet.User_id == 0 { // No user
		return micro.Prep{12, nil, user_id}
	}

	tx, err := db.Begin()
	if err != nil {
		//conn.Write([]byte(strconv.Itoa(7))) // Could not start transaction
		return micro.Prep{7, tx, user_id}
	}

	res, err := tx.Exec("UPDATE wallet SET balance=? WHERE user_id=?", wallet.Balance-price, user_id)
	fmt.Println(res.RowsAffected())

	micro.PreparedList.Mux.Lock()
	for _, n := range micro.PreparedList.List {
		if user_id == n {
			fmt.Println("user_id already in list of prepared transactions")
			micro.PreparedList.Mux.Unlock()
			return micro.Prep{11, nil, user_id}
		}
	}
	micro.PreparedList.List = append(micro.PreparedList.List, user_id)
	micro.PreparedList.Mux.Unlock()

	if wallet.Balance-price >= 0 {
		if err != nil {
			tx.Rollback()
			//conn.Write([]byte(strconv.Itoa(8))) // 8 = Could not lock row
			return micro.Prep{8, tx, user_id}
		}
		return micro.Prep{1, tx, user_id}
	} else {
		tx.Rollback()
		return micro.Prep{9, tx, user_id} // 9 = Balance too low.
	}
}

func main() {
	micro.PreparedList = micro.List{List: []int{}}
	data, err := ioutil.ReadFile("../.config")
	if err != nil {
		fmt.Println("File reading error", err)
		os.Exit(1)
	}
	password := string(data)

	l, err := net.Listen(micro.CONN_TYPE, micro.CONN_HOST+":"+CONN_PORT)
	if err != nil {
		fmt.Println("Error listening:", err.Error())
		os.Exit(1)
	}

	defer l.Close()
	fmt.Println("Listening on " + micro.CONN_HOST + ":" + CONN_PORT)
	for {
		conn, err := l.Accept()

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
