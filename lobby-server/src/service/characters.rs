use tonic::{Request, Response, Status};
use protocol::lobby::characters_server::Characters;
use protocol::lobby::{create_character_response, CreateCharacterRequest, CreateCharacterResponse, DeleteCharacterRequest, DeleteCharacterResponse, ListCharactersResponse};
use crate::data::character::Race;
use crate::error::Error;
use crate::context::Context;
use crate::util::token::Claims;

#[derive(Debug, sqlx::FromRow)]
pub struct Character {
    pub id: i64,
    pub name: Option<String>,
    pub race: Option<Race>,
}

#[tonic::async_trait]
impl Characters for Context {
    async fn list_characters(
        &self,
        request: Request<()>
    ) -> Result<Response<ListCharactersResponse>, Status> {
        let claims = request.extensions().get::<Claims>().unwrap();

        let characters = sqlx::query_as!(
            Character,
            r#"select id, name, race as "race: _"
            from character
            where id = $1"#,
            claims.account_id,
        )
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| Error::Database(e))?;

        let characters: Vec<protocol::Character> = characters
            .into_iter()
            .map(|c| c.into())
            .collect();
        let response = ListCharactersResponse { characters };

        Ok(Response::new(response))
    }

    async fn create_character(
        &self,
        request: Request<CreateCharacterRequest>
    ) -> Result<Response<CreateCharacterResponse>, Status> {
        let claims = request.extensions().get::<Claims>().unwrap().clone();
        let request = request.into_inner();
        let race: Race = protocol::Race::try_from(request.race)
            .map_err(|e| Error::UnknownEnumValue(e))?
            .into();

        let character_id = util::id::generate();

        sqlx::query!(
            r#"insert into character (id, account_id, name, race)
            values ($1, $2, $3, $4)"#,
            character_id,
            claims.account_id,
            request.name,
            race as Race,
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| Error::Database(e))?;

        let character = Some(protocol::Character {
            id: character_id,
            name: request.name,
            race: request.race,
        });

        let response = CreateCharacterResponse {
            result: create_character_response::Result::Ok.into(),
            character,
        };

        Ok(Response::new(response))
    }

    async fn delete_character(
        &self,
        request: Request<DeleteCharacterRequest>
    ) -> Result<Response<DeleteCharacterResponse>, Status> {
        todo!()
    }
}

impl Into<protocol::Character> for Character {
    fn into(self) -> protocol::Character {
        let race: protocol::Race = self.race.unwrap().into();

        protocol::Character {
            id: self.id,
            name: self.name.unwrap(),
            race: race.into(),
        }
    }
}
