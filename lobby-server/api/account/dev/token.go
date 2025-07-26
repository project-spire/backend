package dev

import (
	"context"
	"net/http"
	"time"

	"github.com/geldata/gel-go/geltypes"
	"github.com/gin-gonic/gin"
	"github.com/golang-jwt/jwt/v5"
	"spire/lobby/core"
)

func HandleToken(c *gin.Context, x *core.Context) {
	type Request struct {
		AccountId string `json:"account_id" binding:"required"`
	}

	type Response struct {
		Token string `json:"token"`
	}

	var r Request
	if !core.Check(c.Bind(&r), c, http.StatusBadRequest) {
		return
	}
	accountId, err := geltypes.ParseUUID(r.AccountId)
	if !core.Check(err, c, http.StatusBadRequest) {
		return
	}

	query := `SELECT exists(SELECT DevAccount FILTER .id = <uuid>$id)`
	args := map[string]interface{}{"id": accountId}

	var accountExists bool
	if err := x.D.QuerySingle(context.Background(), query, &accountExists, args); err != nil {
		core.Check(err, c, http.StatusInternalServerError)
		return
	}

	if !accountExists {
		core.Check(err, c, http.StatusUnauthorized)
		return
	}

	token := jwt.NewWithClaims(jwt.SigningMethodHS256, jwt.MapClaims{
		"aid": accountId.String(),
		"exp": jwt.NewNumericDate(time.Now().Add(24 * time.Hour)),
	})
	tokenString, err := token.SignedString([]byte(x.S.TokenKey))
	if !core.Check(err, c, http.StatusInternalServerError) {
		return
	}

	c.JSON(http.StatusOK, Response{Token: tokenString})
}
