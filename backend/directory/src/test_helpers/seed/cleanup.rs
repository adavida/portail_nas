use crate::{
    domain::{groups::Gid, users::Uid},
    repository::ldap::delete_group,
};

use super::ensure_clean::ensure_clean;

pub(crate) async fn cleanup(gid: &Gid, uids: &[Uid]) {
    let _ = delete_group(gid.clone()).await;
    for uid in uids {
        ensure_clean(uid).await;
    }
}
