use crate::Source;
use std::collections::HashMap;
use std::ffi::{OsStr, OsString};

/// Value container stored in `Matches`.
#[derive(Clone, Debug)]
pub enum Value {
    Flag,
    One(OsString),
    Many(Vec<OsString>),
}

/// Whether a key is set and from where.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Unset,
    Set(Source),
}

/// Prefix used to differentiate positional keys.
const POS_PREFIX: &str = "@pos";

/// Join a command path and logical name into the internal key (options/flags).
#[must_use]
pub fn key_for(path: &[&str], name: &str) -> String {
    if path.is_empty() {
        name.to_string()
    } else {
        format!("{}.{}", path.join("."), name)
    }
}

/// Join a command path and positional name into the internal key.
#[must_use]
pub fn pos_key_for(path: &[&str], name: &str) -> String {
    if path.is_empty() {
        format!("{POS_PREFIX}.{name}")
    } else {
        format!("{}.{}.{}", path.join("."), POS_PREFIX, name)
    }
}

fn key_for_strings(path: &[String], name: &str) -> String {
    if path.is_empty() {
        name.to_string()
    } else {
        format!("{}.{}", path.join("."), name)
    }
}

/// Same as `pos_key_for`, but for an owned `Vec<String>` without temporaries.
fn pos_key_for_strings(path: &[String], name: &str) -> String {
    if path.is_empty() {
        format!("{POS_PREFIX}.{name}")
    } else {
        format!("{}.{}.{}", path.join("."), POS_PREFIX, name)
    }
}

/// All parsed values and their sources. Internally keyed by flattened strings.
/// We add `leaf_path` so callers can query *scoped* without spelling keys.
#[derive(Debug)]
pub struct Matches {
    pub(crate) values: HashMap<String, Value>,
    pub(crate) status: HashMap<String, Status>,
    pub(crate) flag_counts: HashMap<String, usize>,
    leaf_path: Vec<String>,
}

impl Matches {
    pub(crate) fn new() -> Self {
        Self { values: HashMap::new(), status: HashMap::new(), flag_counts: HashMap::new(), leaf_path: Vec::new() }
    }

    /// Set the *leaf* (selected) command path. Parser calls this before returning.
    pub(crate) fn set_leaf_path(&mut self, path: &[&str]) {
        self.leaf_path.clear();
        self.leaf_path.extend(path.iter().map(|s| (*s).to_string()));
    }

    /// The leaf command path as `Vec<&str>`.
    #[must_use]
    pub fn leaf_path(&self) -> Vec<&str> {
        self.leaf_path.iter().map(std::string::String::as_str).collect()
    }

    /// Scoped view at the *leaf* command. Most handlers will use this.
    #[must_use]
    pub fn view(&self) -> MatchView<'_> {
        MatchView { m: self, path: self.leaf_path() }
    }

    /// Scoped view at an explicit command path, e.g., `&["remote", "add"]`.
    #[must_use]
    pub fn at<'a>(&'a self, path: &[&'a str]) -> MatchView<'a> {
        MatchView { m: self, path: path.to_vec() }
    }

    /// Get a positional by name in the *leaf* scope as a slice (one becomes a 1‑elem slice).
    /// Returns None if not present.
    #[must_use]
    pub fn get_position(&self, name: &str) -> Option<&[OsString]> {
        let k = pos_key_for_strings(&self.leaf_path, name);
        match self.values.get(&k) {
            Some(Value::Many(vs)) => Some(vs.as_slice()),
            Some(Value::One(v)) => Some(std::slice::from_ref(v)),
            _ => None,
        }
    }

    /// Get a single positional by name in the leaf scope.
    #[must_use]
    pub fn get_position_one(&self, name: &str) -> Option<&OsStr> {
        let k = pos_key_for_strings(&self.leaf_path, name);
        match self.values.get(&k) {
            Some(Value::One(v)) => Some(v.as_os_str()),
            Some(Value::Many(vs)) => vs.first().map(std::ffi::OsString::as_os_str),
            _ => None,
        }
    }

    /// Get an option value in the leaf scope (first of many if repeated).
    #[must_use]
    pub fn get_value(&self, name: &str) -> Option<&OsStr> {
        let k = key_for_strings(&self.leaf_path, name);
        match self.values.get(&k) {
            Some(Value::One(v)) => Some(v.as_os_str()),
            Some(Value::Many(vs)) => vs.first().map(std::ffi::OsString::as_os_str),
            _ => None,
        }
    }

    /// Get all values for an option in the leaf scope.
    #[must_use]
    pub fn get_values(&self, name: &str) -> Option<&[OsString]> {
        let k = key_for_strings(&self.leaf_path, name);
        match self.values.get(&k) {
            Some(Value::Many(vs)) => Some(vs.as_slice()),
            Some(Value::One(v)) => Some(std::slice::from_ref(v)),
            _ => None,
        }
    }

    /// Test whether a flag/option was set in the leaf scope (from any Source).
    #[must_use]
    pub fn is_set(&self, name: &str) -> bool {
        let k = key_for_strings(&self.leaf_path, name);
        self.status.contains_key(&k)
    }

    /// Like `is_set`, but only if set from a specific Source.
    #[must_use]
    pub fn is_set_from(&self, name: &str, src: Source) -> bool {
        let k = key_for_strings(&self.leaf_path, name);
        matches!(self.status.get(&k), Some(Status::Set(s)) if *s == src)
    }

    /// Number of times a `flag` appeared in the leaf scope.
    /// NOTE: only flags are counted; options with values return 0.
    #[must_use]
    pub fn flag_count(&self, name: &str) -> usize {
        let k = key_for_strings(&self.leaf_path, name);
        *self.flag_counts.get(&k).unwrap_or(&0)
    }
}

/// Read‑only scoped accessor into `Matches`.
pub struct MatchView<'a> {
    m: &'a Matches,
    path: Vec<&'a str>,
}

impl MatchView<'_> {
    /// Current command path for this view.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn path(&self) -> &[&str] {
        &self.path
    }

    /// True if a flag/option `name` is present in this scope.
    #[must_use]
    pub fn is_set(&self, name: &str) -> bool {
        let k = key_for(&self.path, name);
        self.m.status.contains_key(&k)
    }

    /// True if present and from the given `Source`.
    #[must_use]
    pub fn is_set_from(&self, name: &str, src: Source) -> bool {
        let k = key_for(&self.path, name);
        matches!(self.m.status.get(&k), Some(Status::Set(s)) if *s == src)
    }

    /// Number of times a `flag` appeared in this scope.
    /// NOTE: only flags are counted; options with values return 0.
    #[must_use]
    pub fn flag_count(&self, name: &str) -> usize {
        let k = key_for(&self.path, name);
        *self.m.flag_counts.get(&k).unwrap_or(&0)
    }

    /// Get a single **option** value (first of many if repeated).
    #[must_use]
    pub fn value(&self, name: &str) -> Option<&OsStr> {
        let k = key_for(&self.path, name);
        match self.m.values.get(&k) {
            Some(Value::One(v)) => Some(v.as_os_str()),
            Some(Value::Many(vs)) => vs.first().map(std::ffi::OsString::as_os_str),
            Some(Value::Flag) | None => None,
        }
    }

    /// Get all **option** values as a slice.
    #[must_use]
    pub fn values(&self, name: &str) -> Option<&[OsString]> {
        let k = key_for(&self.path, name);
        match self.m.values.get(&k) {
            Some(Value::Many(vs)) => Some(vs.as_slice()),
            Some(Value::One(v)) => Some(std::slice::from_ref(v)),
            _ => None,
        }
    }

    /// Get the first **positional** with `name`.
    #[must_use]
    pub fn pos_one(&self, name: &str) -> Option<&OsStr> {
        let k = pos_key_for(&self.path, name);
        match self.m.values.get(&k) {
            Some(Value::One(v)) => Some(v.as_os_str()),
            Some(Value::Many(vs)) => vs.first().map(std::ffi::OsString::as_os_str),
            _ => None,
        }
    }

    /// Get all **positionals** with `name` as a slice (one becomes a 1‑elem slice).
    #[must_use]
    pub fn pos_many(&self, name: &str) -> Option<&[OsString]> {
        let k = pos_key_for(&self.path, name);
        match self.m.values.get(&k) {
            Some(Value::Many(vs)) => Some(vs.as_slice()),
            Some(Value::One(v)) => Some(std::slice::from_ref(v)),
            _ => None,
        }
    }

    /// Typed parse helper (option). Parses the *first* value via `FromStr`.
    #[must_use]
    pub fn parse<T: std::str::FromStr>(&self, name: &str) -> Option<Result<T, T::Err>> {
        self.value(name).map(|s| s.to_string_lossy().parse::<T>())
    }
}
