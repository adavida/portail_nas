use crate::{
    domain::{
        groups::{Description, Gid, Members, Name, NewGroup},
        users::Uid,
    },
    repository::ldap::create_group,
};
use portail_core::error::AppError;

pub(crate) async fn seed_group(gid: &Gid, members: &[Uid]) -> bool {
    let new = NewGroup {
        gid: gid.clone(),
        name: Name::try_new("Group Seed".into()).unwrap(),
        description: Description::try_new("".into()),
        members: Members::try_new(members.to_vec()).unwrap(),
    };

    match create_group(new).await {
        Ok(_) => true,
        Err(AppError::Ldap(_)) => false,
        Err(e) => panic!("create_group should succeed, got {e}"),
    }
}
