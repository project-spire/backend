package core

import (
	"log"

	"github.com/gin-gonic/gin"
)

func Check(err error, c *gin.Context, status int) bool {
	if err == nil {
		return true
	}

	c.AbortWithStatus(status)
	log.Printf("[ERROR] %s %s: %v", c.Request.Method, c.Request.URL.Path, err)
	return false
}
