use crate::domain::ldap::{Cn, Gid, Uid};
use crate::domain::users::User;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupUids {
    pub gid: Gid,
    pub members: Vec<Members>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Members {
    Uid(Uid),
    Cn(Cn),
}

impl Members {
    pub fn member_uids(member_dns: &[String]) -> Vec<Self> {
        member_dns.iter().filter_map(|dn| Self::from(dn)).collect()
    }

    pub fn from(dn: &str) -> Option<Self> {
        let (kind, rest) = dn.split_once('=')?;
        let first = rest.split(',').next()?;
        match kind.to_ascii_lowercase().as_str() {
            "uid" => Uid::try_new(first.to_string()).map(Self::Uid).ok(),
            "cn" => Cn::try_new(first.to_string()).map(Self::Cn).ok(),
            _ => None,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Members::Uid(uid) => uid.as_str(),
            Members::Cn(cn) => cn.as_str(),
        }
    }

    fn contains_user(&self, user: &User) -> bool {
        self.name() == user.uid.as_str()
    }
}

impl GroupUids {
    pub fn from_attrs(cn: Option<String>, member_dns: &[String]) -> Option<Self> {
        let gid = Gid::try_new(cn?).ok()?;
        Some(Self {
            gid,
            members: Members::member_uids(member_dns),
        })
    }

    pub fn group_of(&self, user: &User) -> Option<Gid> {
        self.members
            .iter()
            .any(|m| m.contains_user(user))
            .then(|| self.gid.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: &str = "dc=dev,dc=example,dc=com";

    #[test]
    fn member_dns_map_to_members() {
        let base = "dc=dev,dc=example,dc=com";
        let members = Members::member_uids(&[
            format!("uid=alice,ou=people,{base}"),
            format!("uid=bob,ou=people,{base}"),
            format!("cn=devs,ou=groups,{base}"),
        ]);

        assert_eq!(members.len(), 3, "uid and cn DNs should both decode");
        assert_eq!(
            members[0],
            Members::Uid(Uid::try_new("alice".into()).unwrap())
        );
        assert_eq!(
            members[1],
            Members::Uid(Uid::try_new("bob".into()).unwrap())
        );
        assert_eq!(members[2], Members::Cn(Cn::try_new("devs".into()).unwrap()));
    }

    #[test]
    fn from_parses_uid_dn() {
        let member = Members::from("uid=alice,ou=people,dc=dev,dc=example,dc=com");

        assert_eq!(
            member,
            Some(Members::Uid(Uid::try_new("alice".into()).unwrap())),
            "uid DN should decode to Members::Uid"
        );
    }

    #[test]
    fn from_parses_cn_dn() {
        let member = Members::from("cn=devs,ou=groups,dc=dev,dc=example,dc=com");

        assert_eq!(
            member,
            Some(Members::Cn(Cn::try_new("devs".into()).unwrap())),
            "cn DN should decode to Members::Cn"
        );
    }

    #[test]
    fn from_rejects_unknown_kind() {
        let member = Members::from("ou=people,dc=dev,dc=example,dc=com");

        assert_eq!(member, None, "non uid/cn kind should yield None");
    }

    #[test]
    fn from_rejects_dn_without_equals() {
        let member = Members::from("alice-people-dev");

        assert_eq!(member, None, "DN without '=' should yield None");
    }

    #[test]
    fn from_rejects_invalid_uid() {
        let member = Members::from("uid=Alice!,ou=people,dc=dev,dc=example,dc=com");

        assert_eq!(member, None, "invalid uid characters should yield None");
    }

    #[test]
    fn name_returns_value_regardless_of_variant() {
        let uid_member = Members::Uid(Uid::try_new("alice".into()).unwrap());
        let cn_member = Members::Cn(Cn::try_new("devs".into()).unwrap());

        assert_eq!(uid_member.name(), "alice");
        assert_eq!(cn_member.name(), "devs");
    }

    #[test]
    fn builds_gid_and_uids() {
        let result = GroupUids::from_attrs(
            Some("devs".into()),
            &[
                format!("uid=alice,ou=people,{BASE}"),
                format!("cn=devs,ou=groups,{BASE}"),
            ],
        )
        .unwrap();

        assert_eq!(result.gid.as_str(), "devs");
        assert_eq!(
            result.members.len(),
            2,
            "self cn placeholder should be kept as Cn member"
        );
        assert_eq!(
            result.members[0],
            Members::Uid(Uid::try_new("alice".into()).unwrap())
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
    fn group_of_matches_cn_member_by_uid() {
        let alice = User::from_attrs(
            Some("alice".into()),
            None,
            Some("Alice".into()),
            Some(String::new()),
        )
        .unwrap();
        let g = GroupUids::from_attrs(Some("devs".into()), &["cn=alice,ou=people".into()]).unwrap();

        let groups = g.group_of(&alice);

        assert_eq!(
            groups,
            Some(Gid::try_new("devs".into()).unwrap()),
            "cn member matching user uid should count as membership"
        );
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
