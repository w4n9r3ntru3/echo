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
		log.Fatalln("Invalid arguments")
	}

	localhost := fmt.Sprintf("localhost:%d", port)

	client, err := net.Dial("tcp", localhost)
	if err != nil {
		log.Fatalf("Connection to %s failed", localhost)
	}

	fmt.Println("Connected to", localhost)

	var wg sync.WaitGroup

	inChan := make(chan string)
	outChan := make(chan string)

	stdinReader := bufio.NewReader(os.Stdin)

	cliReadWrite := bufio.NewReadWriter(bufio.NewReader(client), bufio.NewWriter(client))

	mutexClient := sync.Mutex{}

	// input thread
	wg.Add(1)
	go func() {
		defer wg.Done()
		for {
			line, prefix, err := stdinReader.ReadLine()
			if prefix || err != nil {
				log.Fatalln("Read from stdin failed")
			}
			fmt.Println("sent")
			inChan <- string(line) + "\n"

			wait()
		}
	}()

	// output thread
	wg.Add(1)
	go func() {
		defer wg.Done()
		for {
			words := <-outChan
			fmt.Println(words)

			wait()
		}
	}()

	// client server
	wg.Add(1)
	go func() {
		defer wg.Done()
		for {
			mutexClient.Lock()
			str, prefix, err := cliReadWrite.ReadLine()
			mutexClient.Unlock()
			if prefix || err != nil {
				log.Fatalln("Connection broken")
			}
			outChan <- string(str)

			wait()
		}
	}()

	wg.Add(1)
	go func() {
		defer wg.Done()
		for {
			str := <-inChan

			mutexClient.Lock()
			_, err := cliReadWrite.Write([]byte(str))
			mutexClient.Unlock()
			if err != nil {
				log.Fatalln("Connection broken")
			}

			wait()
		}
	}()
	wg.Wait()
}
