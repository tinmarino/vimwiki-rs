use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Divider;

impl_located_borrowed_owned!(Divider, Divider, |_| Divider, |_| Divider);
