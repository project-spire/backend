package dev

import (
	"context"
	"net/http"

	"github.com/gin-gonic/gin"
	"spire/lobby/internal/core"
)

func HandleAccountDevCreate(c *gin.Context, x *core.Context) {
	type Request struct {
		DevID string `json:"dev_id" binding:"required,min=4,max=16"`
	}

	type Response struct {
		AccountID uint64 `json:"account_id"`
	}

	var r Request
	if !core.Check(c.Bind(&r), c, http.StatusBadRequest) {
		return
	}

	ctx := context.Background()
	tx, err := x.P.Begin(ctx)
	if err != nil {
		core.Check(err, c, http.StatusInternalServerError)
		return
	}
	defer tx.Rollback(ctx)

	var accountID uint64
	err = tx.QueryRow(ctx,
		`INSERT INTO accounts (platform, platform_id)
		VALUES ('Dev', 0) RETURNING id`).Scan(&accountID)
	if err != nil {
		core.Check(err, c, http.StatusInternalServerError)
		return
	}

	_, err = tx.Exec(ctx,
		`INSERT INTO dev_accounts (id, account_id) VALUES ($1, $2)`,
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
