// Command cacheoff implements the Go build cache protocol with a miss-only
// policy. It accepts writes so the go command can finish an action, but every
// read misses and every temporary object is removed on close.
package main

import (
	"bufio"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"io"
	"os"
)

type request struct {
	ID       int64  `json:"ID"`
	Command  string `json:"Command"`
	BodySize int64  `json:"BodySize"`
}

type response struct {
	ID            int64    `json:"ID"`
	KnownCommands []string `json:"KnownCommands,omitempty"`
	Miss          bool     `json:"Miss,omitempty"`
	DiskPath      string   `json:"DiskPath,omitempty"`
	Size          int64    `json:"Size,omitempty"`
	Err           string   `json:"Err,omitempty"`
}

func main() {
	encoder := json.NewEncoder(os.Stdout)
	if err := encoder.Encode(response{
		KnownCommands: []string{"get", "put", "close"},
	}); err != nil {
		return
	}

	decoder := json.NewDecoder(bufio.NewReader(os.Stdin))
	var objects []string
	defer func() {
		for _, object := range objects {
			_ = os.Remove(object)
		}
	}()

	for {
		var req request
		if err := decoder.Decode(&req); err != nil {
			if err == io.EOF {
				return
			}
			return
		}

		switch req.Command {
		case "get":
			_ = encoder.Encode(response{ID: req.ID, Miss: true})
		case "put":
			object, size, err := receiveObject(decoder, req.BodySize)
			if err != nil {
				_ = encoder.Encode(response{ID: req.ID, Err: err.Error()})
				continue
			}
			objects = append(objects, object)
			_ = encoder.Encode(response{ID: req.ID, DiskPath: object, Size: size})
		case "close":
			_ = encoder.Encode(response{ID: req.ID})
			return
		default:
			_ = encoder.Encode(response{ID: req.ID, Err: fmt.Sprintf("unsupported command %q", req.Command)})
		}
	}
}

func receiveObject(decoder *json.Decoder, bodySize int64) (string, int64, error) {
	var encoded string
	if bodySize > 0 {
		if err := decoder.Decode(&encoded); err != nil {
			return "", 0, err
		}
	}
	body, err := base64.StdEncoding.DecodeString(encoded)
	if err != nil {
		return "", 0, err
	}
	if int64(len(body)) != bodySize {
		return "", 0, fmt.Errorf("cache object size %d does not match %d", len(body), bodySize)
	}
	object, err := os.CreateTemp("", "platform-bridge-cacheoff-")
	if err != nil {
		return "", 0, err
	}
	name := object.Name()
	if _, err := object.Write(body); err != nil {
		_ = object.Close()
		_ = os.Remove(name)
		return "", 0, err
	}
	if err := object.Close(); err != nil {
		_ = os.Remove(name)
		return "", 0, err
	}
	return name, int64(len(body)), nil
}
