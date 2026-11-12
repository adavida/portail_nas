use crate::domain::groups::NewGroup;
use crate::domain::ldap::{Description, Gid, Members, Name, Uid};

pub(crate) fn new_group(gid: &Gid, members: &[Uid]) -> NewGroup {
    NewGroup {
        gid: gid.clone(),
        name: Name::try_new("Seed Group".into()).unwrap(),
        description: Description::try_new("seeded group".into()),
        members: Members::try_new(members.to_vec()).unwrap(),
    }
}
