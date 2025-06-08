package dev

import (
	"context"
	"database/sql"
	"errors"
	"net/http"

	"github.com/gin-gonic/gin"
	"spire/lobby/internal/core"
)

func HandleAccountDevMe(c *gin.Context, x *core.Context) {
	type Request struct {
		DevID string `json:"dev_id" binding:"required"`
	}

	type Response struct {
		Found     bool   `json:"found"`
		AccountID uint64 `json:"account_id"`
	}

	var r Request
	if !core.Check(c.Bind(&r), c, http.StatusBadRequest) {
		return
	}

	found := true
	var accountID uint64 = 0
	err := x.P.QueryRow(context.Background(), "SELECT account_id FROM dev_accounts WHERE id=$1", r.DevID).Scan(&accountID)
	if err != nil {
		if !errors.Is(err, sql.ErrNoRows) {
			core.Check(err, c, http.StatusInternalServerError)
			return
		}

		found = false
	}

	c.JSON(http.StatusOK, Response{
		Found:     found,
		AccountID: accountID,
	})
}
