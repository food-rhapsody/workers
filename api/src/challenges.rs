use serde::{Deserialize, Serialize};
use serde_json::json;
use worker::*;

use crate::api_error::ApiError;
use crate::api_result::ApiResult;
use crate::req::ParseReqJson;
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
    async fn find<T: for<'a> Deserialize<'a>>(&self, key: &str) -> ApiResult<Option<T>> {
        let data = self.state.storage().get::<T>(key).await;

        match data {
            Ok(x) => Ok(Some(x)),
            Err(e) => match e {
                Error::JsError(msg) => match &msg[..] {
                    "No such value in storage." => Ok(None),
                    _ => Err(ApiError::ServerError("storage error".to_string())),
                },
                _ => Err(ApiError::WorkerError { source: e }),
            },
        }
    }

    pub async fn find_challenge_by_id(&self, id: &str) -> ApiResult<Option<Challenge>> {
        self.find::<Challenge>(&challenge_id_key(id)).await
    }

    pub async fn get_challenge_by_id(&self, id: &str) -> ApiResult<Challenge> {
        let challenge = self.find_challenge_by_id(id).await?;

        match challenge {
            Some(x) => Ok(x),
            None => Err(ApiError::ChallengeNotExists),
        }
    }

    pub async fn list_challenges(&self) -> ApiResult<Vec<Challenge>> {
        let storage = self.state.storage();
        let mut challenges = Vec::<Challenge>::new();

        let options = ListOptions::new().prefix(ID_PREFIX);
        let entries = storage.list_with_options(options).await?;

        entries.for_each(&mut |value, _| {
            challenges.push(value.into_serde::<Challenge>().unwrap());
        });

        Ok(challenges.to_owned())
    }

    pub async fn update_challenge(&self, challenge: &Challenge) -> ApiResult<()> {
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

    challenges.update_challenge(&challenge).await?;

    Ok(challenge)
}

pub async fn update_challenge(challenges: &Challenges, mut req: Request) -> ApiResult<Challenge> {
    let dto = req.parse_json::<UpdateChallengeDto>().await?;
    let mut challenge = challenges.get_challenge_by_id(&dto.id).await?;
    challenge.update(&dto);

    challenges.update_challenge(&challenge).await?;

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

        match method {
            Method::Get => match &path[..] {
                "/challenges" => match self.list_challenges().await {
                    Ok(challenges) => {
                        let body = json!({
                            "challenges": challenges,
                        });
                        let res = Response::from_json(&body)?;
                        let mut res_headers = Headers::new();
                        res_headers.append("cache-control", &format!("public, max-age={}", 60))?;
                        res_headers.set("content-type", "application/json; charset=utf-8")?;

                        Ok(res.with_headers(res_headers))
                    }
                    Err(e) => Ok(e.to_response()),
                },
                _ => Response::error("not found", 404),
            },
            Method::Post => match &path[..] {
                "/challenges" => match create_challenge(self, req).await {
                    Ok(challenge) => Response::from_json(&challenge),
                    Err(e) => Ok(e.to_response()),
                },
                _ => Response::error("not found", 404),
            },
            Method::Put => match &path[..] {
                "/challenges" => match update_challenge(self, req).await {
                    Ok(challenge) => Response::from_json(&challenge),
                    Err(e) => Ok(e.to_response()),
                },
                _ => Response::error("not found", 404),
            },
            _ => Response::error("not found", 404),
        }
    }
}
