package character

import (
	"context"
	"net/http"

	"github.com/gin-gonic/gin"
	"spire/lobby/core"
)

func HandleCharacterCreate(c *gin.Context, x *core.Context) {
	type Request struct {
		AccountID     int64  `json:"account_id" binding:"required"`
		CharacterName string `json:"character_name" binding:"required"`
		Race          string `json:"race" binding:"required"`
	}

	type Response struct {
		CharacterID int64 `json:"character_id"`
	}

	var r Request
	if !core.Check(c.Bind(&r), c, http.StatusBadRequest) {
		return
	}

	characterID := x.GenerateID()
	err := x.P.QueryRow(context.Background(),
		"INSERT INTO character (id, account_id, name, race) VALUES ($1, $2, $3, $4) RETURNING id",
		r.AccountID, r.CharacterName, r.Race).Scan(&characterID)
	if err != nil {
		core.Check(err, c, http.StatusInternalServerError)
		return
	}

	c.JSON(http.StatusOK, Response{CharacterID: characterID})
}
