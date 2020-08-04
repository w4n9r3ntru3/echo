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

	ioChan := make(chan string)
	cliChan := make(chan string)
	connChan := make(chan net.Conn)
	clients := make([]net.Conn, 0)

	var wg sync.WaitGroup

	// IO thread
	wg.Add(1)
	go func() {
		defer wg.Done()

		for {
			msg := <-ioChan
			fmt.Println("Broadcasting:", msg)

			wait()
		}
	}()

	// communication thread
	wg.Add(1)
	go func() {
		defer wg.Done()

		for {
			conn, err := server.Accept()
			if err != nil {
				log.Fatalln(err)
			}

			connChan <- conn

			fmt.Println("Listening to:", conn.LocalAddr())
			readConn := bufio.NewReader(conn)

			wg.Add(1)
			go func() {
				defer wg.Done()

				for {
					line := make([]byte, 0)
					prefix := true
					var err error
					for prefix {
						var pline []byte
						pline, prefix, err = readConn.ReadLine()
						line = append(line, pline...)

						if err != nil {
							ioChan <- fmt.Sprint(err)
							return
						}
					}

					if len(line) == 0 {
						continue
					}

					str := string(line)
					cliChan <- str + "\n"

					wait()
				}
			}()

			wait()
		}
	}()

	// update clients
	wg.Add(1)
	go func() {
		defer wg.Done()

		for {

			select {
			case cli := <-connChan:
				clients = append(clients, cli)
			default:
			}

			select {
			case msg := <-cliChan:

				onlineClients := make([]net.Conn, 0)
				for _, cli := range clients {
					_, err := cli.Write([]byte(msg))
					if err != nil {
						fmt.Printf("Connection to %s closed\n", cli.LocalAddr())
						continue
					}
					onlineClients = append(onlineClients, cli)
				}

				clients = onlineClients

				ioChan <- msg
				ioChan <- fmt.Sprintln(clients)

			default:
				wait()
			}

		}
	}()

	wg.Wait()
}
