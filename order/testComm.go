package main

import (
	"bufio"
	"fmt"
	"net"
	"os"
	"strings"
	"unicode"
)

func main() {
	p := make([]byte, 2048)
	for {
		conn, err := net.Dial("tcp", "localhost:3334")
		if err != nil {
			fmt.Printf("Some error %v", err)
			return
		}
		reader := bufio.NewReader(os.Stdin)
		var text = ""
		var firstNo = ""
		for {
			fmt.Print("first number: \n")
			firstNo, _ = reader.ReadString('\n')
			firstNo = firstNo[:len(firstNo)-1]
			if isInt(firstNo) {
				text += firstNo
				break
			}
		}
		var operator = ""
		for {
			fmt.Print("operator: (*, /, +, -)\n")
			operator, _ = reader.ReadString('\n')
			operator = operator[:len(operator)-1]
			if (strings.Contains(operator, "+") || strings.Contains(operator, "-") || strings.Contains(operator, "*") || strings.Contains(operator, "/")) && len(operator) == 1 {
				fmt.Print(operator)
				text += " " + operator + " "
				break
			}
		}
		var secondNo = ""
		for {
			fmt.Print("second number: \n")
			secondNo, _ = reader.ReadString('\n')
			secondNo = secondNo[:len(secondNo)-1]
			if isInt(secondNo) {
				text += secondNo + " "
				break
			}
		}
		fmt.Fprintf(conn, text)
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
