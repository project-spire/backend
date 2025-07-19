package middleware

import (
	"fmt"
	"github.com/gin-gonic/gin"
	"github.com/golang-jwt/jwt/v5"
	"net/http"
	"spire/lobby/core"
	"strings"
)

func Authenticate(x *core.Context) gin.HandlerFunc {
	return func(c *gin.Context) {
		authHeader := c.GetHeader("Authorization")
		if authHeader == "" {
			c.AbortWithStatus(http.StatusUnauthorized)
			return
		}

		// Expected header format: "Bearer <token>"
		parts := strings.SplitN(authHeader, " ", 2)
		if parts[0] != "Bearer" {
			c.AbortWithStatus(http.StatusUnauthorized)
			return
		}
		tokenString := parts[1]

		token, err := jwt.Parse(tokenString, func(token *jwt.Token) (interface{}, error) {
			if _, ok := token.Method.(*jwt.SigningMethodHMAC); !ok {
				return nil, fmt.Errorf("unexpected signing method: %v", token.Header["alg"])
			}
			return []byte(x.S.AuthKey), nil
		})
		if err != nil {
			c.AbortWithStatus(http.StatusUnauthorized)
			return
		}

		if claims, ok := token.Claims.(jwt.MapClaims); ok && token.Valid {
			aid, okAid := claims["aid"].(float64)
			privilege, okPrv := claims["prv"].(string)

			if !okAid || !okPrv {
				c.AbortWithStatus(http.StatusUnauthorized)
				return
			}

			accountId := int64(aid)

			c.Set("account_id", accountId)
			c.Set("privilege", privilege)

			c.Next()
		} else {
			c.AbortWithStatus(http.StatusUnauthorized)
		}
	}
}
