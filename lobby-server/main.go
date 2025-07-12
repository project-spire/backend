package main

import (
	"fmt"
	"io"
	"log"
	"os"
	"spire/lobby/core"
	"sync"

	"github.com/gin-gonic/gin"
	"spire/lobby/api"
)

func main() {
	ns := core.NewNetworkSettings()
	ctx := core.NewContext(ns)

	f, _ := os.Create("gin.log")
	gin.DefaultWriter = io.MultiWriter(f, os.Stdout)
	log.SetOutput(gin.DefaultWriter)

	r := api.NewRouter(ctx)

	wg := sync.WaitGroup{}
	wg.Add(1)

	go func() {
		defer ctx.Close()
		defer wg.Done()

		listenAddr := fmt.Sprintf(":%d", ns.ListenPort)
		if err := r.RunTLS(listenAddr, ns.CertificateFile, ns.PrivateKeyFile); err != nil {
			panic(err)
		}
	}()

	wg.Wait()
}
