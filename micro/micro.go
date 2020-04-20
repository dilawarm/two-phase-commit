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

const CONN_HOST = ""
const CONN_TYPE = "tcp"
var ORDER_HOST = "localhost"  //"10.128.0.10"
var WALLET_HOST = "localhost" //"10.128.0.9"

func HandleCommit(conn net.Conn, tx *sql.Tx, user_id int, list List, prepMessage int) {
	// Read orchestrator response
	buf := make([]byte, 4)
	_, err := conn.Read(buf)
	if err != nil {
		fmt.Println("Error reading:", err.Error())
	}
	list.Mux.Lock()
	// If the user is in the hashmap it is set to false to free it for other threads
	if prepMessage != 3 {
		list.List[user_id] = false
	}
	list.Mux.Unlock()

	data := binary.BigEndian.Uint32(buf[:4])
	id := int(data)
	// Interpret the response
	if id == 1 {
		// commit changes
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
		// Rollback changes
		tx.Rollback()
		b := make([]byte, 2)
		binary.LittleEndian.PutUint16(b, uint16(7))
		conn.Write(b)
	} else {
		b := make([]byte, 2)
		binary.LittleEndian.PutUint16(b, uint16(8))
		conn.Write(b)
	}
	conn.Close()
}
