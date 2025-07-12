package character

import (
	"context"
	"net/http"
	context2 "spire/lobby/core"

	"github.com/gin-gonic/gin"
)

func HandleCharacterCreate(c *gin.Context, x *context2.Context) {
	type Request struct {
		CharacterName string `json:"character_name" binding:"required"`
		Race          string `json:"race" binding:"required"`
	}

	type Response struct {
		CharacterId int64 `json:"character_id"`
	}

	var r Request
	if !context2.Check(c.Bind(&r), c, http.StatusBadRequest) {
		return
	}

	accountId := c.MustGet("account_id").(int64)
	characterId := x.GenerateID()

	_, err := x.P.Exec(context.Background(),
		"INSERT INTO character (id, account_id, name, race) VALUES ($1, $2, $3, $4)",
		characterId, accountId, r.CharacterName, r.Race)
	if err != nil {
		context2.Check(err, c, http.StatusInternalServerError)
		return
	}

	c.JSON(http.StatusOK, Response{CharacterId: characterId})
}
