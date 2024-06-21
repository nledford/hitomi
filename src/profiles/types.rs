// Derives inside the nutype macro are not recognized by the compiler so to suppress the warnings,
// allow "unused" imports
#![allow(unused_imports)]

use std::fmt::Display;

use nutype::nutype;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

static PROFILE_SOURCE_ID_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\d+$").unwrap());

#[nutype(
    derive(
        Clone,
        Default,
        Debug,
        Deserialize,
        Display,
        PartialEq,
        Serialize,
        AsRef,
        Deref
    ),
    default = "New Profile",
    validate(not_empty, len_char_max = 25)
)]
pub struct ProfileTitle(String);

#[nutype(
    derive(Clone, Default, Debug, Deserialize, PartialEq, Serialize, AsRef, Deref),
    default = "0",
    validate(not_empty, regex = PROFILE_SOURCE_ID_REGEX)
)]
pub struct ProfileSourceId(String);

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
