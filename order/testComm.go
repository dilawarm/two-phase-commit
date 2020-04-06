package main

import (
	"bufio"
	"fmt"
	"net"
	"os"
)

func main() {
	p := make([]byte, 2048)
	conn, err := net.Dial("tcp", "localhost:3334")
	for {
		if err != nil {
			fmt.Printf("Some error %v", err)
			return
		}
		reader := bufio.NewReader(os.Stdin)
		var firstNo = ""

		fmt.Print("text to send: \n")
		firstNo, _ = reader.ReadString('\n')
		firstNo = firstNo[:len(firstNo)-1]

		fmt.Fprintf(conn, firstNo)
		_, err = bufio.NewReader(conn).Read(p)
		if err == nil {
			fmt.Printf("%s\n", p)
		} else {
			fmt.Printf("Some error %v\n", err)
		}

	}
	conn.Close()
}
