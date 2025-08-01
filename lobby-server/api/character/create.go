package character

import (
	"context"
	"github.com/geldata/gel-go/geltypes"
	"net/http"

	"github.com/gin-gonic/gin"
	"spire/lobby/core"
)

func HandleCreate(c *gin.Context, x *core.Context) {
	type Request struct {
		CharacterName string `json:"character_name" binding:"required"`
		CharacterRace string `json:"character_race" binding:"required"`
	}

	type Character struct {
		Id   string `json:"id" binding:"required"`
		Name string `json:"name" binding:"required"`
		Race string `json:"race" binding:"required"`
	}

	type Response struct {
		Character Character `json:"character"`
	}

	var r Request
	if !core.Check(c.Bind(&r), c, http.StatusBadRequest) {
		return
	}

	accountId := c.MustGet("account_id").(geltypes.UUID)

	query := `
		SELECT (
			INSERT Character { 
				name := <str>$name,
				race := <Race>$race,
				account := (
					SELECT Account
					FILTER .id = <uuid>$account_id
					LIMIT 1
				)
			}
		) { id };`
	args := map[string]interface{}{
		"name":       r.CharacterName,
		"race":       r.CharacterRace,
		"account_id": accountId,
	}

	var result struct {
		Id geltypes.UUID `gel:"id"`
	}
	if err := x.D.QuerySingle(context.Background(), query, &result, args); err != nil {
		core.Check(err, c, http.StatusInternalServerError)
		return
	}

	c.JSON(http.StatusOK, Response{Character: Character{
		Id:   result.Id.String(),
		Name: r.CharacterName,
		Race: r.CharacterRace,
	}})
}
