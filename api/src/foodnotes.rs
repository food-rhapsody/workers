use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use worker::*;

use crate::{ApiError, uid};
use crate::api_result::ApiResult;
use crate::durable::DurableStorageFind;
use crate::place::PlaceDocument;
use crate::req::ParseReqJson;
use crate::res::response;

const ID_PREFIX: &str = "id_";
const AUTHOR_ID_PREFIX: &str = "author_";

pub fn foodnote_id_key(id: &str) -> String {
    format!("{}{}", ID_PREFIX, id)
}

pub fn foodnote_author_id_key(author_id: &str) -> String {
    format!("{}{}", AUTHOR_ID_PREFIX, author_id)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Foodnote {
    pub id: String,
    pub stamp_id: String,
    pub author_id: String,
    pub text: String,
    pub place: PlaceDocument,
    pub timestamp: i64,
    pub img_urls: Vec<String>,
    pub is_public: bool,
}

impl Foodnote {
    pub fn new(dto: CreateFoodnoteDto) -> Self {
        let id = uid!();
        let timestamp = Utc::now().timestamp();

        Self {
            id,
            stamp_id: dto.stamp_id,
            author_id: dto.author_id,
            text: dto.text,
            place: dto.place,
            timestamp,
            img_urls: dto.img_urls,
            is_public: dto.is_public,
        }
    }

    pub fn id_key(&self) -> String {
        foodnote_id_key(&self.id)
    }
}

#[durable_object]
pub struct Foodnotes {
    state: State,
    env: Env,
}

impl Foodnotes {
    pub async fn find_by_id(&self, id: &str) -> ApiResult<Option<Foodnote>> {
        self.state
            .storage()
            .find::<Foodnote>(&foodnote_id_key(id))
            .await
    }

    pub async fn get_by_id(&self, id: &str) -> ApiResult<Foodnote> {
        let foodnote = self.find_by_id(id).await?;

        match foodnote {
            Some(x) => Ok(x),
            None => Err(ApiError::FoodnoteNotExists),
        }
    }

    pub async fn list_ids_for_author(&self, author_id: &str) -> ApiResult<Vec<String>> {
        let ids = self
            .state
            .storage()
            .find::<Vec<String>>(&foodnote_author_id_key(author_id))
            .await?
            .unwrap_or(Vec::new());

        Ok(ids)
    }

    pub async fn list_for_author(&self, author_id: &str) -> ApiResult<Vec<Foodnote>> {
        let storage = self.state.storage();
        let ids = self
            .list_ids_for_author(&author_id)
            .await?
            .into_iter()
            .map(|id| foodnote_id_key(&id))
            .collect();

        let mut foodnotes = Vec::<Foodnote>::new();
        storage.get_multiple(ids).await?.for_each(&mut |value, _| {
            let val = value.into_serde::<Foodnote>().unwrap();
            foodnotes.push(val);
        });

        Ok(foodnotes)
    }

    pub async fn create(&self, foodnote: Foodnote) -> ApiResult<Foodnote> {
        self.state
            .storage()
            .put(&foodnote.id_key(), &foodnote)
            .await?;
        self.append_as_author(&foodnote).await?;

        Ok(foodnote)
    }

    async fn append_as_author(&self, foodnote: &Foodnote) -> ApiResult<()> {
        let id = &foodnote.id;
        let author_id = &foodnote.author_id;

        let mut ids = self.list_ids_for_author(author_id).await?;
        ids.push(id.to_owned());
        console_log!("ids: {:?}", &ids);

        self.state
            .storage()
            .put(&foodnote_author_id_key(author_id), &ids)
            .await?;

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFoodnoteDto {
    pub stamp_id: String,
    pub author_id: String,
    pub text: String,
    pub place: PlaceDocument,
    pub img_urls: Vec<String>,
    pub is_public: bool,
}

pub async fn list_my_foodnotes(foodnotes: &Foodnotes, req: Request) -> ApiResult<Vec<Foodnote>> {
    let author_id = req.headers().get("X-Foodrhapsody-User")?.unwrap();

    foodnotes.list_for_author(&author_id).await
}

pub async fn add_my_foodnote(foodnotes: &Foodnotes, mut req: Request) -> ApiResult<Foodnote> {
    let dto = req.parse_json::<CreateFoodnoteDto>().await?;
    let foodnote = Foodnote::new(dto);

    foodnotes.create(foodnote).await
}

#[durable_object]
impl DurableObject for Foodnotes {
    fn new(state: State, env: Env) -> Self {
        Self { state, env }
    }

    async fn fetch(&mut self, req: Request) -> Result<Response> {
        let method = req.method();
        let path = req.path();

        if method == Method::Get && &path == "/foodnotes" {
            return match list_my_foodnotes(self, req).await {
                Ok(foodnotes) => response(&json!({ "foodnotes": foodnotes })),
                Err(e) => Ok(e.to_response()),
            };
        }

        if method == Method::Post && &path == "/foodnotes" {
            return match add_my_foodnote(self, req).await {
                Ok(foodnote) => response(&json!(foodnote)),
                Err(e) => Ok(e.to_response()),
            };
        }

        Response::error("not found", 404)
    }
}
