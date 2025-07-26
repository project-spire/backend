package dev

import (
	"context"
	"fmt"
	"net/http"
	"spire/lobby/core"
	"unicode/utf8"

	"github.com/geldata/gel-go/geltypes"
	"github.com/gin-gonic/gin"
)

const (
	DevIdMaxLength = 16
	DevIdMinLength = 4
)

func HandleCreate(c *gin.Context, x *core.Context) {
	type Request struct {
		DevId string `json:"dev_id" binding:"required"`
	}

	type Response struct {
		AccountId geltypes.UUID `json:"account_id"`
	}

	var r Request
	if !core.Check(c.Bind(&r), c, http.StatusBadRequest) {
		return
	}

	devIdLength := utf8.RuneCountInString(r.DevId)
	if devIdLength < DevIdMinLength || devIdLength > DevIdMaxLength {
		c.AbortWithStatusJSON(http.StatusNotAcceptable, gin.H{
			"error": fmt.Sprintf("Device Id length must be between %d and %d", DevIdMinLength, DevIdMaxLength),
		})
		return
	}

	query := `
		SELECT (
			INSERT DevAccount { 
				dev_id := <str>$dev_id
			}
		) { id };`
	args := map[string]interface{}{"dev_id": r.DevId}

	var accountId geltypes.UUID
	if err := x.D.QuerySingle(context.Background(), query, &accountId, args); err != nil {
		core.Check(err, c, http.StatusInternalServerError)
		return
	}

	c.JSON(http.StatusOK, Response{
		AccountId: accountId,
	})
}
