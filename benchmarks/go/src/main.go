package main

import (
	"fmt"
	"net/http"
	"time"
)

func root(w http.ResponseWriter, req *http.Request) {
	flusher := w.(http.Flusher)
	body := []byte("Hello World!")

	w.Header().Add("Content-Type", "text/html")
	w.Header().Add("Content-Length", fmt.Sprintf("%d", len(body)))
	w.Write(body)
	flusher.Flush()
}

func chunked(w http.ResponseWriter, req *http.Request) {
	flusher := w.(http.Flusher)

	w.Header().Add("Content-Type", "text/html")
	w.Header().Add("Transfer-Encoding", "chunked")
	flusher.Flush()

	for i := 0; i < 10; i++ {
		fmt.Fprintf(w, "%d\n", i)
		fmt.Println(i)
		flusher.Flush()
		time.Sleep(time.Second * 1)
	}
}

func main() {
	http.HandleFunc("/", root)
	http.HandleFunc("/chunked", chunked)

	http.ListenAndServe(":8080", nil)
}
