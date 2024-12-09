package main

import (
	"fmt"
	"io"
	"net/http"
	"os/exec"

	"github.com/creack/pty"
	"github.com/gorilla/websocket"
  "encoding/json"
)

// {"event": "command", "content": "ls"}
// or
// {"event": "resize", "content": {"cols": 80, "rows": 24}}

// Upgrade configures the HTTP server to upgrade connections to WebSockets.
var upgrader = websocket.Upgrader{
	CheckOrigin: func(r *http.Request) bool {
		// Allow all origins (adjust for production).
		return true
	},
}

// HandleWebSocket handles the WebSocket connection.
func HandleWebSocket(w http.ResponseWriter, r *http.Request) {
	// Upgrade the connection to a WebSocket.
	conn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		fmt.Println("Error upgrading connection:", err)
		return
	}
	defer conn.Close()

	fmt.Println("Client connected")

	// Create a new PTY with bash.
	c := exec.Command("bash")
	f, err := pty.Start(c)
	if err != nil {
		fmt.Println("Error starting PTY:", err)
		return
	}
	defer f.Close()

	// Goroutine to forward PTY output to WebSocket.
	go func() {
		buf := make([]byte, 1024)
		for {
			n, err := f.Read(buf)
			if err != nil {
				if err != io.EOF {
					fmt.Println("Error reading from PTY:", err)
				}
				break
			}
			if err := conn.WriteMessage(websocket.TextMessage, buf[:n]); err != nil {
				fmt.Println("Error writing to WebSocket:", err)
				break
			}
		}
	}()

	// Forward WebSocket messages to PTY input.
	for {
		_, msg, err := conn.ReadMessage()

    var data map[string]interface{}
    json.Unmarshal(msg, &data)

    if data["event"] == "command" {
      data := data["content"].(string)
      if _, err := f.Write([]byte(data)); err != nil {
			  fmt.Println("Error writing to PTY:", err)
			  break
		  }
    } else if data["event"] == "resize" {
      // resize
      cols := int(data["content"].(map[string]interface{})["cols"].(float64))
      rows := int(data["content"].(map[string]interface{})["rows"].(float64))
      pty.Setsize(f, &pty.Winsize{Cols: uint16(cols), Rows: uint16(rows)})
    }

		if err != nil {
			fmt.Println("Error reading from WebSocket:", err)
			break
		}

		
	}
}

func main() {
  fs := http.FileServer(http.Dir("dist"))
  http.Handle("/", fs)

	http.HandleFunc("/connect", HandleWebSocket)

	fmt.Println("Starting WebSocket server on :8080")
	if err := http.ListenAndServe(":8080", nil); err != nil {
		fmt.Println("Error starting server:", err)
	}
}

