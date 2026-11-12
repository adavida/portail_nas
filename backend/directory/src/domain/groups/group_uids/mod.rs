use crate::domain::ldap::{Gid, Uid};
use crate::domain::users::User;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupUids {
    pub gid: Gid,
    pub members: Vec<Uid>,
}

impl GroupUids {
    pub fn from_attrs(cn: Option<String>, member_dns: &[String]) -> Option<Self> {
        let gid = Gid::try_new(cn?).ok()?;
        Some(Self {
            gid,
            members: member_dns
                .iter()
                .filter_map(|dn| Self::uid_from_dn(dn))
                .collect(),
        })
    }

    pub fn uid_from_dn(dn: &str) -> Option<Uid> {
        let (kind, rest) = dn.split_once('=')?;
        if !kind.eq_ignore_ascii_case("uid") && !kind.eq_ignore_ascii_case("cn") {
            return None;
        }
        Uid::try_new(rest.split(',').next()?.to_string()).ok()
    }

    pub fn group_of(&self, user: &User) -> Option<Gid> {
        self.members.contains(&user.uid).then(|| self.gid.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: &str = "dc=dev,dc=example,dc=com";

    #[test]
    fn member_dns_map_to_uids() {
        let group = GroupUids::from_attrs(
            Some("devs".into()),
            &[
                format!("uid=alice,ou=people,{BASE}"),
                format!("uid=bob,ou=people,{BASE}"),
                format!("cn=carol,ou=people,{BASE}"),
            ],
        )
        .unwrap();

        assert_eq!(group.gid.as_str(), "devs", "gid should decode from cn");
        assert_eq!(
            group.members,
            vec![
                Uid::try_new("alice".into()).unwrap(),
                Uid::try_new("bob".into()).unwrap(),
                Uid::try_new("carol".into()).unwrap(),
            ],
            "uid and cn members should both decode"
        );
    }

    #[test]
    fn uid_from_dn_parses_cn_rdn() {
        let member = GroupUids::uid_from_dn(&format!("cn=alice,ou=people,{BASE}"));

        assert_eq!(
            member,
            Some(Uid::try_new("alice".into()).unwrap()),
            "legacy cn DN should decode to the uid"
        );
    }

    #[test]
    fn uid_from_dn_rejects_unknown_shapes() {
        assert_eq!(
            GroupUids::uid_from_dn("alice-people-dev"),
            None,
            "DN without '=' should yield None"
        );
        assert_eq!(
            GroupUids::uid_from_dn("uid=Alice!,ou=people,dc=dev"),
            None,
            "invalid uid characters should yield None"
        );
    }

    #[test]
    fn rejects_missing_cn() {
        let result = GroupUids::from_attrs(None, &[]);

        assert_eq!(result, None, "no cn should yield None");
    }

    #[test]
    fn rejects_invalid_gid() {
        let result = GroupUids::from_attrs(Some("Devs!".into()), &[]);

        assert_eq!(result, None, "invalid gid should yield None");
    }

    #[test]
    fn group_of_returns_gid_when_member() {
        let alice = User::from_attrs(
            Some("alice".into()),
            None,
            Some("Alice".into()),
            Some(String::new()),
        )
        .unwrap();
        let g = GroupUids::from_attrs(Some("devs".into()), &["uid=alice".into()]).unwrap();

        let groups = g.group_of(&alice);

        assert_eq!(groups, Some(Gid::try_new("devs".into()).unwrap()));
    }

    #[test]
    fn group_of_none_when_not_member() {
        let dave = User::from_attrs(
            Some("dave".into()),
            None,
            Some("Dave".into()),
            Some(String::new()),
        )
        .unwrap();
        let g = GroupUids::from_attrs(Some("devs".into()), &["uid=alice".into()]).unwrap();

        let groups = g.group_of(&dave);

        assert_eq!(groups, None, "uid not a member should yield None");
    }
}
