/// Top-level error type.
#[derive(Debug)]
pub enum Error {
    /// Invalid CLI input (unknown flag, bad value, missing value, etc.).
    User(String),
    /// Arbitrary user error bubbled from callbacks.
    UserAny(Box<dyn std::error::Error + Send + Sync + 'static>),
    /// Internal/structural error (should be rare).
    Parse(String),
    /// Exit with a specific code (e.g., `--help`). Optional message payload.
    ExitMsg {
        code: i32,
        message: Option<String>,
    },
    /// Rich diagnostics
    UnknownOption {
        token: String,
        suggestions: Vec<String>,
    },
    UnknownCommand {
        token: String,
        suggestions: Vec<String>,
    },
    MissingValue {
        opt: String,
    },
    UnexpectedPositional {
        token: String,
    },
}

fn format_alternates(items: &[String]) -> String {
    match items.len() {
        0 => String::new(),
        1 => format!("'{}'", items[0]),
        2 => format!("'{}' or '{}'", items[0], items[1]),
        _ => {
            // 'a', 'b', or 'c'
            let mut s = String::new();
            for (i, it) in items.iter().enumerate() {
                if i > 0 {
                    s.push_str(if i + 1 == items.len() { ", or " } else { ", " });
                }
                s.push('\'');
                s.push_str(it);
                s.push('\'');
            }
            s
        }
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UserAny(e) => write!(f, "{e}"),
            Self::Parse(s) | Self::User(s) => write!(f, "{s}"),
            Self::ExitMsg { code, message } => {
                if let Some(m) = message {
                    write!(f, "{m}")?;
                }
                write!(f, " (exit {code})")
            }
            Self::UnknownOption { token, suggestions } => {
                write!(f, "unknown option: '{token}'")?;
                if !suggestions.is_empty() {
                    write!(f, ". Did you mean {}?", format_alternates(suggestions))?;
                }
                Ok(())
            },
            Self::UnknownCommand { token, suggestions } => {
                write!(f, "unknown command: '{token}'")?;
                if !suggestions.is_empty() {
                    write!(f, ". Did you mean {}?", format_alternates(suggestions))?;
                }
                Ok(())
            }
            Self::MissingValue { opt } => write!(f, "missing value for option: '{opt}'"),
            Self::UnexpectedPositional { token } => write!(f, "unexpected positional: '{token}'"),
        }
    }
}
impl std::error::Error for Error {}

impl From<Box<dyn std::error::Error + Send + Sync + 'static>> for Error {
    fn from(e: Box<dyn std::error::Error + Send + Sync + 'static>) -> Self {
        Self::UserAny(e)
    }
}

impl Error {
    pub fn user<E>(e: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::UserAny(Box::new(e))
    }
}

/// Result alias.
pub type Result<T, E = Error> = core::result::Result<T, E>;
