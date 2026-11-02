pub(crate) fn req_id_from(location: &str) -> String {
    location
        .split("req=")
        .nth(1)
        .unwrap_or_default()
        .split('&')
        .next()
        .unwrap_or_default()
        .to_string()
}
