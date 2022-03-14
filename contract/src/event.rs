use near_sdk::serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Event(pub u64);

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!(
            "EVENT_JSON: {}",
            &serde_json::to_string(self).map_err(|_| fmt::Error)?
        ))
    }
}
