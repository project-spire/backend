package core

import (
	"context"
	"fmt"
	"math"
	"os"
	"strconv"
	"strings"

	"github.com/bwmarrin/snowflake"
	"github.com/jackc/pgx/v5/pgxpool"
)

type Context struct {
	S *Settings
	P *pgxpool.Pool
	N *snowflake.Node
}

func (c *Context) GenerateID() int64 {
	return int64(c.N.Generate())
}

type Settings struct {
	AuthKey string
}

type NetworkSettings struct {
	DbHost     string
	DbPort     int
	DbName     string
	DbUser     string
	DbPassword string

	ListenPort int

	CertificateFile string
	PrivateKeyFile  string

	NodeID uint16
}

func NewContext(ns *NetworkSettings) *Context {
	s := newSettings()

	pool, err := pgxpool.New(context.Background(), fmt.Sprintf(
		"postgresql://%s:%s@%s:%d/%s?sslmode=disable",
		ns.DbUser, ns.DbPassword, ns.DbHost, ns.DbPort, ns.DbName))
	if err != nil {
		panic(err)
	}

	node, err := snowflake.NewNode(int64(readIntEnv("NODE_ID", math.MaxInt16)))
	if err != nil {
		panic(err)
	}

	return &Context{
		S:  s,
		P:  pool,
		ID: node,
	}
}

func (c *Context) Close() {
	c.P.Close()
}

func newSettings() *Settings {
	s := &Settings{}

	s.AuthKey = readFileEnv("SPIRE_AUTH_KEY_FILE")

	return s
}

func NewNetworkSettings() *NetworkSettings {
	s := &NetworkSettings{}

	s.DbHost = readEnv("SPIRE_DB_HOST")
	s.DbPort = readIntEnv("SPIRE_DB_PORT", math.MaxUint16)
	s.DbName = readEnv("SPIRE_DB_NAME")
	s.DbUser = readEnv("SPIRE_DB_USER")
	s.DbPassword = readFileEnv("SPIRE_DB_PASSWORD_FILE")

	s.ListenPort = readIntEnv("SPIRE_LOBBY_PORT", math.MaxUint16)

	s.CertificateFile = readEnv("SPIRE_LOBBY_CERTIFICATE_FILE")
	s.PrivateKeyFile = readEnv("SPIRE_LOBBY_PRIVATE_KEY_FILE")

	return s
}

func readEnv(key string) string {
	s := strings.TrimSpace(os.Getenv(key))
	if len(s) == 0 {
		panic(fmt.Errorf("environment variable %s not set", key))
	}
	return s
}

func readIntEnv(key string, max int) int {
	s := readEnv(key)
	i, err := strconv.Atoi(s)
	if err != nil {
		panic(err)
	}
	if i > max {
		panic(fmt.Errorf("environment variable %s out of range: %d", key, i))
	}

	return i
}

func readFileEnv(key string) string {
	data, err := os.ReadFile(os.Getenv("SPIRE_DB_PASSWORD_FILE"))
	if err != nil {
		panic(err)
	}
	s := strings.TrimSpace(string(data))
	if len(s) == 0 {
		panic(fmt.Errorf("empty file from environment variable %s", key))
	}

	return s
}
