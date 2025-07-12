package dev

import (
	"context"
	"fmt"
	"net/http"
	"spire/lobby/core"
	"unicode/utf8"

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
		AccountId int64 `json:"account_id"`
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

	ctx := context.Background()
	tx, err := x.P.Begin(ctx)
	if err != nil {
		core.Check(err, c, http.StatusInternalServerError)
		return
	}
	defer tx.Rollback(ctx)

	accountId := x.GenerateID()
	_, err = tx.Exec(ctx,
		`INSERT INTO account (id, platform, platform_id)
		VALUES ($1, 'Dev', 0)`,
		accountId)
	if err != nil {
		core.Check(err, c, http.StatusInternalServerError)
		return
	}

	_, err = tx.Exec(ctx,
		`INSERT INTO dev_account (id, account_id) VALUES ($1, $2)`,
		r.DevId,
		accountId)
	if err != nil {
		core.Check(err, c, http.StatusInternalServerError)
		return
	}

	if tx.Commit(ctx) != nil {
		core.Check(err, c, http.StatusInternalServerError)
		return
	}

	c.JSON(http.StatusOK, Response{
		AccountId: accountId,
	})
}
