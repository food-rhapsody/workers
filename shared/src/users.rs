use serde::{Deserialize, Serialize};
use worker::kv::KvStore;
use worker::*;

#[derive(Serialize, Deserialize)]
pub struct User {
    id: String,
    email: String,
    token: String,
}

pub struct UsersService<D> {
    ctx: RouteContext<D>,
}

impl<D> UsersService<D> {
    pub fn new(ctx: RouteContext<D>) -> UsersService<D> {
        UsersService { ctx }
    }

    pub fn get_ky_store(&self) -> Result<KvStore> {
        self.ctx.kv("USERS")
    }

    pub async fn get_user_by_id(&self, id: &str) -> Result<Option<User>> {
        let users = self.get_ky_store()?;
        let key = format!("id_{}", id);
        let value = users.get(&key).await?;

        match value {
            Some(x) => {
                let user = x.as_json::<User>()?;
                Ok(Some(user))
            }
            None => Ok(None),
        }
    }

    pub async fn get_user_by_token(&self, token: &str) -> Result<Option<User>> {
        let users = self.get_ky_store()?;
        let key = format!("token_{}", token);
        let value = users.get(&key).await?;

        match value {
            Some(x) => {
                let user_id = x.as_string();
                let user = self.get_user_by_id(&user_id).await?;

                Ok(user)
            }
            None => Ok(None)
        }
    }
}
