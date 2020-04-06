package main

import (
	"fmt"
	"io/ioutil"

	_ "github.com/go-sql-driver/mysql"
)

const (
	CONN_HOST = "localhost"
	CONN_PORT = "3334"
	CONN_TYPE = "tcp"
)

type Order struct {
	User_id int `json:"user_id"`
	Amount  int `json:"balance"`
}

func errorHandler(err error) {
	if err != nil {
		panic(err.Error())
	}
}


func handlePrepare(conn net.Conn, password string) *sql.Tx {
	p:= make([]byte, 2048)
	_, err := conn.Read(p)

	if err != nil {
		fmt.Println("Error reading: ", err.Error())
	}
	message := string(p)
	temp := strings.Split(message, " ")
	user_id, _ := strconv.Atoi(temp[0])
	amount, _:= strconv.Atoi(temp[1])

	for i := 0; i<amount; i++ {

	}

	if err != nil {
		fmt.Println("Error reading:", err.Error())
	}
	data, err := ioutil.ReadFile("../.config")
	if err != nil {
	  fmt.Println("File reading error", err)
	}
	fmt.Println("Contents of file:")
	fmt.Println(string(data))
	password := string(data)

	db, err:=sql.Open("mysql", "haavasma:")
}

func main() {
	data, err := ioutil.ReadFile("../.config")
	if err != nil {
		fmt.Println("File reading error", err)
		os.Exit(data)
	}
	fmt.Println("Contents of file:")
	fmt.Println(string(data))
	password:= string(data)

	1, err := net.Listen(CONN_TYPE, CONN_HOST+":"+CONN_PORT)
	if err != nil {
		fmt.Println("Error listening: ", err.Error())
		os.Exit(1)
	}
}
