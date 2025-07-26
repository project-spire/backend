package character

import (
	"context"
	"github.com/geldata/gel-go/geltypes"
	"net/http"
	"spire/lobby/core"

	"github.com/gin-gonic/gin"
)

func HandleList(c *gin.Context, x *core.Context) {
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

	var characters []Character
	if err := x.D.QuerySingle(context.Background(), query, &characters, args); err != nil {
		core.Check(err, c, http.StatusInternalServerError)
		return
	}

	c.JSON(http.StatusOK, Response{Characters: characters})
}
