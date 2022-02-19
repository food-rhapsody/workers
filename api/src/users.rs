use chrono::Duration;
use serde::{Deserialize, Serialize};
use serde_json::json;
use worker::*;

use crate::api_error::ApiError;
use crate::api_result::ApiResult;
use crate::auth::{authorize_access_token, authorize_refresh_token};
use crate::durable::DurableStorageFind;
use crate::jwt::Jwt;
use crate::oauth::OAuthProvider;
use crate::req::ParseReqJson;
use crate::res::response;
use crate::uid;

const ADMIN_EMAILS: [&str; 1] = ["seokju.me@kakao.com"];

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfoDto {
    pub id: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTokenDto {
    pub id: String,
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

    pub fn is_admin(&self) -> bool {
        ADMIN_EMAILS.iter().any(|x| x == &self.email.as_str())
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

    pub fn to_info_dto(&self) -> UserInfoDto {
        UserInfoDto {
            id: self.id.to_owned(),
            email: self.email.to_owned(),
        }
    }

    pub fn to_token_dto(&self) -> UserTokenDto {
        UserTokenDto {
            id: self.id.to_owned(),
            access_token: self.access_token.clone(),
            refresh_token: self.refresh_token.clone(),
        }
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
    pub fn get_jwt_for_access_token(&self) -> ApiResult<Jwt> {
        let jwt_secret = self.env.secret("JWT_SECRET")?;

        Ok(Jwt::new(&jwt_secret.to_string()))
    }

    pub fn get_jwt_for_refresh_token(&self) -> ApiResult<Jwt> {
        let jwt_secret = self.env.secret("JWT_SECRET_2")?;

        Ok(Jwt::new(&jwt_secret.to_string()))
    }

    pub async fn find_by_id(&self, user_id: &str) -> ApiResult<Option<User>> {
        self.state
            .storage()
            .find::<User>(&user_id_key(user_id))
            .await
    }

    pub async fn find_by_email(&self, email: &str) -> ApiResult<Option<User>> {
        let user_id = self
            .state
            .storage()
            .find::<String>(&user_email_key(email))
            .await?;

        match user_id {
            Some(x) => self.find_by_id(&x).await,
            None => Ok(None),
        }
    }

    pub async fn find_by_refresh_id(&self, refresh_id: &str) -> ApiResult<Option<User>> {
        let user_id = self.state.storage().find::<String>(&refresh_id).await?;

        match user_id {
            Some(x) => self.find_by_id(&x).await,
            None => Ok(None),
        }
    }

    pub async fn get_by_id(&self, user_id: &str) -> ApiResult<User> {
        let user = self.find_by_id(user_id).await?;

        match user {
            Some(x) => Ok(x),
            None => Err(ApiError::UserNotExists),
        }
    }

    pub async fn get_by_email(&self, email: &str) -> ApiResult<User> {
        let user = self.find_by_email(email).await?;

        match user {
            Some(x) => Ok(x),
            None => Err(ApiError::UserNotExists),
        }
    }

    pub async fn get_user_by_refresh_id(&self, refresh_id: &str) -> ApiResult<User> {
        let user = self.find_by_refresh_id(refresh_id).await?;

        match user {
            Some(x) => Ok(x),
            None => Err(ApiError::UserNotExists),
        }
    }

    pub async fn create(&self, mut user: User) -> ApiResult<User> {
        let (refresh_id, refresh_token) = self.create_refresh_token()?;
        let access_token = self.create_user_access_token(&user.id)?;

        user.with_refresh_token(&refresh_token)
            .with_access_token(&access_token);

        let mut s = self.state.storage();

        s.put(&refresh_id, &user.id).await?;
        s.put(&user.id_key(), &user).await?;
        s.put(&user.email_key(), &user.id).await?;

        Ok(user)
    }

    pub async fn update_refresh_token(&self, mut user: User) -> ApiResult<User> {
        let (refresh_id, refresh_token) = self.create_refresh_token()?;

        user.with_refresh_token(&refresh_token);
        self.state.storage().put(&refresh_id, &user.id).await?;
        self.state.storage().put(&user.id_key(), &user).await?;

        Ok(user)
    }

    pub async fn update_access_token(&self, mut user: User) -> ApiResult<User> {
        let access_token = self.create_user_access_token(&user.id)?;

        user.with_access_token(&access_token);
        self.state.storage().put(&user.id_key(), &user).await?;

        Ok(user)
    }

    fn create_refresh_token(&self) -> ApiResult<(String, String)> {
        let jwt = self.get_jwt_for_refresh_token()?;
        let refresh_id = uid!();

        let user_claims = UserClaims::for_refresh_token(&refresh_id);
        let claims = jwt.create_claims(user_claims, Duration::weeks(4));
        let refresh_token = jwt.sign(&claims)?;

        Ok((refresh_id, refresh_token))
    }

    fn create_user_access_token(&self, user_id: &str) -> ApiResult<String> {
        let jwt = self.get_jwt_for_access_token()?;

        let user_claims = UserClaims::for_access_token(&user_id);
        let claims = jwt.create_claims(user_claims, Duration::hours(3));
        let access_token = jwt.sign(&claims)?;

        Ok(access_token)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserDto {
    pub email: String,
    pub name: Option<String>,
    pub oauth_token: String,
    pub oauth_provider: String,
}

pub async fn create_or_update_user(users: &Users, mut req: Request) -> ApiResult<User> {
    let dto = req.parse_json::<CreateUserDto>().await?;

    let provider = OAuthProvider::from_str(&dto.oauth_provider)?;
    provider.verify_token(&dto.oauth_token, &dto.email).await?;

    let exists_user = users.find_by_email(&dto.email).await?;
    if let Some(user) = exists_user {
        let user = users.update_refresh_token(user).await?;
        let user = users.update_access_token(user).await?;

        return Ok(user);
    }

    let user = users.create(User::new(&dto)).await?;

    Ok(user)
}

pub async fn recognize_me(users: &Users, req: Request) -> ApiResult<User> {
    let user = authorize_access_token(&users, req).await?;

    Ok(user)
}

pub async fn update_my_token(users: &Users, req: Request) -> ApiResult<User> {
    let user = authorize_refresh_token(&users, req).await?;

    // TODO(@seokju-na): Renew only when refresh_token expires less than 1 month
    let user = users.update_refresh_token(user).await?;
    let user = users.update_access_token(user).await?;

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

        // GET /me
        if method == Method::Get && &path == "/me" {
            return match recognize_me(self, req).await {
                Ok(user) => response(&json!(user.to_info_dto())),
                Err(e) => Ok(e.to_response()),
            };
        }

        // GET /me/admin
        if method == Method::Get && &path == "/me/admin" {
            return match recognize_me(self, req).await {
                Ok(user) => match user.is_admin() {
                    true => response(&json!(user.to_info_dto())),
                    false => Ok(ApiError::Unauthorized.to_response()),
                },
                Err(e) => Ok(e.to_response()),
            };
        }

        // POST /users
        if method == Method::Post && &path == "/users" {
            return match create_or_update_user(self, req).await {
                Ok(user) => response(&json!(user.to_token_dto())),
                Err(e) => Ok(e.to_response()),
            };
        }

        // POST /me/token
        if method == Method::Post && &path == "/me/token" {
            return match update_my_token(self, req).await {
                Ok(user) => response(&json!(user.to_token_dto())),
                Err(e) => Ok(e.to_response()),
            };
        }

        Response::error("not found", 404)
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
