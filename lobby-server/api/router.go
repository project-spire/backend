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

	r.POST("/account/dev/create", func(c *gin.Context) { dev.HandleAccountDevCreate(c, ctx) })
	r.POST("/account/dev/me", func(c *gin.Context) { dev.HandleAccountDevMe(c, ctx) })
	r.POST("/account/dev/token", func(c *gin.Context) { dev.HandleAccountToken(c, ctx) })

	characterRoutes := r.Group("/character")
	characterRoutes.Use(middleware.Authenticate(ctx))
	{
		characterRoutes.POST("/create", func(c *gin.Context) { character.HandleCharacterCreate(c, ctx) })
		characterRoutes.POST("/list", func(c *gin.Context) { character.HandleCharacterList(c, ctx) })
	}

	return r
}
