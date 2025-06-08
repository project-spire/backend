package core

import (
	"context"
	"fmt"
	"os"
	"strconv"
	"strings"

	"github.com/jackc/pgx/v5/pgxpool"
)

type Context struct {
	S *Settings
	P *pgxpool.Pool
}

type Settings struct {
	DbHost     string
	DbPort     int
	DbName     string
	DbUser     string
	DbPassword string

	ListenPort int

	CertificateFile string
	PrivateKeyFile  string
	AuthKey         string
}

func NewContext() *Context {
	s := newSettings()

	pool, err := pgxpool.New(context.Background(), fmt.Sprintf(
		"postgresql://%s:%s@%s:%d/%s?sslmode=disable",
		s.DbUser, s.DbPassword, s.DbHost, s.DbPort, s.DbName))
	if err != nil {
		panic(err)
	}

	return &Context{
		S: s,
		P: pool,
	}
}

func (c *Context) Close() {
	c.P.Close()
}

func newSettings() *Settings {
	s := &Settings{}

	s.DbHost = os.Getenv("SPIRE_DB_HOST")

	port, err := strconv.Atoi(os.Getenv("SPIRE_DB_PORT"))
	if err != nil {
		panic(err)
	}
	s.DbPort = port

	s.DbName = os.Getenv("SPIRE_DB_NAME")

	s.DbUser = os.Getenv("SPIRE_DB_USER")

	data, err := os.ReadFile(os.Getenv("SPIRE_DB_PASSWORD_FILE"))
	if err != nil {
		panic(err)
	}
	s.DbPassword = strings.TrimSpace(string(data))

	port, err = strconv.Atoi(os.Getenv("SPIRE_LOBBY_PORT"))
	if err != nil {
		panic(err)
	}
	s.ListenPort = port

	data, err = os.ReadFile(os.Getenv("SPIRE_AUTH_KEY_FILE"))
	if err != nil {
		panic(err)
	}
	s.AuthKey = strings.TrimSpace(string(data))

	s.CertificateFile = os.Getenv("SPIRE_LOBBY_CERTIFICATE_FILE")
	s.PrivateKeyFile = os.Getenv("SPIRE_LOBBY_PRIVATE_KEY_FILE")

	if !s.validate() {
		panic("Invalid settings")
	}

	return s
}

func (s *Settings) validate() bool {
	if (s.DbHost == "") || (s.DbName == "") || (s.DbUser == "") || (s.DbPassword == "") {
		return false
	}
	if s.AuthKey == "" {
		return false
	}

	return true
}
