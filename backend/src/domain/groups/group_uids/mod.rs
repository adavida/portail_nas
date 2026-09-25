use super::Gid;
use crate::domain::users::{Uid, User};

/// Association `gid` -> membres, telle que rapportée par LDAP (cn + DNs member).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupUids {
    pub gid: Gid,
    pub members: Vec<Members>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Members {
    Uid(Uid),
    Cn(Gid),
}

impl Members {
    fn contains_user(&self, user: &User) -> bool {
        match self {
            Members::Uid(uid) => uid == &user.uid,
            Members::Cn(gid) => gid.as_str() == user.uid.as_str(),
        }
    }
}

impl GroupUids {
    /// `None` when the entry has no valid `cn` (invalid gid).
    pub fn from_attrs(cn: Option<String>, member_dns: &[String]) -> Option<Self> {
        let gid = Gid::try_new(cn?).ok()?;
        Some(Self {
            gid,
            members: member_uids(member_dns),
        })
    }

    pub fn group_of(&self, user: &User) -> Option<Gid> {
        self.members
            .iter()
            .any(|m| m.contains_user(user))
            .then(|| self.gid.clone())
    }
}

/// member DNs -> `Members` (`uid=...` -> `Uid`, `cn=...` -> `Cn`),
/// anything the `Uid`/`Gid` VOs reject is dropped.
pub fn member_uids(member_dns: &[String]) -> Vec<Members> {
    member_dns
        .iter()
        .filter_map(|dn| {
            let (kind, rest) = dn.split_once('=')?;
            let first = rest.split(',').next()?;
            match kind.to_ascii_lowercase().as_str() {
                "uid" => Uid::try_new(first.to_string()).map(Members::Uid).ok(),
                "cn" => Gid::try_new(first.to_string()).map(Members::Cn).ok(),
                _ => None,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: &str = "dc=dev,dc=example,dc=com";

    #[test]
    fn member_dns_map_to_members() {
        let base = "dc=dev,dc=example,dc=com";
        let members = member_uids(&[
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
        assert_eq!(
            members[2],
            Members::Cn(Gid::try_new("devs".into()).unwrap())
        );
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
