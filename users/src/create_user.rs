use worker::*;
use shared::users::*;

pub async fn create_user(ctx: RouteContext<()>, dto: &CreateUserDto) -> Result<User> {
    let users_store = UsersStore::new(ctx);
    if let Some(_) = users_store.get_user_by_email(&dto.email).await? {
        return Err(Error::from("Email duplicated"));
    }

    let user = User::new(&dto);
    users_store.save_user(&user)?;

    Ok(user)
}
