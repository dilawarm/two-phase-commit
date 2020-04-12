package micro

import (
	"database/sql"
	"encoding/binary"
	"fmt"
	"net"
	"sync"
)

type Prep struct {
	Id      int
	Tx      *sql.Tx
	User_id int
}

type List struct {
	List map[int]bool
	Mux  sync.Mutex
}

//var PreparedList List

const CONN_HOST = "localhost"
const CONN_TYPE = "tcp"
const ORDER_HOST = "34.67.42.245"
const WALLET_HOST = "35.202.15.128"

func HandleCommit(conn net.Conn, tx *sql.Tx, user_id int, list List, prepMessage int) {

	buf := make([]byte, 4)
	_, err := conn.Read(buf)
	if err != nil {
		fmt.Println("Error reading:", err.Error())
	}
	list.Mux.Lock()
	if prepMessage != 11 {
		list.List[user_id] = false
	}
	list.Mux.Unlock()

	data := binary.BigEndian.Uint32(buf[:4])
	id := int(data)
	fmt.Println("ID :", id)
	if id == 1 {
		err = tx.Commit()
		if err != nil {
			b := make([]byte, 2)
			binary.LittleEndian.PutUint16(b, uint16(10)) // Could not COMMIT
			conn.Write(b)
		}
		b := make([]byte, 2)
		binary.LittleEndian.PutUint16(b, uint16(2)) // 2 =
		conn.Write(b)
	} else if tx != nil {
		tx.Rollback()
		fmt.Println("Transaction rolled back")
		b := make([]byte, 2)
		binary.LittleEndian.PutUint16(b, uint16(69))
		conn.Write(b)
	} else {
		fmt.Println("do nothing, transaction never started")
		b := make([]byte, 2)
		binary.LittleEndian.PutUint16(b, uint16(69))
		conn.Write(b)
	}
	//time.Sleep(200 * time.Millisecond)
	conn.Close()
}
