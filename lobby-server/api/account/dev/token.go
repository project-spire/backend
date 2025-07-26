package dev

import (
	"context"
	"errors"
	"net/http"
	"time"

	"github.com/geldata/gel-go/gelerr"
	"github.com/geldata/gel-go/geltypes"
	"github.com/gin-gonic/gin"
	"github.com/golang-jwt/jwt/v5"
	"spire/lobby/core"
)

func HandleToken(c *gin.Context, x *core.Context) {
	type Request struct {
		AccountId geltypes.UUID `json:"account_id" binding:"required"`
	}

	type Response struct {
		Token string `json:"token"`
	}

	var r Request
	if !core.Check(c.Bind(&r), c, http.StatusBadRequest) {
		return
	}

	query := `
		SELECT DevAccount { 
			id
		}
		FILTER .id = <uuid>id`
	args := map[string]interface{}{"id": r.AccountId}

	if err := x.D.QuerySingle(context.Background(), query, nil, args); err != nil {
		var gelErr gelerr.Error
		if errors.As(err, &gelErr) && !gelErr.Category(gelerr.NoDataError) {
			core.Check(err, c, http.StatusUnauthorized)
			return
		}

		core.Check(err, c, http.StatusInternalServerError)
		return
	}

	token := jwt.NewWithClaims(jwt.SigningMethodHS256, jwt.MapClaims{
		"aid": r.AccountId,
		"exp": jwt.NewNumericDate(time.Now().Add(24 * time.Hour)),
	})
	tokenString, err := token.SignedString([]byte(x.S.TokenKey))
	if !core.Check(err, c, http.StatusInternalServerError) {
		return
	}

	c.JSON(http.StatusOK, Response{Token: tokenString})
}
