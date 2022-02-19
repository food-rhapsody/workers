use serde::{Deserialize, Serialize};
use serde_json::json;
use worker::*;

use crate::api_error::ApiError;
use crate::api_result::ApiResult;
use crate::durable::DurableStorageFind;
use crate::req::ParseReqJson;
use crate::res::{response, response_with_cache};
use crate::uid;

const ID_PREFIX: &str = "id_";

pub fn challenge_id_key(id: &str) -> String {
    format!("{}{}", ID_PREFIX, id)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChallengeDto {
    pub name: String,
    pub stamps: Vec<Stamp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateChallengeDto {
    pub id: String,
    pub name: Option<String>,
    pub stamps: Option<Vec<Stamp>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Challenge {
    pub id: String,
    pub name: String,
    pub stamps: Vec<Stamp>,
}

impl Challenge {
    pub fn new(dto: &CreateChallengeDto) -> Self {
        let id = uid!();

        Self {
            id,
            name: dto.name.to_owned(),
            stamps: dto.stamps.clone(),
        }
    }

    pub fn id_key(&self) -> String {
        challenge_id_key(&self.id)
    }

    pub fn update(&mut self, updates: &UpdateChallengeDto) -> &mut Self {
        if let Some(name) = &updates.name {
            self.name = name.to_owned();
        }
        if let Some(stamps) = &updates.stamps {
            self.stamps = stamps.clone();
        }

        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stamp {
    pub id: String,
    pub title: String,
    pub description: String,
    pub img_url: String,
}

#[durable_object]
pub struct Challenges {
    state: State,
    env: Env,
}

impl Challenges {
    pub async fn find_by_id(&self, id: &str) -> ApiResult<Option<Challenge>> {
        self.state
            .storage()
            .find::<Challenge>(&challenge_id_key(id))
            .await
    }

    pub async fn get_by_id(&self, id: &str) -> ApiResult<Challenge> {
        let challenge = self.find_by_id(id).await?;

        match challenge {
            Some(x) => Ok(x),
            None => Err(ApiError::ChallengeNotExists),
        }
    }

    pub async fn list(&self) -> ApiResult<Vec<Challenge>> {
        let storage = self.state.storage();
        let mut challenges = Vec::<Challenge>::new();

        let options = ListOptions::new().prefix(ID_PREFIX);
        let entries = storage.list_with_options(options).await?;

        entries.for_each(&mut |value, _| {
            challenges.push(value.into_serde::<Challenge>().unwrap());
        });

        Ok(challenges.to_owned())
    }

    pub async fn update(&self, challenge: &Challenge) -> ApiResult<()> {
        self.state
            .storage()
            .put(&challenge.id_key(), &challenge)
            .await?;

        Ok(())
    }
}

pub async fn create_challenge(challenges: &Challenges, mut req: Request) -> ApiResult<Challenge> {
    let dto = req.parse_json::<CreateChallengeDto>().await?;
    let challenge = Challenge::new(&dto);

    challenges.update(&challenge).await?;

    Ok(challenge)
}

pub async fn update_challenge(challenges: &Challenges, mut req: Request) -> ApiResult<Challenge> {
    let dto = req.parse_json::<UpdateChallengeDto>().await?;
    let mut challenge = challenges.get_by_id(&dto.id).await?;
    challenge.update(&dto);

    challenges.update(&challenge).await?;

    Ok(challenge)
}

#[durable_object]
impl DurableObject for Challenges {
    fn new(state: State, env: Env) -> Self {
        Self { state, env }
    }

    async fn fetch(&mut self, req: Request) -> Result<Response> {
        let method = req.method();
        let path = req.path();

        // GET /challenges
        if method == Method::Get && &path == "/challenges" {
            return match self.list().await {
                Ok(challenges) => response_with_cache(&json!({ "challenges": challenges }), 60),
                Err(e) => Ok(e.to_response()),
            };
        }

        // POST /challenges
        if method == Method::Post && &path == "/challenges" {
            return match create_challenge(self, req).await {
                Ok(challenge) => response(&challenge),
                Err(e) => Ok(e.to_response()),
            };
        }

        // PUT /challenges
        if method == Method::Put && &path == "/challenges" {
            return match update_challenge(self, req).await {
                Ok(challenge) => response(&challenge),
                Err(e) => Ok(e.to_response()),
            };
        }

        Response::error("not found", 404)
    }
}
