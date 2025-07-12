package character

import (
	"context"
	"net/http"
	context2 "spire/lobby/core"

	"github.com/gin-gonic/gin"
)

func HandleCharacterList(c *gin.Context, x *context2.Context) {
	type Response struct {
		Characters []Character `json:"characters"`
	}

	accountId := c.MustGet("account_id").(int64)

	rows, err := x.P.Query(context.Background(),
		"SELECT id, name, race FROM character WHERE account_id=$1", accountId)
	if err != nil {
		context2.Check(err, c, http.StatusInternalServerError)
		return
	}
	defer rows.Close()

	characters := make([]Character, 0)

	for rows.Next() {
		var characterId int64
		var characterName string
		var characterRace string
		if err := rows.Scan(&characterId, &characterName, &characterRace); err != nil {
			context2.Check(err, c, http.StatusInternalServerError)
			return
		}
		characters = append(characters, Character{
			Id:   characterId,
			Name: characterName,
			Race: characterRace,
		})
	}

	c.JSON(http.StatusOK, Response{Characters: characters})
}
