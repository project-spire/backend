package dev

import (
	"context"
	"database/sql"
	"errors"
	"net/http"
	"spire/lobby/core"
	"strconv"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/golang-jwt/jwt/v5"
	_ "github.com/jackc/pgx/v5"
)

func HandleToken(c *gin.Context, x *core.Context) {
	type Request struct {
		AccountId int64 `json:"account_id" binding:"required"`
	}

	type Response struct {
		Token string `json:"token"`
	}

	var r Request
	if !core.Check(c.Bind(&r), c, http.StatusBadRequest) {
		return
	}

	var privilege string
	err := x.P.QueryRow(context.Background(),
		`SELECT a.privilege
		FROM account a
		WHERE a.id = $1`,
		r.AccountId).Scan(&privilege)
	if err != nil {
		if errors.Is(err, sql.ErrNoRows) {
			core.Check(err, c, http.StatusUnauthorized)
			return
		}
		core.Check(err, c, http.StatusInternalServerError)
		return
	}

	token := jwt.NewWithClaims(jwt.SigningMethodHS256, jwt.MapClaims{
		"aid": strconv.FormatInt(r.AccountId, 10),
		"prv": privilege,
		"exp": jwt.NewNumericDate(time.Now().Add(24 * time.Hour)),
	})
	tokenString, err := token.SignedString([]byte(x.S.AuthKey))
	if !core.Check(err, c, http.StatusInternalServerError) {
		return
	}

	c.JSON(http.StatusOK, Response{Token: tokenString})
}
