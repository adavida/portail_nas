use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct CustomClaim {
    pub name: String,
    pub value: String,
    #[serde(default)]
    pub group: String,
    #[serde(default)]
    pub not_group: String,
}

impl CustomClaim {
    pub fn applies_to(&self, groups: &[String]) -> bool {
        if !self.group.is_empty() {
            return groups.contains(&self.group);
        }
        if !self.not_group.is_empty() {
            return !groups.contains(&self.not_group);
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::{ClientEntry, CustomClaim};

    fn claim(group: &str, not_group: &str) -> CustomClaim {
        CustomClaim {
            name: "role".into(),
            value: "tester".into(),
            group: group.into(),
            not_group: not_group.into(),
        }
    }

    #[test]
    fn applies_to_unconditional_when_no_gate() {
        let rule = claim("", "");

        let applies = rule.applies_to(&["devs".to_string()]);

        assert!(applies, "no gate must always apply");
    }

    #[test]
    fn applies_to_group_member_only() {
        let rule = claim("admins", "");

        let member = rule.applies_to(&["admins".to_string()]);
        let outsider = rule.applies_to(&["devs".to_string()]);

        assert!(member, "member of the group must match");
        assert!(!outsider, "non-member must not match");
    }

    #[test]
    fn applies_to_non_member_of_not_group() {
        let rule = claim("", "guests");

        let outsider = rule.applies_to(&["devs".to_string()]);
        let member = rule.applies_to(&["guests".to_string()]);

        assert!(outsider, "user outside the group must match");
        assert!(!member, "member of the excluded group must not match");
    }

    fn entry(required_group: &str, forbidden_group: &str) -> ClientEntry {
        ClientEntry {
            id: "app".into(),
            secret_file: None,
            enabled: true,
            redirect_uris: vec!["http://localhost/cb".into()],
            scopes: vec!["openid".into()],
            custom_claims: vec![],
            required_group: required_group.into(),
            forbidden_group: forbidden_group.into(),
        }
    }

    #[test]
    fn gate_denial_allows_without_gates() {
        let client = entry("", "");

        let denial = client.gate_denial(&[]);

        assert_eq!(denial, None, "no gate must allow anyone");
    }

    #[test]
    fn gate_denial_requires_membership() {
        let client = entry("vpn", "");

        let outsider = client.gate_denial(&["devs".to_string()]);
        let member = client.gate_denial(&["vpn".to_string()]);

        assert!(
            outsider.is_some(),
            "non-member must be denied, got {outsider:?}"
        );
        assert_eq!(member, None, "member must be allowed");
    }

    #[test]
    fn gate_denial_blocks_membership() {
        let client = entry("", "guests");

        let guest = client.gate_denial(&["guests".to_string()]);
        let staff = client.gate_denial(&["staff".to_string()]);

        assert!(guest.is_some(), "member of blocked group denied");
        assert_eq!(staff, None, "non-member must be allowed");
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClientEntry {
    pub id: String,
    #[serde(default)]
    pub secret_file: Option<String>,
    #[serde(default)]
    pub enabled: bool,
    pub redirect_uris: Vec<String>,
    #[serde(default)]
    pub scopes: Vec<String>,
    #[serde(default)]
    pub custom_claims: Vec<CustomClaim>,
    #[serde(default)]
    pub required_group: String,
    #[serde(default)]
    pub forbidden_group: String,
}

impl ClientEntry {
    pub fn gate_denial(&self, groups: &[String]) -> Option<String> {
        if !self.required_group.is_empty() && !groups.contains(&self.required_group) {
            return Some(format!(
                "user is not a member of required group '{}'",
                self.required_group
            ));
        }
        if !self.forbidden_group.is_empty() && groups.contains(&self.forbidden_group) {
            return Some(format!(
                "user is a member of forbidden group '{}'",
                self.forbidden_group
            ));
        }
        None
    }
}
