package middleware

import (
	"fmt"
	"net/http"
	"strings"

	"github.com/geldata/gel-go/geltypes"
	"github.com/gin-gonic/gin"
	"github.com/golang-jwt/jwt/v5"
	"spire/lobby/core"
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
			return []byte(x.S.TokenKey), nil
		})
		if err != nil {
			c.AbortWithStatus(http.StatusUnauthorized)
			return
		}

		if claims, ok := token.Claims.(jwt.MapClaims); ok && token.Valid {
			accountIdStr, okAid := claims["aid"].(string)
			//privilege, okPrv := claims["prv"].(string)

			if !okAid {
				c.AbortWithStatus(http.StatusUnauthorized)
				return
			}

			accountId, err := geltypes.ParseUUID(accountIdStr)
			if err != nil {
				core.Check(err, c, http.StatusUnauthorized)
				return
			}

			c.Set("account_id", accountId)
			//c.Set("privilege", privilege)

			c.Next()
		} else {
			c.AbortWithStatus(http.StatusUnauthorized)
		}
	}
}
