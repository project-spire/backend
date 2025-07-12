package character

import (
	"context"
	"net/http"
	"spire/lobby/core"

	"github.com/gin-gonic/gin"
)

func HandleCreate(c *gin.Context, x *core.Context) {
	type Request struct {
		CharacterName string `json:"character_name" binding:"required"`
		CharacterRace string `json:"character_race" binding:"required"`
	}

	type Response struct {
		CharacterId int64 `json:"character_id"`
	}

	var r Request
	if !core.Check(c.Bind(&r), c, http.StatusBadRequest) {
		return
	}

	accountId := c.MustGet("account_id").(int64)
	characterId := x.GenerateID()

	_, err := x.P.Exec(context.Background(),
		"INSERT INTO character (id, account_id, name, race) VALUES ($1, $2, $3, $4)",
		characterId, accountId, r.CharacterName, r.CharacterRace)
	if err != nil {
		core.Check(err, c, http.StatusInternalServerError)
		return
	}

	c.JSON(http.StatusOK, Response{CharacterId: characterId})
}
