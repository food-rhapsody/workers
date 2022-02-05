use chrono::Duration;
use serde::{Deserialize, Serialize};
use serde_json::json;
use worker::*;

use crate::api_error::ApiError;
use crate::api_result::ApiResult;
use crate::auth::{authorize_access_token, authorize_refresh_token};
use crate::jwt::Jwt;
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
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
}

impl User {
    pub fn new(dto: &CreateUserDto) -> Self {
        let id = uid!();

        User {
            id,
            email: String::from(&dto.email),
            name: dto.name.clone(),
            oauth_provider: dto.oauth_provider.clone(),
            access_token: None,
            refresh_token: None,
        }
    }

    pub fn id_key(&self) -> String {
        user_id_key(&self.id)
    }

    pub fn email_key(&self) -> String {
        user_email_key(&self.email)
    }

    pub fn with_refresh_token(&mut self, refresh_token: &str) -> &mut Self {
        self.refresh_token = Some(refresh_token.to_owned());

        self
    }

    pub fn with_access_token(&mut self, access_token: &str) -> &mut Self {
        self.access_token = Some(access_token.to_owned());

        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserClaims {
    #[serde(rename = "sub")]
    pub subject: String,
}

impl UserClaims {
    pub fn for_access_token(user_id: &str) -> Self {
        Self {
            subject: user_id.to_owned(),
        }
    }

    pub fn for_refresh_token(refresh_id: &str) -> Self {
        Self {
            subject: refresh_id.to_owned(),
        }
    }
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

    pub fn get_jwt_for_access_token(&self) -> ApiResult<Jwt> {
        let jwt_secret = self.env.secret("JWT_SECRET")?;

        Ok(Jwt::new(&jwt_secret.to_string()))
    }

    pub fn get_jwt_for_refresh_token(&self) -> ApiResult<Jwt> {
        let jwt_secret = self.env.secret("JWT_SECRET_2")?;

        Ok(Jwt::new(&jwt_secret.to_string()))
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

    pub async fn find_user_by_refresh_id(&self, refresh_id: &str) -> ApiResult<Option<User>> {
        let user_id = self.find::<String>(&refresh_id).await?;

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

    pub async fn get_user_by_refresh_id(&self, refresh_id: &str) -> ApiResult<User> {
        let user = self.find_user_by_refresh_id(refresh_id).await?;

        match user {
            Some(x) => Ok(x),
            None => Err(ApiError::UserNotExists),
        }
    }

    pub async fn create_new_user(&self, mut user: User) -> ApiResult<User> {
        let email_duplicated = self.find_user_by_email(&user.email).await?;
        if let Some(_) = email_duplicated {
            return Err(ApiError::UserEmailDuplicated);
        }

        let (refresh_id, refresh_token) = self.create_new_user_refresh_token()?;
        let access_token = self.create_new_user_access_token(&user.id)?;

        user.with_refresh_token(&refresh_token)
            .with_access_token(&access_token);

        let mut s = self.state.storage();

        s.put(&refresh_id, &user.id).await?;
        s.put(&user.id_key(), &user).await?;
        s.put(&user.email_key(), &user.id).await?;

        Ok(user)
    }

    pub fn create_new_user_refresh_token(&self) -> ApiResult<(String, String)> {
        let jwt = self.get_jwt_for_refresh_token()?;
        let refresh_id = uid!();

        let user_claims = UserClaims::for_refresh_token(&refresh_id);
        let claims = jwt.create_claims(user_claims, Duration::weeks(4));
        let refresh_token = jwt.sign(&claims)?;

        Ok((refresh_id, refresh_token))
    }

    pub fn create_new_user_access_token(&self, user_id: &str) -> ApiResult<String> {
        let jwt = self.get_jwt_for_access_token()?;

        let user_claims = UserClaims::for_access_token(&user_id);
        let claims = jwt.create_claims(user_claims, Duration::hours(3));
        let access_token = jwt.sign(&claims)?;

        Ok(access_token)
    }

    pub async fn update_user_refresh_token(&self, mut user: User) -> ApiResult<User> {
        let (refresh_id, refresh_token) = self.create_new_user_refresh_token()?;

        user.with_refresh_token(&refresh_token);
        self.state.storage().put(&refresh_id, &user.id).await?;
        self.state.storage().put(&user.id_key(), &user).await?;

        Ok(user)
    }

    pub async fn update_user_access_token(&self, mut user: User) -> ApiResult<User> {
        let access_token = self.create_new_user_access_token(&user.id)?;

        user.with_access_token(&access_token);
        self.state.storage().put(&user.id_key(), &user).await?;

        Ok(user)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserDto {
    pub email: String,
    pub name: Option<String>,
    pub oauth_token: String,
    pub oauth_provider: String,
}

pub async fn create_user(users: &Users, mut req: Request) -> ApiResult<User> {
    let dto = req.parse_json::<CreateUserDto>().await?;

    let provider = OAuthProvider::from_str(&dto.oauth_provider)?;
    provider.verify_token(&dto.oauth_token).await?;

    let user = users.create_new_user(User::new(&dto)).await?;

    Ok(user)
}

pub async fn recognize_me(users: &Users, req: Request) -> ApiResult<User> {
    let user = authorize_access_token(&users, req).await?;

    Ok(user)
}

pub async fn update_my_token(users: &Users, req: Request) -> ApiResult<User> {
    let user = authorize_refresh_token(&users, req).await?;

    // TODO(@seokju-na): Renew only when refresh_token expires less than 1 month
    let user = users.update_user_refresh_token(user).await?;
    let user = users.update_user_access_token(user).await?;

    Ok(user)
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
            Method::Get => match &path[..] {
                "/me" => match recognize_me(self, req).await {
                    Ok(user) => {
                        let body = json!({
                            "id": user.id,
                            "email": user.email
                        });

                        Response::from_json(&body)
                    }
                    Err(error) => Ok(error.to_response()),
                },
                _ => Response::error("not found", 404),
            },
            Method::Post => match &path[..] {
                "/users" => match create_user(self, req).await {
                    Ok(user) => {
                        let body = json!({
                            "id": user.id,
                            "refreshToken": user.refresh_token,
                            "accessToken": user.access_token,
                        });

                        Response::from_json(&body)
                    }
                    Err(error) => Ok(error.to_response()),
                },
                "/me/token" => match update_my_token(self, req).await {
                    Ok(user) => {
                        let body = json!({
                            "id": user.id,
                            "refreshToken": user.refresh_token,
                            "accessToken": user.access_token,
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
        let data = CreateUserDto {
            email: "seokju.me@gmail.com".to_string(),
            name: Some("Seokju Na".to_string()),
            oauth_token: "token".to_string(),
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
        let data = CreateUserDto {
            email: "test@test.com".to_string(),
            name: None,
            oauth_token: "token".to_string(),
            oauth_provider: "kakao".to_string(),
        };
        let user = User::new(&data);

        assert_eq!(user.id.len(), 21);
        assert_eq!(user.email, "test@test.com");
        assert_eq!(user.name.unwrap_or(String::from("NO_NAMED")), "NO_NAMED");
        assert_eq!(user.oauth_provider, "kakao");
    }
}
