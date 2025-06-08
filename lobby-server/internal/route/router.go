package route

import (
	"github.com/gin-gonic/gin"
	"spire/lobby/internal/core"
	"spire/lobby/internal/route/account"
	"spire/lobby/internal/route/account/dev"
	"spire/lobby/internal/route/character"
)

func NewRouter(ctx *core.Context) *gin.Engine {
	r := gin.Default()

	r.GET("/ping")

	r.POST("/account/auth", func(c *gin.Context) { account.HandleAccountAuth(c, ctx) })
	r.POST("/account/dev/create", func(c *gin.Context) { dev.HandleAccountDevCreate(c, ctx) })
	r.POST("/account/dev/me", func(c *gin.Context) { dev.HandleAccountDevMe(c, ctx) })
	r.POST("/character/create", func(c *gin.Context) { character.HandleCharacterCreate(c, ctx) })
	r.POST("/character/list", func(c *gin.Context) { character.HandleCharacterList(c, ctx) })

	return r
}
