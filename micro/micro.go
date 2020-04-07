package micro

import (
	"database/sql"
	"fmt"
	"net"
	"strconv"
	"strings"
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

	buf := make([]byte, 2)
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
	PreparedList.Mux.Lock()
	for i := 0; i < len(PreparedList.List); i++ {
		if user_id == PreparedList.List[i] {
			PreparedList.List[i] = PreparedList.List[len(PreparedList.List)-1] // Copy last element to index i.
			PreparedList.List[len(PreparedList.List)-1] = 0                    // Erase last element (write zero value).
			PreparedList.List = PreparedList.List[:len(PreparedList.List)-1]
		}
	}
	PreparedList.Mux.Unlock()
}
