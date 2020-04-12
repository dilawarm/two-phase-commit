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

var list micro.List

type Wallet struct {
	User_id int `json:"user_id"`
	Balance int `json:"balance"`
}

func handlePrepare(conn net.Conn, password string) micro.Prep {
	buf := make([]byte, 4)
	_, err := conn.Read(buf)
	data := binary.BigEndian.Uint32(buf[:4])
	user_id := int(data)
	if err != nil {
		fmt.Println("Error reading:", err.Error())
		return micro.Prep{0, nil, 0} // Error reading data
	}
	_, err = conn.Read(buf)
	data = binary.BigEndian.Uint32(buf[:4])
	price := int(data)
	if err != nil {
		fmt.Println("Error reading:", err.Error())
		return micro.Prep{0, nil, 0} // Error reading data
	}

	//fmt.Printf(user_id, price)
	fmt.Println(user_id, price)

	list.Mux.Lock()
	if list.List[user_id] {
		fmt.Println("user_id already in list of prepared transactions")
		list.Mux.Unlock()
		return micro.Prep{3, nil, user_id}
	}
	list.List[user_id] = true
	list.Mux.Unlock()
	/*message := string(buf[:2048])
	temp := strings.Split(message, " ")
	//user_id, _ := strconv.Atoi(temp[0])
	price, _ := strconv.Atoi(temp[1])*/

	fmt.Println(user_id, price)

	if err != nil {
		fmt.Println("Error reading:", err.Error())
		return micro.Prep{3, nil, user_id}
	}
	fmt.Println(password)
	db, err := sql.Open("mysql" /*password+*/, "dilawar:passord123@tcp(localhost:3306)/wallet_service")
	if err != nil {
		//conn.Write([]byte(strconv.Itoa(4))) // 4 = Error connecting to database
		fmt.Println(err)
		return micro.Prep{4, nil, user_id}
	}

	defer db.Close()

	results, err := db.Query("SELECT * FROM wallet WHERE user_id=?", user_id)
	if err != nil {
		//conn.Write([]byte(strconv.Itoa(5))) // Query went wrong
		fmt.Println(err)
		return micro.Prep{9, nil, user_id}
	}

	var wallet Wallet
	for results.Next() {

		err = results.Scan(&wallet.User_id, &wallet.Balance)
		if err != nil {
			//conn.Write([]byte(strconv.Itoa(6))) // 6 = Wrong format on wallet object
			return micro.Prep{10, nil, user_id} //
		}
	}
	fmt.Println("Wallet :", wallet)
	if wallet.User_id == 0 { // No user
		return micro.Prep{11, nil, user_id}
	}

	tx, err := db.Begin()
	if err != nil {
		//conn.Write([]byte(strconv.Itoa(7))) // Could not start transaction
		return micro.Prep{5, tx, user_id}
	}

	_, err = tx.Exec("UPDATE wallet SET balance=? WHERE user_id=?", wallet.Balance-price, user_id)
	//fmt.Println(res.RowsAffected())

	if wallet.Balance-price >= 0 {
		if err != nil {
			fmt.Println(err)
			tx.Rollback()
			//conn.Write([]byte(strconv.Itoa(8))) // 8 = Could not lock row
			return micro.Prep{6, tx, user_id}
		}
		return micro.Prep{1, tx, user_id}
	} else {
		tx.Rollback()
		return micro.Prep{12, tx, user_id} // 9 = Balance too low.
	}
}

func main() {
	list = micro.List{List: make(map[int]bool)}
	data, err := ioutil.ReadFile("../.config")
	if err != nil {
		fmt.Println("File reading error", err)
		os.Exit(1)
	}
	password := string(data)

	l, err := net.Listen(micro.CONN_TYPE, micro.WALLET_HOST+":"+CONN_PORT)
	if err != nil {
		fmt.Println("Error listening:", err.Error())
		os.Exit(1)
	}

	defer l.Close()
	fmt.Println("Listening on " + micro.CONN_HOST + ":" + CONN_PORT)
	for {
		fmt.Println("started new connection")

		conn, err := l.Accept()

		if err != nil {
			fmt.Println("Error accepting: ", err.Error())
			os.Exit(1)
		}
		go prepareAndCommit(conn, password)
		//time.Sleep(20 * time.Millisecond)
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
	micro.HandleCommit(conn, tx, user_id, list, prep.Id)
	//micro.Remove(list, user_id)
}
