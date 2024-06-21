// Derives inside the nutype macro are not recognized by the compiler so to suppress the warnings,
// allow "unused" imports
#![allow(unused_imports)]

use std::fmt::Display;

use nutype::nutype;
use serde::{Deserialize, Serialize};

#[nutype(
    default = 2,
    validate(less_or_equal = 60),
    derive(
        Clone,
        Debug,
        Default,
        Deserialize,
        Display,
        PartialEq,
        Serialize,
        AsRef,
        Deref
    )
)]
pub struct RefreshInterval(u32);
