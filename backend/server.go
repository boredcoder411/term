package main

import (
	"encoding/base64"
	"fmt"
	"io"
	"net/http"
  "os"
	"os/exec"
	"strings"

	"github.com/creack/pty"
	"github.com/gorilla/websocket"
	"encoding/json"
)

// BasicAuth middleware to require username and password
func BasicAuth(handler http.HandlerFunc, username, password string) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		auth := r.Header.Get("Authorization")
		if auth == "" {
			w.Header().Set("WWW-Authenticate", `Basic realm="Restricted"`)
			http.Error(w, "Unauthorized", http.StatusUnauthorized)
			return
		}

		// Decode the Base64 encoded credentials
		payload, err := base64.StdEncoding.DecodeString(strings.TrimPrefix(auth, "Basic "))
		if err != nil {
			http.Error(w, "Unauthorized", http.StatusUnauthorized)
			return
		}
		pair := strings.SplitN(string(payload), ":", 2)
		if len(pair) != 2 || pair[0] != username || pair[1] != password {
			http.Error(w, "Unauthorized", http.StatusUnauthorized)
			return
		}

		handler(w, r)
	}
}

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
  creds := strings.Split(os.Args[1], ":")
  username := creds[0]
  password := creds[1]

	fs := http.FileServer(http.Dir("dist"))
	http.Handle("/", BasicAuth(func(w http.ResponseWriter, r *http.Request) {
		fs.ServeHTTP(w, r)
	}, username, password))

	http.HandleFunc("/connect", HandleWebSocket)

	fmt.Println("Starting WebSocket server on :8080")
	if err := http.ListenAndServe(":8080", nil); err != nil {
		fmt.Println("Error starting server:", err)
	}
}

