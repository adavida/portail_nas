pub mod errors;
pub use errors::ClientError;

use portail_core::config::ClientEntry;

use crate::domain::oidc::{ClientId, RedirectUri};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Client {
    pub id: ClientId,
    pub secret_file: Option<String>,
    pub enabled: bool,
    pub redirect_uris: Vec<RedirectUri>,
    pub scopes: Vec<String>,
}

impl Client {
    pub fn try_new(
        id: String,
        secret_file: Option<String>,
        enabled: bool,
        redirect_uris: Vec<String>,
        scopes: Vec<String>,
    ) -> Result<Self, ClientError> {
        let id = ClientId::try_new(id).map_err(ClientError::Id)?;
        let secret_file = secret_file
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        if redirect_uris.is_empty() {
            return Err(ClientError::NoRedirects);
        }
        let redirect_uris = redirect_uris
            .into_iter()
            .map(|r| RedirectUri::try_new(r).map_err(ClientError::Redirect))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            id,
            secret_file,
            enabled,
            redirect_uris,
            scopes,
        })
    }

    pub fn from_entries(entries: &[ClientEntry]) -> Result<Vec<Self>, ClientError> {
        entries
            .iter()
            .map(|entry| {
                Self::try_new(
                    entry.id.clone(),
                    entry.secret_file.clone(),
                    entry.enabled,
                    entry.redirect_uris.clone(),
                    entry.scopes.clone(),
                )
            })
            .collect()
    }

    pub fn find_in<'a>(clients: &'a [Client], id: &str) -> Option<&'a Client> {
        clients.iter().find(|c| c.id.as_str() == id)
    }

    pub fn is_public(&self) -> bool {
        self.secret_file.is_none()
    }

    pub fn allows_redirect(&self, uri: &str) -> bool {
        self.redirect_uris.iter().any(|r| r.as_str() == uri)
    }

    pub fn grants(&self, scope: &str) -> bool {
        self.scopes.iter().any(|s| s == scope)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: &str, secret_file: Option<&str>, enabled: bool) -> ClientEntry {
        ClientEntry {
            id: id.into(),
            secret_file: secret_file.map(Into::into),
            enabled,
            redirect_uris: vec!["http://localhost:5173/callback".into()],
            scopes: vec!["openid".into(), "groups".into()],
            custom_claims: Default::default(),
        }
    }

    fn entries() -> Vec<ClientEntry> {
        vec![entry("portail-dev", Some("portail-dev_secret"), true)]
    }

    #[test]
    fn from_entries_parses_all_fields() {
        let clients = Client::from_entries(&entries()).unwrap();

        assert_eq!(clients.len(), 1, "one client expected");
        assert_eq!(clients[0].id.as_str(), "portail-dev");
        assert!(clients[0].enabled, "enabled flag must be parsed");
        assert_eq!(
            clients[0].secret_file.as_deref(),
            Some("portail-dev_secret"),
            "secret file name must be parsed"
        );
        assert!(
            clients[0].grants("groups"),
            "declared scope should be granted"
        );
    }

    #[test]
    fn try_new_rejects_empty_redirects() {
        let err =
            Client::try_new("app1".into(), Some("s".into()), true, vec![], vec![]).unwrap_err();

        assert_eq!(
            err,
            ClientError::NoRedirects,
            "client without redirect_uris should fail"
        );
    }

    #[test]
    fn missing_or_blank_secret_file_marks_public_client() {
        let public = Client::from_entries(&[entry("app1", None, true)])
            .unwrap()
            .remove(0);
        let blank = Client::try_new(
            "app2".into(),
            Some("   ".into()),
            true,
            vec!["https://app/cb".into()],
            vec![],
        )
        .unwrap();

        assert!(
            public.is_public(),
            "absent secret_file must mark a public client"
        );
        assert!(blank.is_public(), "blank secret_file trims to public");
    }

    #[test]
    fn disabled_entry_stays_disabled() {
        let clients = Client::from_entries(&[entry("app1", Some("a"), false)]).unwrap();

        assert!(
            !clients[0].enabled,
            "entry with enabled=false must stay disabled"
        );
    }

    #[test]
    fn from_entries_propagates_invalid_client() {
        let err = Client::from_entries(&[ClientEntry {
            id: "app1".into(),
            secret_file: None,
            enabled: true,
            redirect_uris: vec![],
            scopes: vec![],
            custom_claims: Default::default(),
        }])
        .unwrap_err();

        assert!(
            matches!(err, ClientError::NoRedirects),
            "invalid entry should surface its error, got {err}"
        );
    }

    #[test]
    fn find_in_returns_matching_client() {
        let clients = Client::from_entries(&entries()).unwrap();

        let found = Client::find_in(&clients, "portail-dev");
        let missing = Client::find_in(&clients, "nope");

        assert!(found.is_some(), "declared client should be found");
        assert!(missing.is_none(), "unknown client should be None");
    }

    #[test]
    fn allows_redirect_exact_match() {
        let clients = Client::from_entries(&entries()).unwrap();
        let client = &clients[0];

        assert!(
            client.allows_redirect("http://localhost:5173/callback"),
            "declared redirect should be allowed"
        );
        assert!(
            !client.allows_redirect("http://evil.example.com/cb"),
            "undeclared redirect must be rejected"
        );
    }
}
