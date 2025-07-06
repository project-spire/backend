package dev

import (
	"context"
	"fmt"
	"net/http"
	"unicode/utf8"

	"github.com/gin-gonic/gin"
	"spire/lobby/core"
)

const (
	DevIdMaxLength = 16
	DevIdMinLength = 4
)

func HandleAccountDevCreate(c *gin.Context, x *core.Context) {
	type Request struct {
		DevID string `json:"dev_id" binding:"required"`
	}

	type Response struct {
		AccountID int64 `json:"account_id"`
	}

	var r Request
	if !core.Check(c.Bind(&r), c, http.StatusBadRequest) {
		return
	}

	devIdLength := utf8.RuneCountInString(r.DevID)
	if devIdLength < DevIdMinLength || devIdLength > DevIdMaxLength {
		c.AbortWithStatusJSON(http.StatusNotAcceptable, gin.H{
			"error": fmt.Sprintf("Device ID length must be between %d and %d", DevIdMinLength, DevIdMaxLength),
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

	accountID := x.GenerateID()
	_, err = tx.Exec(ctx,
		`INSERT INTO account (id, platform, platform_id)
		VALUES (accountID, 'Dev', 0)`)
	if err != nil {
		core.Check(err, c, http.StatusInternalServerError)
		return
	}

	_, err = tx.Exec(ctx,
		`INSERT INTO dev_account (id, account_id) VALUES ($1, $2)`,
		r.DevID,
		accountID)
	if err != nil {
		core.Check(err, c, http.StatusInternalServerError)
		return
	}

	if tx.Commit(ctx) != nil {
		core.Check(err, c, http.StatusInternalServerError)
		return
	}

	c.JSON(http.StatusOK, Response{
		AccountID: accountID,
	})
}
