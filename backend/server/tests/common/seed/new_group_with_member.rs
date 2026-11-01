use portail_directory::domain::{
    groups::NewGroup,
    ldap::{Description, Gid, Members, Name, Uid},
};

use super::nanos::nanos;

pub fn new_group_with_member(prefix: &str, member_uid: &str) -> NewGroup {
    NewGroup {
        gid: Gid::try_new(format!("{prefix}{}", nanos())).unwrap(),
        name: Name::try_new("Unit Group".into()).unwrap(),
        description: Description::try_new("unit test group".into()),
        members: Members::try_new(vec![Uid::try_new(member_uid.into()).unwrap()]).unwrap(),
    }
}
