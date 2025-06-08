package character

type Character struct {
	ID   uint64 `json:"id" binding:"required"`
	Name string `json:"name" binding:"required"`
	Race string `json:"race" binding:"required"`
}
