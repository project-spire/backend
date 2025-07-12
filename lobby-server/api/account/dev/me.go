package dev

import (
	"context"
	"database/sql"
	"errors"
	"net/http"
	context2 "spire/lobby/core"

	"github.com/gin-gonic/gin"
)

func HandleAccountDevMe(c *gin.Context, x *context2.Context) {
	type Request struct {
		DevId string `json:"dev_id" binding:"required"`
	}

	type Response struct {
		Found     bool  `json:"found"`
		AccountId int64 `json:"account_id"`
	}

	var r Request
	if !context2.Check(c.Bind(&r), c, http.StatusBadRequest) {
		return
	}

	found := true
	var accountId int64 = 0
	err := x.P.QueryRow(context.Background(), "SELECT account_id FROM dev_account WHERE id=$1", r.DevId).Scan(&accountId)
	if err != nil {
		if !errors.Is(err, sql.ErrNoRows) {
			context2.Check(err, c, http.StatusInternalServerError)
			return
		}

		found = false
	}

	c.JSON(http.StatusOK, Response{
		Found:     found,
		AccountId: accountId,
	})
}
