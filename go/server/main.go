package main

import (
	"bufio"
	"fmt"
	"log"
	"net"
	"os"
	"strconv"
	"sync"
	"time"
)

func main() {
	wait := func() { time.Sleep(1 * time.Second) }

	args := os.Args
	port, err := strconv.Atoi(args[1])
	if err != nil {
		log.Fatalln("Invalid arguements")
	}

	localhost := fmt.Sprintf("localhost:%d", port)

	server, err := net.Listen("tcp", localhost)
	if err != nil {
		log.Fatalf("Listening to port %d failed\n", port)
	}

	fmt.Println("Listening to", localhost)

	connChan := make(chan net.Conn)
	strChan := make(chan string)

	var wg sync.WaitGroup

	cliMutex := sync.Mutex{}
	clients := make([]net.Conn, 0)

	mutexReadWrite := sync.Mutex{}

	wg.Add(1)
	go func() {
		defer wg.Done()
		for {
			conn, err := server.Accept()
			if err != nil {
				continue
			}

			fmt.Println("Connected to", conn)

			connChan <- conn

			reader := bufio.NewReader(conn)

			go func() {
				for {
					mutexReadWrite.Lock()
					line, prefix, err := reader.ReadLine()
					mutexReadWrite.Unlock()

					if prefix || err != nil {
						fmt.Printf("Connection to %s closed\n", conn)
						break
					}

					strChan <- string(line)
				}
			}()

			wait()
		}
	}()

	wg.Add(1)
	go func() {
		defer wg.Done()
		for {
			conn := <-connChan
			cliMutex.Lock()
			clients = append(clients, conn)
			cliMutex.Unlock()

			wait()
		}
	}()

	wg.Add(1)
	go func() {
		defer wg.Done()
		for str := range strChan {
			fmt.Println(str, clients)
			cliMutex.Lock()
			newClients := make([]net.Conn, 0)
			for _, cli := range clients {
				str := fmt.Sprintln(str)
				mutexReadWrite.Lock()
				_, err := cli.Write([]byte(str))
				mutexReadWrite.Unlock()
				if err != nil {
					newClients = append(newClients, cli)
				}
			}
			clients = newClients
			cliMutex.Unlock()

			wait()
		}
	}()

	wg.Wait()
}
