use crate::config::config;
use crate::error::Error;
use data::character::Race;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use jsonwebtoken::EncodingKey;
use protocol::lobby::characters_server::Characters;
use protocol::lobby::{
    CreateCharacterRequest, CreateCharacterResponse, DeleteCharacterRequest,
    DeleteCharacterResponse, ListCharactersResponse, create_character_response,
};
use tonic::{Request, Response, Status};
use util::id::Id;
use util::token::Claims;

pub struct Server {
    pub encoding_key: EncodingKey,
}

impl Server {
    pub fn new() -> Self {
        let encoding_key = EncodingKey::from_secret(&config().token_key);

        Self { encoding_key }
    }
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = data::schema::character)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Character {
    pub id: Id,
    pub name: String,
    pub race: Race,
}

#[derive(Insertable)]
#[diesel(table_name = data::schema::character)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewCharacter {
    pub id: Id,
    pub account_id: Id,
    pub name: String,
    pub race: Race,
}

#[tonic::async_trait]
impl Characters for Server {
    async fn list_characters(
        &self,
        request: Request<()>,
    ) -> Result<Response<ListCharactersResponse>, Status> {
        use data::schema::character::dsl::*;

        let claims = request.extensions().get::<Claims>().unwrap();

        let mut conn = db::conn().await.map_err(Error::DatabaseConnection)?;
        let characters: Vec<Character> = character
            .filter(account_id.eq(claims.account_id))
            .select(Character::as_select())
            .load(&mut conn)
            .await
            .map_err(Error::DatabaseQuery)?;

        let characters: Vec<protocol::CharacterData> =
            characters.into_iter().map(|c| c.into()).collect();
        let response = ListCharactersResponse { characters };

        Ok(Response::new(response))
    }

    async fn create_character(
        &self,
        request: Request<CreateCharacterRequest>,
    ) -> Result<Response<CreateCharacterResponse>, Status> {
        let claims = request.extensions().get::<Claims>().unwrap().clone();
        let request = request.into_inner();
        let race: Race = protocol::Race::try_from(request.race)
            .map_err(|e| Error::UnknownEnumValue(e))?
            .into();

        let character_id = util::id::global();
        let new_character = NewCharacter {
            id: character_id,
            account_id: claims.account_id,
            name: request.name.clone(),
            race,
        };

        {
            use data::schema::character::dsl::*;

            let mut conn = db::conn().await.map_err(Error::DatabaseConnection)?;
            diesel::insert_into(character)
                .values(&new_character)
                .execute(&mut conn)
                .await
                .map_err(Error::DatabaseQuery)?;
        }

        let character = Some(protocol::CharacterData {
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
        request: Request<DeleteCharacterRequest>,
    ) -> Result<Response<DeleteCharacterResponse>, Status> {
        todo!()
    }
}

impl Into<protocol::CharacterData> for Character {
    fn into(self) -> protocol::CharacterData {
        let race: protocol::Race = self.race.into();

        protocol::CharacterData {
            id: self.id,
            name: self.name,
            race: race.into(),
        }
    }
}
