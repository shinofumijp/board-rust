use rocket::request::FromForm;
use validator::ValidationError;
use validator_derive::Validate;

#[derive(FromForm, Validate)]
pub struct SignInForm {
    #[validate(email, length(min = 1))]
    pub email: String,
    #[validate(length(min = 8), custom = "validate_password_ascii")]
    pub password: String,
}

fn validate_password_ascii(password: &str) -> Result<(), ValidationError> {
    if password.is_ascii() {
        Ok(())
    } else {
        Err(ValidationError::new("non_ascii"))
    }
}
