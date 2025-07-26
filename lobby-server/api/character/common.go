package character

import "github.com/geldata/gel-go/geltypes"

type Character struct {
	Id   geltypes.UUID `json:"id" binding:"required"`
	Name string        `json:"name" binding:"required"`
	Race string        `json:"race" binding:"required"`
}
