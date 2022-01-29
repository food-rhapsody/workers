use worker::*;
use worker::kv::*;
use crate::users::User;

pub struct UsersStore<D> {
    ctx: RouteContext<D>,
}

impl<D> UsersStore<D> {
    pub fn new(ctx: RouteContext<D>) -> Self {
        UsersStore { ctx }
    }

    pub fn id_key(id: &str) -> String {
        format!("id_{}", id)
    }

    pub fn token_key(token: &str) -> String {
        format!("token_{}", token)
    }

    pub fn email_key(email: &str) -> String {
        format!("email_{}", email)
    }

    fn get_ky_store(&self) -> Result<KvStore> {
        self.ctx.kv("USERS")
    }

    pub async fn get_user_by_id(&self, id: &str) -> Result<Option<User>> {
        let users = self.get_ky_store()?;
        let key = UsersStore::<()>::id_key(id);
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
        let token_key = UsersStore::<()>::token_key(token);
        let id_key = users.get(&token_key).await?;

        match id_key {
            Some(x) => {
                let user_id = x.as_string();
                let user = self.get_user_by_id(&user_id).await?;

                Ok(user)
            }
            None => Ok(None),
        }
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        let users = self.get_ky_store()?;
        let email_key = UsersStore::<()>::email_key(email);
        let id_key = users.get(&email_key).await?;

        match id_key {
            Some(x) => {
                let user_id = x.as_string();
                let user = self.get_user_by_id(&user_id).await?;

                Ok(user)
            }
            None => Ok(None),
        }
    }

    pub fn save_user(&self, user: &User) -> Result<()> {
        let users = self.get_ky_store()?;
        let id_key = UsersStore::<()>::id_key(&user.id);
        let email_key = UsersStore::<()>::email_key(&user.email);

        users.put(&id_key, &user);
        users.put(&email_key, &id_key);

        Ok(())
    }

    pub async fn update_user_token(&self, user_id: &str, token: &str) -> Result<()> {
        if let None = self.get_user_by_id(user_id).await? {
            return Err(Error::from("User not exists"));
        }

        let users = self.get_ky_store()?;
        let token_key = UsersStore::<()>::token_key(token);
        users.put(&token_key, user_id);

        Ok(())
    }
}
