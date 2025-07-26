package dev

import (
	"context"
	"errors"
	"github.com/geldata/gel-go/gelerr"
	"net/http"

	"github.com/geldata/gel-go/geltypes"
	"github.com/gin-gonic/gin"
	"spire/lobby/core"
)

func HandleMe(c *gin.Context, x *core.Context) {
	type Request struct {
		DevId string `json:"dev_id" binding:"required"`
	}

	type Response struct {
		Found     bool          `json:"found"`
		AccountId geltypes.UUID `json:"account_id"`
	}

	var r Request
	if !core.Check(c.Bind(&r), c, http.StatusBadRequest) {
		return
	}

	query := `
		SELECT DevAccount { 
			id
		}
		FILTER .dev_id = <str>$dev_id`
	args := map[string]interface{}{"dev_id": r.DevId}

	found := true
	var accountId geltypes.UUID
	if err := x.D.QuerySingle(context.Background(), query, &accountId, args); err != nil {
		var gelErr gelerr.Error
		if errors.As(err, &gelErr) && !gelErr.Category(gelerr.NoDataError) {
			core.Check(err, c, http.StatusInternalServerError)
			return
		}

		found = false
	}

	c.JSON(http.StatusOK, Response{
		Found:     found,
		AccountId: accountId,
	})
}
