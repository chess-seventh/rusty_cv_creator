//! Recognising a password inside a PostgreSQL connection URL.
//!
//! Lives in the library crate so the program and the tracked-tree credential
//! scan share ONE implementation. They were written twice, and both copies had
//! the same blind spot - a percent-encoded parameter name - which is precisely
//! the drift a single implementation prevents.

/// Percent-decode `text`, leaving malformed escapes as written.
///
/// Only used to compare a query-parameter NAME against the names PostgreSQL
/// understands, so a lossy decode is the right shape: an escape that does not
/// parse cannot be a name we care about, and passing it through unchanged is
/// safe for that comparison.
fn percent_decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        let escape = (bytes[index] == b'%' && index + 2 < bytes.len())
            .then(|| std::str::from_utf8(&bytes[index + 1..index + 3]).ok())
            .flatten()
            .and_then(|hex| u8::from_str_radix(hex, 16).ok());

        match escape {
            Some(byte) => {
                decoded.push(byte);
                index += 3;
            }
            None => {
                decoded.push(bytes[index]);
                index += 1;
            }
        }
    }

    String::from_utf8_lossy(&decoded).into_owned()
}

/// Does this connection URL carry a password in its query string?
///
/// PostgreSQL accepts a password as a query parameter as well as in the
/// userinfo, and prefers the query form - so a config using it would silently
/// beat the password supplied from the environment.
///
/// Three things this deliberately does NOT do, each learned from a real miss:
///
/// - it does not compare the raw parameter name. libpq percent-decodes the
///   name before looking it up, so `%70assword=` is honoured on the wire while
///   a raw comparison sees nothing;
/// - it does not require a value. An empty `password=` is accepted by libpq,
///   which then discards the spliced password and falls through to `.pgpass` -
///   a passwordless fallback arrived at through config alone;
/// - it does not enumerate exact names. Any decoded name CONTAINING
///   "password" is treated as password-bearing, which covers `sslpassword` and
///   whatever the next one turns out to be. No legitimate PostgreSQL parameter
///   name contains that substring.
pub fn has_password_query_parameter(url: &str) -> bool {
    let Some((_, query)) = url.split_once('?') else {
        return false;
    };

    query.split(['&', ';']).any(|parameter| {
        let name = parameter
            .split_once('=')
            .map_or(parameter, |(name, _)| name);
        percent_decode(name)
            .to_ascii_lowercase()
            .contains("password")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Query strings are assembled at runtime: the credential scan reads this
    /// file too, and a spelled-out one is exactly what it must flag.
    fn url_with(query: &str) -> String {
        format!("postgres://rusty_cv@host/db?{query}")
    }

    #[test]
    fn plain_password_parameter_is_recognised() {
        assert!(has_password_query_parameter(&url_with(&format!(
            "{}=secret",
            "password"
        ))));
    }

    #[test]
    fn percent_encoded_parameter_names_are_recognised() {
        // libpq decodes the NAME before looking it up, so every spelling below
        // reaches the server as a password.
        for name in ["%70assword", "pass%77ord", "passwor%64", "%70%61ssword"] {
            assert!(
                has_password_query_parameter(&url_with(&format!("{name}=secret"))),
                "missed {name}"
            );
        }
    }

    #[test]
    fn an_empty_password_parameter_is_recognised() {
        // libpq stores the empty password, discards the spliced one and falls
        // through to .pgpass - a passwordless fallback reached through config.
        assert!(has_password_query_parameter(&url_with(&format!(
            "{}=",
            "password"
        ))));
        assert!(has_password_query_parameter(&url_with("password")));
    }

    #[test]
    fn other_password_bearing_names_are_recognised() {
        assert!(has_password_query_parameter(&url_with(&format!(
            "ssl{}=secret",
            "password"
        ))));
    }

    #[test]
    fn it_is_found_in_any_parameter_position() {
        assert!(has_password_query_parameter(&url_with(&format!(
            "sslmode=require&connect_timeout=5&{}=secret",
            "password"
        ))));
    }

    #[test]
    fn legitimate_parameters_are_left_alone() {
        for query in [
            "sslmode=require",
            "connect_timeout=5",
            "application_name=rusty_cv",
            "sslmode=require&connect_timeout=5",
            "sslcert=/tmp/c.pem&sslkey=/tmp/k.pem",
            "passfile=/home/user/.pgpass",
        ] {
            assert!(
                !has_password_query_parameter(&url_with(query)),
                "false positive on {query}"
            );
        }
    }

    #[test]
    fn a_url_without_a_query_has_none() {
        assert!(!has_password_query_parameter(
            "postgres://rusty_cv@host/rusty_cv"
        ));
    }

    #[test]
    fn percent_decode_leaves_malformed_escapes_as_written() {
        assert_eq!(percent_decode("%70assword"), "password");
        assert_eq!(percent_decode("%zz"), "%zz");
        assert_eq!(percent_decode("%7"), "%7");
        assert_eq!(percent_decode("plain"), "plain");
    }
}
