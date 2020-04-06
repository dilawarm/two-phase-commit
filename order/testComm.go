package main

import (
	"bufio"
	"fmt"
	"net"
	"os"
	"unicode"
)

func main() {
	p := make([]byte, 2048)
	for {
		conn, err := net.Dial("tcp", "localhost:3333")
		if err != nil {
			fmt.Printf("Some error %v", err)
			return
		}
		reader := bufio.NewReader(os.Stdin)
		var firstNo = ""

		fmt.Print("first number: \n")
		firstNo, _ = reader.ReadString('\n')
		firstNo = firstNo[:len(firstNo)-1]

		fmt.Fprintf(conn, firstNo)
		_, err = bufio.NewReader(conn).Read(p)
		if err == nil {
			fmt.Printf("%s\n", p)
		} else {
			fmt.Printf("Some error %v\n", err)
		}
		conn.Close()
	}
}
func isInt(s string) bool {
	for _, c := range s {
		if !unicode.IsDigit(c) {
			return false
		}
	}
	return true
}
