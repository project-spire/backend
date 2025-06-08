package main

import (
	"fmt"
	"io"
	"log"
	"os"
	"sync"

	"github.com/gin-gonic/gin"
	"spire/lobby/internal/core"
	"spire/lobby/internal/route"
)

func main() {
	ctx := core.NewContext()

	f, _ := os.Create("gin.log")
	gin.DefaultWriter = io.MultiWriter(f, os.Stdout)
	log.SetOutput(gin.DefaultWriter)

	r := route.NewRouter(ctx)

	wg := sync.WaitGroup{}
	wg.Add(1)

	go func() {
		defer ctx.Close()
		defer wg.Done()

		listenAddr := fmt.Sprintf(":%d", ctx.S.ListenPort)
		if err := r.RunTLS(listenAddr, ctx.S.CertificateFile, ctx.S.PrivateKeyFile); err != nil {
			panic(err)
		}
	}()

	wg.Wait()
}
