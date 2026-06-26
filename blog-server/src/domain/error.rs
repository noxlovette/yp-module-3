use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Пользователь не найден")]
    UserNotFound,
    #[error("Пользователь с таким username уже есть")]
    UsernameAlreadyExists,
    #[error("Пользователь с таким email уже есть")]
    EmailAlreadyExists,
    #[error("Неверная пара логин/пароль")]
    InvalidCredentials,
    #[error("Пост не найден")]
    PostNotFound,
    #[error("Кышь отсюда")]
    Forbidden,
}
