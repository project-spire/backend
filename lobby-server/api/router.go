package api

import (
	"github.com/gin-gonic/gin"
	"spire/lobby/api/account/dev"
	"spire/lobby/api/character"
	"spire/lobby/core"
	"spire/lobby/middleware"
)

func NewRouter(ctx *core.Context) *gin.Engine {
	r := gin.Default()

	r.GET("/ping")

	r.POST("/account/dev/create", func(c *gin.Context) { dev.HandleCreate(c, ctx) })
	r.POST("/account/dev/me", func(c *gin.Context) { dev.HandleMe(c, ctx) })
	r.POST("/account/dev/token", func(c *gin.Context) { dev.HandleToken(c, ctx) })

	characterRoutes := r.Group("/character")
	characterRoutes.Use(middleware.Authenticate(ctx))
	{
		characterRoutes.POST("/create", func(c *gin.Context) { character.HandleCreate(c, ctx) })
		characterRoutes.POST("/list", func(c *gin.Context) { character.HandleList(c, ctx) })
	}

	return r
}
