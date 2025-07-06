package character

import (
	"context"
	"net/http"

	"github.com/gin-gonic/gin"
	"spire/lobby/core"
)

func HandleCharacterList(c *gin.Context, x *core.Context) {
	type Request struct {
		AccountID int64 `json:"account_id" binding:"required"`
	}

	type Response struct {
		Characters []Character `json:"characters"`
	}

	var r Request
	if !core.Check(c.Bind(&r), c, http.StatusBadRequest) {
		return
	}

	rows, err := x.P.Query(context.Background(),
		"SELECT id, name, race FROM character WHERE account_id=$1", r.AccountID)
	if err != nil {
		core.Check(err, c, http.StatusInternalServerError)
		return
	}
	defer rows.Close()

	characters := make([]Character, 0)

	for rows.Next() {
		var characterID int64
		var characterName string
		var characterRace string
		if err := rows.Scan(&characterID, &characterName, &characterRace); err != nil {
			core.Check(err, c, http.StatusInternalServerError)
			return
		}
		characters = append(characters, Character{
			ID:   characterID,
			Name: characterName,
			Race: characterRace,
		})
	}

	c.JSON(http.StatusOK, Response{Characters: characters})
}
