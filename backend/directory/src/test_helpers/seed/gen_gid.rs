use crate::domain::groups::Gid;

use super::gen_uid::gen_uid;

pub(crate) fn gen_gid(prefix: &str) -> Gid {
    Gid::try_new(gen_uid(prefix).as_str().to_string()).unwrap()
}
