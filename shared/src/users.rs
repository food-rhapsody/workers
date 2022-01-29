use serde::{Deserialize, Serialize};

use crate::uid;

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct CreateUserDto {
    pub email: String,
    pub name: Option<String>,
}

impl User {
    pub fn new(dto: &CreateUserDto) -> User {
        let id = uid!();

        User {
            id,
            email: String::from(&dto.email),
            name: dto.name.clone(),
        }
    }
}

#[cfg(test)]
mod user_struct_tests {
    use super::*;

    #[test]
    fn should_create_user() {
        let dto = CreateUserDto {
            email: "seokju.me@gmail.com".to_string(),
            name: Some("Seokju Na".to_string()),
        };
        let user = User::new(&dto);

        assert_eq!(user.id.len(), 21);
        assert_eq!(user.email, "seokju.me@gmail.com");
        assert_eq!(user.name.unwrap(), "Seokju Na");
    }

    #[test]
    fn should_create_user_with_none_name() {
        let dto = CreateUserDto {
            email: "test@test.com".to_string(),
            name: None,
        };
        let user = User::new(&dto);

        assert_eq!(user.id.len(), 21);
        assert_eq!(user.email, "test@test.com");
        assert_eq!(user.name.unwrap_or(String::from("NO_NAMED")), "NO_NAMED");
    }
}
