use std::time::{SystemTime, UNIX_EPOCH};

use crate::domain::users::Uid;

pub(crate) fn gen_uid(prefix: &str) -> Uid {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        % 1000000;
    Uid::try_new(format!("{prefix}{suffix}")).unwrap()
}
