use serde::{Deserialize, Serialize};
use serde_json::json;
use worker::*;

use crate::api_error::ApiError;
use crate::api_result::ApiResult;
use crate::oauth::OAuthProvider;
use crate::req::ParseReqJson;
use crate::uid;

pub fn user_id_key(id: &str) -> String {
    format!("id_{}", id)
}

pub fn user_email_key(email: &str) -> String {
    format!("email_{}", email)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub oauth_provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserData {
    pub email: String,
    pub name: Option<String>,
    pub oauth_provider: String,
}

impl User {
    pub fn new(data: &CreateUserData) -> User {
        let id = uid!();

        User {
            id,
            email: String::from(&data.email),
            name: data.name.clone(),
            oauth_provider: data.oauth_provider.clone(),
        }
    }

    pub fn id_key(&self) -> String {
        user_id_key(&self.id)
    }

    pub fn email_key(&self) -> String {
        user_email_key(&self.email)
    }
}

#[derive(Serialize, Deserialize)]
struct CreateUserDto {
    pub email: String,
    pub name: Option<String>,
    pub oauth_token: String,
    pub oauth_provider: String,
}

impl CreateUserDto {
    fn to_data(&self) -> CreateUserData {
        CreateUserData {
            email: self.email.clone(),
            name: self.name.clone(),
            oauth_provider: self.oauth_provider.clone(),
        }
    }
}

pub async fn create_user(users: &Users, mut req: Request) -> ApiResult<User> {
    let dto = req.parse_req_json::<CreateUserDto>().await?;

    let provider = OAuthProvider::from_str(&dto.oauth_provider)?;
    provider.verify_token(&dto.oauth_token).await?;

    let user = User::new(&dto.to_data());
    users.save_user(&user).await?;

    Ok(user)
}

#[durable_object]
pub struct Users {
    state: State,
    env: Env,
}

impl Users {
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

    pub async fn find_user_by_id(&self, user_id: &str) -> ApiResult<Option<User>> {
        self.find::<User>(&user_id_key(user_id)).await
    }

    pub async fn find_user_by_email(&self, email: &str) -> ApiResult<Option<User>> {
        let user_id = self.find::<String>(&user_email_key(email)).await?;

        match user_id {
            Some(x) => self.find_user_by_id(&x).await,
            None => Ok(None),
        }
    }

    pub async fn get_user_by_id(&self, user_id: &str) -> ApiResult<User> {
        let user = self.find_user_by_id(user_id).await?;

        match user {
            Some(x) => Ok(x),
            None => Err(ApiError::UserNotExists),
        }
    }

    pub async fn get_user_by_email(&self, email: &str) -> ApiResult<User> {
        let user = self.find_user_by_email(email).await?;

        match user {
            Some(x) => Ok(x),
            None => Err(ApiError::UserNotExists),
        }
    }

    pub async fn save_user(&self, user: &User) -> ApiResult<()> {
        let email_duplicated = self.find_user_by_email(&user.email).await?;
        if let Some(_) = email_duplicated {
            return Err(ApiError::UserEmailDuplicated);
        }

        self.state.storage().put(&user.id_key(), &user).await?;
        self.state
            .storage()
            .put(&user.email_key(), &user.id)
            .await?;

        Ok(())
    }
}

#[durable_object]
impl DurableObject for Users {
    fn new(state: State, env: Env) -> Self {
        Self { state, env }
    }

    async fn fetch(&mut self, req: Request) -> worker::Result<Response> {
        let method = req.method();
        let path = req.path();

        match method {
            Method::Post => match &path[..] {
                "/users" => match create_user(self, req).await {
                    Ok(user) => {
                        let body = json!({
                            "id": user.id,
                        });

                        Response::from_json(&body)
                    }
                    Err(error) => Ok(error.to_response()),
                },
                _ => Response::error("not found", 404),
            },
            _ => Response::error("not found", 404),
        }
    }
}

#[cfg(test)]
mod user_tests {
    use super::*;

    #[test]
    fn should_create_user() {
        let data = CreateUserData {
            email: "seokju.me@gmail.com".to_string(),
            name: Some("Seokju Na".to_string()),
            oauth_provider: "kakao".to_string(),
        };
        let user = User::new(&data);

        assert_eq!(user.id.len(), 21);
        assert_eq!(user.email, "seokju.me@gmail.com");
        assert_eq!(user.name.unwrap(), "Seokju Na");
        assert_eq!(user.oauth_provider, "kakao");
    }

    #[test]
    fn should_create_user_with_none_name() {
        let data = CreateUserData {
            email: "test@test.com".to_string(),
            name: None,
            oauth_provider: "kakao".to_string(),
        };
        let user = User::new(&data);

        assert_eq!(user.id.len(), 21);
        assert_eq!(user.email, "test@test.com");
        assert_eq!(user.name.unwrap_or(String::from("NO_NAMED")), "NO_NAMED");
        assert_eq!(user.oauth_provider, "kakao");
    }
}
