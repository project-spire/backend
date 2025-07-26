package core

import (
	"context"
	"log"
	"math"
	"os"
	"strconv"
	"strings"

	"github.com/geldata/gel-go"
	"github.com/geldata/gel-go/gelcfg"
	"github.com/geldata/gel-go/geltypes"
)

type Context struct {
	S *Settings
	D *gel.Client
}

type Settings struct {
	TokenKey string
}

type NetworkSettings struct {
	DbHost     string
	DbPort     int
	DbName     string
	DbUser     string
	DbPassword string

	ListenPort int

	TlsCertFile string
	TlsKeyFile  string

	NodeID uint16
}

func NewContext(ns *NetworkSettings) *Context {
	s := newSettings()

	client, err := gel.CreateClient(gelcfg.Options{
		Host:     ns.DbHost,
		Port:     ns.DbPort,
		User:     ns.DbUser,
		Password: geltypes.NewOptionalStr(ns.DbPassword),
		TLSOptions: gelcfg.TLSOptions{
			SecurityMode: gelcfg.TLSModeInsecure,
		},
	})
	if err != nil {
		log.Fatal(err)
	}

	if err = client.EnsureConnected(context.Background()); err != nil {
		log.Fatal(err)
	}

	return &Context{
		S: s,
		D: client,
	}
}

func (c *Context) Close() {
	if err := c.D.Close(); err != nil {
		log.Printf("Error closing database connection: %v", err)
	}
}

func newSettings() *Settings {
	s := &Settings{
		TokenKey: readFileEnv("SPIRE_TOKEN_KEY_FILE"),
	}

	return s
}

func NewNetworkSettings() *NetworkSettings {
	s := &NetworkSettings{}

	s.DbHost = readEnv("SPIRE_DB_HOST")
	s.DbPort = readIntEnv("SPIRE_DB_PORT", math.MaxUint16)
	s.DbName = readEnv("SPIRE_DB_NAME")
	s.DbUser = readEnv("SPIRE_DB_USER")
	s.DbPassword = readFileEnv("SPIRE_DB_PASSWORD_FILE")

	s.ListenPort = readIntEnv("SPIRE_LOBBY_SERVER_PORT", math.MaxUint16)

	s.TlsCertFile = readEnv("SPIRE_LOBBY_SERVER_TLS_CERT_FILE")
	s.TlsKeyFile = readEnv("SPIRE_LOBBY_SERVER_TLS_KEY_FILE")

	return s
}

func readEnv(key string) string {
	s := strings.TrimSpace(os.Getenv(key))
	if len(s) == 0 {
		log.Fatalf("environment variable %s not set", key)
	}
	return s
}

func readIntEnv(key string, max int) int {
	s := readEnv(key)
	i, err := strconv.Atoi(s)
	if err != nil {
		log.Fatal(err)
	}
	if i > max {
		log.Fatalf("environment variable %s out of range: %d", key, i)
	}

	return i
}

func readFileEnv(key string) string {
	data, err := os.ReadFile(os.Getenv(key))
	if err != nil {
		log.Fatal(err)
	}
	s := strings.TrimSpace(string(data))
	if len(s) == 0 {
		log.Fatalf("empty file from environment variable %s", key)
	}

	return s
}
