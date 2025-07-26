package character

import (
	"context"
	"net/http"

	"github.com/geldata/gel-go/geltypes"
	"github.com/gin-gonic/gin"
	"spire/lobby/core"
)

func HandleList(c *gin.Context, x *core.Context) {
	type Character struct {
		Id   string `json:"id" binding:"required"`
		Name string `json:"name" binding:"required"`
		Race string `json:"race" binding:"required"`
	}

	type Response struct {
		Characters []Character `json:"characters"`
	}

	accountId := c.MustGet("account_id").(geltypes.UUID)

	query := `
		SELECT Character { 
			id,
			name,
			race
		}
		FILTER .account.id = <uuid>$account_id
		ORDER BY .created DESC;`
	args := map[string]interface{}{
		"account_id": accountId,
	}

	var rows []struct {
		Id   geltypes.UUID `gel:"id"`
		Name string        `gel:"name"`
		Race string        `gel:"race"`
	}
	if err := x.D.Query(context.Background(), query, &rows, args); err != nil {
		core.Check(err, c, http.StatusInternalServerError)
		return
	}

	characters := make([]Character, 0)
	for _, row := range rows {
		characters = append(characters, Character{
			Id:   row.Id.String(),
			Name: row.Name,
			Race: row.Race,
		})
	}

	c.JSON(http.StatusOK, Response{Characters: characters})
}
