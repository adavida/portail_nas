use crate::domain::users::{Email, Name, NewUser, Password, Uid};

pub(crate) fn new_user(uid: &Uid, name: &str, email: &str, password: &str) -> NewUser {
    NewUser {
        uid: uid.clone(),
        name: Name::try_new(name.into()).unwrap(),
        email: Email::try_new(email.into()).unwrap(),
        password: Password::try_new(password.into()).unwrap(),
    }
}
