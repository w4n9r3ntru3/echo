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
	const EOF = "EOF encountered"
	const QUIT = ":quit"
	wait := func() { time.Sleep(1 * time.Second) }

	args := os.Args
	port, err := strconv.Atoi(args[1])
	if err != nil {
		log.Fatalln("Invalid arguments")
	}

	localhost := fmt.Sprintf("localhost:%d", port)

	conn, err := net.Dial("tcp", localhost)
	if err != nil {
		log.Fatalf("Connection to %s failed", localhost)
	}

	fmt.Println("Connected to", localhost)

	var wg sync.WaitGroup

	ioChan := make(chan string)

	stdinScanner := bufio.NewScanner(os.Stdin)
	connScanner := bufio.NewScanner(conn)

	// IO thread
	wg.Add(1)
	go func() {
		defer wg.Done()

		for {
			fmt.Println("Write a message:")

			ok := stdinScanner.Scan()
			if !ok {
				log.Fatalln(EOF)
			}
			if err := stdinScanner.Err(); err != nil {
				log.Fatalln(err)
			}
			txt := stdinScanner.Text()

			ioChan <- txt

			if txt == QUIT {
				break
			}

			output := <-ioChan

			fmt.Println(output)

			wait()
		}
	}()

	// communication thread
	wg.Add(1)
	go func() {
		defer wg.Done()

		for {
			snd := <-ioChan
			if snd == QUIT {
				break
			}

			_, err := conn.Write([]byte(snd + "\n"))
			if err != nil {
				log.Fatalln(err)
			}

			ok := connScanner.Scan()
			if !ok {
				log.Fatalln(EOF)
			}
			if err := connScanner.Err(); err != nil {
				log.Fatalln(err)
			}

			ioChan <- connScanner.Text()

			wait()
		}
	}()

	wg.Wait()
}
