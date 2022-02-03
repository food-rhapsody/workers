use serde::{Deserialize, Serialize};
use serde_json::json;
use worker::*;

use crate::api_result::ApiResult;

const ID_PREFIX: &str = "id_";

pub fn challenge_id_key(id: &str) -> String {
    format!("{}{}", ID_PREFIX, id)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Challenge {
    pub id: String,
    pub name: String,
}

#[durable_object]
pub struct Challenges {
    state: State,
    env: Env,
}

impl Challenges {
    pub async fn list_challenges(&self) -> ApiResult<Vec::<Challenge>> {
        let storage = self.state.storage();
        let mut challenges = Vec::<Challenge>::new();

        let options = ListOptions::new().prefix(ID_PREFIX);
        let entries = storage.list_with_options(options).await?;

        entries.for_each(&mut |value, _| {
            challenges.push(value.into_serde::<Challenge>().unwrap());
        });

        Ok(challenges.to_owned())
    }
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

                        Response::from_json(&body)
                    },
                    Err(e) => Ok(e.to_response())
                },
                _ => Response::error("not found", 404),
            },
            _ => Response::error("not found", 404),
        }
    }
}
