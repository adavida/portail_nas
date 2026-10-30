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
    use super::CustomClaim;

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
}
