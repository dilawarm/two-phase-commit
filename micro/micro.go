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
	List []int
	Mux  sync.Mutex
}

var PreparedList List

const CONN_HOST = "localhost"
const CONN_TYPE = "tcp"

func HandleCommit(conn net.Conn, tx *sql.Tx, user_id int) {

	buf := make([]byte, 4)
	_, err := conn.Read(buf)
	if err != nil {
		fmt.Println("Error reading:", err.Error())
	}

	data := binary.BigEndian.Uint32(buf[:4])
	id := int(data)
	fmt.Println("ID :", id)
	if id == 1 {
		err = tx.Commit()
		if err != nil {
			b := make([]byte, 2)
			binary.LittleEndian.PutUint16(b, uint16(10)) // Could not COMMIT
		}
		b := make([]byte, 2)
		binary.LittleEndian.PutUint16(b, uint16(2)) // 2 =																																																																																					 OK COMMIT
	} else if tx != nil {
		tx.Rollback()
		fmt.Println("Transaction rolled back")
	} else {
		fmt.Println("do nothing, transaction never started")
	}
	PreparedList.Mux.Lock()
	fmt.Println("LISTE: ", PreparedList.List)
	for i := 0; i < len(PreparedList.List); i++ {
		if user_id == PreparedList.List[i] {
			PreparedList.List[i] = PreparedList.List[len(PreparedList.List)-1] // Copy last element to index i.
			PreparedList.List[len(PreparedList.List)-1] = 0                    // Erase last element (write zero value).
			PreparedList.List = PreparedList.List[:len(PreparedList.List)-1]
		}
	}
	PreparedList.Mux.Unlock()
}
