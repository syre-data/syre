//! `clap` implementations for [`UserId`].
use crate::types::user_id::{ParseError, UserId};

impl clap::FromArgMatches for UserId {
    /// Creates a UserId from clap::ArgMatches.
    /// Argument name must be `user`.
    fn from_arg_matches(matches: &clap::ArgMatches) -> Result<Self, clap::error::Error> {
        let arg_name = "user";
        let mut cmd = clap::Command::new("UserId");
        let id = arg_from_arg_matches(matches, arg_name, &mut cmd)?;
        match UserId::from_string(id) {
            Ok(uid) => Ok(uid),
            Err(ParseError(msg)) => Err(cmd.error(clap::error::ErrorKind::InvalidValue, msg)),
        }
    }

    ///
    /// # Panics
    /// Should panic if type of user id changes.
    fn update_from_arg_matches(
        &mut self,
        matches: &clap::ArgMatches,
    ) -> Result<(), clap::error::Error> {
        let arg_name = "user";
        let mut cmd = clap::Command::new("UserId");
        let id = arg_from_arg_matches(matches, arg_name, &mut cmd)?;
        let id = match UserId::from_string(id) {
            Ok(uid) => uid,
            Err(ParseError(msg)) => {
                return Err(cmd.error(clap::error::ErrorKind::InvalidValue, msg))
            }
        };

        match (self, id) {
            (UserId::Email(ref mut s_em), UserId::Email(n_em)) => {
                *s_em = n_em;
            }
            (UserId::Id(ref mut s_id), UserId::Id(n_id)) => {
                *s_id = n_id;
            }
            _ => {
                return Err(cmd.error(
                    clap::error::ErrorKind::ArgumentConflict,
                    "Incompatibale Id types",
                ))
            }
        };

        Ok(())
    }
}

// @todo [1]: Write tests and implement properly
impl clap::Args for UserId {
    fn augment_args(cmd: clap::Command) -> clap::Command {
        cmd.arg(clap::Arg::new("user"))
    }

    fn augment_args_for_update(cmd: clap::Command) -> clap::Command {
        cmd.arg(clap::Arg::new("user"))
    }
}

/// Extracts the `arg_name` arg from a clap::ArgMatches.
fn arg_from_arg_matches(
    matches: &clap::ArgMatches,
    arg_name: &str,
    cmd: &mut clap::Command,
) -> Result<String, clap::error::Error> {
    let mut arg: Option<String> = None;
    if matches.contains_id(arg_name) {
        match matches.get_one::<String>(arg_name) {
            Some(id_res) => {
                arg = Some(id_res.to_string());
            }
            None => {
                return Err(cmd.error(
                    clap::error::ErrorKind::InvalidValue,
                    "Id found, but could not be retrieved.",
                ))
            }
        }
    }

    if arg.is_none() {
        return Err(cmd.error(
            clap::error::ErrorKind::MissingRequiredArgument,
            "No valid user id found.",
        ));
    }

    Ok(arg.unwrap())
}

#[cfg(test)]
#[path = "./user_id_test.rs"]
mod user_id_test;
