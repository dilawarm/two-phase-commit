package main

import (
	"encoding/binary"
	"fmt"
	"net"
)

func main() {
	p := make([]byte, 2048)
	conn, err := net.Dial("tcp", "localhost:3000")

	if err != nil {
		fmt.Printf("Some error %v", err)
		return
	}
	//reader := bufio.NewReader(os.Stdin)
	var firstNo = 1
	var secondNo = 2
	var thridNo = 1
	var fourthNo = 2

	b := make([]byte, 4)
	binary.LittleEndian.PutUint32(b, uint32(firstNo))
	conn.Write(b)
	binary.LittleEndian.PutUint32(b, uint32(secondNo))
	conn.Write(b)
	binary.LittleEndian.PutUint32(b, uint32(thridNo))
	conn.Write(b)
	binary.LittleEndian.PutUint32(b, uint32(fourthNo))
	conn.Write(b)

	//fmt.Fprintf(conn, firstNo)
	//_, err = bufio.NewReader(conn).Read(p)
	if err == nil {
		fmt.Printf("%s\n", p)
	} else {
		fmt.Printf("Some error %v\n", err)
	}

	conn.Close()
}
