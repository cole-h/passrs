//! [structopt]-powered command line interface
//!
//! # cli
//!
//! [structopt]: https://docs.rs/structopt

use std::io::{self, Write};

use anyhow::Result;
use structopt::clap::AppSettings;
use structopt::StructOpt;

use crate::subcmds::{cp, edit, find, generate, git, grep, init, insert, ls, mv, rm, show, unclip};
use crate::util;

// TODO: check out argh: https://github.com/google/argh
#[derive(Debug, StructOpt)]
#[structopt(
    name = "passrs",
    set_term_width(80),
    setting = AppSettings::DeriveDisplayOrder,
    version = env!("PASSRS_VERSION")
)]
struct Pass {
    #[structopt(subcommand)]
    subcmd: Option<PassSubcmd>,
}

#[derive(Debug, StructOpt)]
#[structopt(no_version)]
/// A crabby rewrite of `pass`, the standard unix password manager
pub(crate) enum PassSubcmd {
    /// Initialize a new store or substore.
    Init {
        /// The gpg-id(s) to encrypt the store with. If no keys are specified,
        /// PASSWORD_STORE_KEY will be used.
        gpg_ids: Vec<String>,
        #[structopt(long, short = "p")]
        /// The specified gpg-id(s) is assigned to the specified subfolder.
        path: Option<String>,
    },
    /// List all secrets.
    Ls {
        /// The subfolder to list.
        subfolder: Option<String>,
    },
    /// List secrets that match secret-name.
    Find {
        #[structopt(required = true)]
        /// The secret to match.
        secret_name: String,
    },
    /// Show existing secret.
    Show {
        #[structopt(required = true)]
        /// The secret to show.
        secret_name: String,
        #[structopt(long, short = "c")]
        #[allow(clippy::option_option)]
        /// Optionally, put the secret on the clipboard. If a line number is
        /// specified, that line (1-based) will be copied. Otherwise, the first
        /// line of the file will be copied. If put on the clipboard, the secret
        /// will be cleared in PASSWORD_STORE_CLIP_TIME in seconds, or 45
        /// seconds if unspecified.
        /// NOTE: This flag must be the final argument.
        // Some(Some(usize)) => contents of line at usize.wrapping_sub(1)
        // Some(None) => contents of first line
        // None => don't clip
        clip: Option<Option<usize>>,
    },
    /// Search for pattern in secrets.
    Grep {
        #[structopt(required = true)]
        /// The string to grep for.
        search_string: String,
    },
    /// Insert a new secret.
    Insert {
        #[structopt(required = true)]
        /// The secret to insert into.
        secret_name: String,
        #[structopt(long, short = "e", conflicts_with = "multiline")]
        /// Echo the secret back to the console during entry.
        echo: bool,
        #[structopt(long, short = "f")]
        /// Overwriting existing secret forcefully.
        force: bool,
        #[structopt(long, short = "m", conflicts_with = "echo")]
        /// Enable multiline mode.
        multiline: bool,
    },
    /// Insert a new secret or edit an existing one using $EDITOR.
    Edit {
        #[structopt(required = true)]
        /// The secret to edit.
        secret_name: String,
    },
    /// Generate a new secret.
    Generate {
        #[structopt(required = true)]
        /// The secret to generate for.
        secret_name: String,
        #[structopt(long, short = "c")]
        /// Optionally, put the secret on the clipboard. If put on the
        /// clipboard, the secret will be cleared in PASSWORD_STORE_CLIP_TIME in
        /// seconds, or 45 seconds if unspecified.
        clip: bool,
        #[structopt(long, short = "f", conflicts_with = "in-place")]
        /// Overwriting existing secret forcefully.
        force: bool,
        #[structopt(long, short = "i", conflicts_with = "force")]
        /// Remove only the first line of an existing file with a new secret.
        in_place: bool,
        #[structopt(long, short = "n")]
        /// Disable special symbols.
        no_symbols: bool,
        /// The length of the secret, which defaults to 24 if not specified.
        length: Option<usize>,
    },
    /// Remove existing secret or directory.
    Rm {
        #[structopt(required = true)]
        /// The secret to remove.
        secret_name: String,
        #[structopt(long, short = "f")]
        /// Delete forcefully.
        force: bool,
        #[structopt(long, short = "r")]
        /// Delete recursively.
        recursive: bool,
    },
    /// Move old-path to new-path.
    Mv {
        #[structopt(required = true)]
        /// The path to move from.
        old_path: String,
        #[structopt(required = true)]
        /// The path to move to.
        new_path: String,
        #[structopt(long, short = "f")]
        /// Move forcefully.
        force: bool,
    },
    /// Copy old-path to new-path.
    Cp {
        #[structopt(required = true)]
        /// The path to copy from.
        old_path: String,
        #[structopt(required = true)]
        /// The path to copy to.
        new_path: String,
        #[structopt(long, short = "f")]
        /// Copy forcefully.
        force: bool,
    },
    #[structopt(settings = &[AppSettings::TrailingVarArg, AppSettings::AllowLeadingHyphen])]
    /// Execute a git command inside the password store.
    Git {
        /// Arguments to be passed to the git binary
        git_command_args: Vec<String>,
    },
    #[cfg(feature = "otp")]
    /// Manage TOTP secrets
    Otp(Otp),
    #[structopt(setting = AppSettings::Hidden)]
    /// Clipboard daemon functionality.
    Unclip {
        #[structopt(required = true)]
        /// Amount of time to kill the clipboard after.
        timeout: u64,
        #[structopt(long, short = "f")]
        /// Clear clipboard even if checksum mismatches
        force: bool,
    },
}

/// For managing one-time-password (OTP) tokens with passrs
#[cfg(feature = "otp")]
#[derive(Debug, StructOpt)]
#[structopt(no_version, setting = AppSettings::DeriveDisplayOrder)]
pub(crate) enum Otp {
    /// Generate and print a TOTP code from the key in secret-name.
    Code {
        #[structopt(required = true)]
        /// The secret to generate the code from.
        secret_name: String,
        #[structopt(long, short = "c")]
        /// Optionally, put the generated code on the clipboard. If put on the
        /// clipboard, the code will be cleared in PASSWORD_STORE_CLIP_TIME in
        /// seconds, or 45 seconds if unspecified.
        clip: bool,
    },
    /// Insert TOTP secret to secret-name.
    Insert {
        #[structopt(required = true)]
        /// The secret to insert into.
        secret_name: String,
        #[structopt(long, short = "e")]
        /// Echo the secret back to the console during entry.
        echo: bool,
        #[structopt(long, short = "f")]
        /// Overwriting existing secret forcefully.
        force: bool,
        #[structopt(long, short = "g")]
        /// Generate a TOTP code from the newly-inserted secret.
        generate: bool,
        #[structopt(long, short = "s")]
        /// Create a TOTP URI from the provided secret. Assumes SHA1 algorithm,
        /// 30-second period, and 6 digits.
        from_secret: bool,
        #[structopt(long, requires = "from-secret")]
        /// One of SHA1, SHA256, or SHA512.
        algorithm: Option<String>,
        #[structopt(long, requires = "from-secret")]
        /// How often the TOTP refreshes.
        period: Option<u32>,
        #[structopt(long, requires = "from-secret")]
        /// The length of the generated TOTP code.
        digits: Option<usize>,
    },
    /// Append a TOTP secret to secret-name.
    Append {
        #[structopt(required = true)]
        /// The secret to append to.
        secret_name: String,
        #[structopt(long, short = "e")]
        /// Echo the secret back to the console during entry.
        echo: bool,
        #[structopt(long, short = "s")]
        /// Create a TOTP URI from the provided secret. Assumes SHA1 algorithm,
        /// 30-second period, and 6 digits.
        from_secret: bool,
        #[structopt(long, short = "g")]
        /// Generate a TOTP code from the newly-appended secret.
        generate: bool,
        #[structopt(long, requires = "from-secret")]
        /// One of SHA1, SHA256, or SHA512.
        algorithm: Option<String>,
        #[structopt(long, requires = "from-secret")]
        /// How often the TOTP refreshes.
        period: Option<u32>,
        #[structopt(long, requires = "from-secret")]
        /// The length of the TOTP code.
        digits: Option<usize>,
    },
    /// Print the key URI stored in secret-name.
    Uri {
        #[structopt(required = true)]
        /// The secret that contains a URI to print.
        secret_name: String,
        #[structopt(long, short = "c", conflicts_with = "qrcode")]
        /// Copy the URI to the clipboard.
        clip: bool,
        #[structopt(long, short = "q", conflicts_with = "clip")]
        /// Generate a QR code to stdout.
        qrcode: bool,
    },
    /// Test a URI for validity according to the Key Uri Format.
    Validate {
        #[structopt(required = true)]
        /// The URI to test.
        uri: String,
    },
}

#[derive(Debug, Default, Clone, Copy)]
/// A `struct` holding common boolean flags.
pub(crate) struct Flags {
    pub clip: bool,
    pub echo: bool,
    pub force: bool,
    pub from_secret: bool,
    pub generate: bool,
    pub in_place: bool,
    pub multiline: bool,
    pub no_symbols: bool,
    pub qrcode: bool,
    pub recursive: bool,
}

pub fn opt() -> Result<()> {
    let matches = Pass::from_args();

    // NOTE: committing is handled inside any subcommand that may modify the
    // store
    match matches.subcmd {
        Some(sub) => match sub {
            PassSubcmd::Init { gpg_ids, path } => {
                init::init(gpg_ids, path)?;
            }
            PassSubcmd::Ls { subfolder } => {
                util::verify_store_exists()?;
                ls::ls(subfolder)?;
            }
            PassSubcmd::Find { secret_name } => {
                util::verify_store_exists()?;
                find::find(secret_name)?;
            }
            PassSubcmd::Show { secret_name, clip } => {
                util::verify_store_exists()?;
                show::show(secret_name, clip)?;
            }
            PassSubcmd::Grep { search_string } => {
                util::verify_store_exists()?;
                grep::grep(search_string)?;
            }
            PassSubcmd::Insert {
                secret_name,
                echo,
                force,
                multiline,
            } => {
                let flags = Flags {
                    echo,
                    force,
                    multiline,
                    ..Default::default()
                };

                util::ensure_stdout_is_tty()?;
                util::verify_store_exists()?;
                insert::insert(secret_name, flags)?;
            }
            PassSubcmd::Edit { secret_name } => {
                util::ensure_stdout_is_tty()?;
                util::verify_store_exists()?;
                edit::edit(secret_name)?;
            }
            PassSubcmd::Generate {
                secret_name,
                clip,
                force,
                in_place,
                no_symbols,
                length,
            } => {
                let flags = Flags {
                    clip,
                    force,
                    in_place,
                    no_symbols,
                    ..Default::default()
                };

                util::ensure_stdout_is_tty()?;
                util::verify_store_exists()?;
                generate::generate(secret_name, length, flags)?;
            }
            PassSubcmd::Rm {
                secret_name,
                force,
                recursive,
            } => {
                let flags = Flags {
                    recursive,
                    force,
                    ..Default::default()
                };

                util::ensure_stdout_is_tty()?;
                util::verify_store_exists()?;
                rm::rm(secret_name, flags)?;
            }
            PassSubcmd::Mv {
                old_path,
                new_path,
                force,
            } => {
                util::ensure_stdout_is_tty()?;
                util::verify_store_exists()?;
                mv::mv(old_path, new_path, force)?;
            }
            PassSubcmd::Cp {
                old_path,
                new_path,
                force,
            } => {
                util::ensure_stdout_is_tty()?;
                util::verify_store_exists()?;
                cp::cp(old_path, new_path, force)?;
            }
            PassSubcmd::Git { git_command_args } => {
                util::ensure_stdout_is_tty()?;
                util::verify_store_exists()?;
                git::git(git_command_args)?;
            }
            PassSubcmd::Unclip { timeout, force } => {
                util::verify_store_exists()?;
                unclip::unclip(timeout, force)?;
            }
            #[cfg(feature = "otp")]
            PassSubcmd::Otp(otp) => {
                use crate::subcmds::otp::{append, code, insert, uri, validate};

                match otp {
                    Otp::Code { secret_name, clip } => {
                        util::verify_store_exists()?;
                        code::code(secret_name, clip)?;
                    }
                    Otp::Insert {
                        secret_name,
                        echo,
                        force,
                        from_secret,
                        generate,
                        algorithm,
                        digits,
                        period,
                    } => {
                        let flags = Flags {
                            echo,
                            force,
                            from_secret,
                            generate,
                            ..Default::default()
                        };

                        util::ensure_stdout_is_tty()?;
                        util::verify_store_exists()?;
                        insert::insert(secret_name, algorithm, digits, period, flags)?;
                    }
                    Otp::Append {
                        secret_name,
                        echo,
                        from_secret,
                        generate,
                        algorithm,
                        digits,
                        period,
                    } => {
                        let flags = Flags {
                            echo,
                            from_secret,
                            generate,
                            ..Default::default()
                        };

                        util::ensure_stdout_is_tty()?;
                        util::verify_store_exists()?;
                        append::append(secret_name, algorithm, digits, period, flags)?;
                    }
                    Otp::Uri {
                        secret_name,
                        clip,
                        qrcode,
                    } => {
                        let flags = Flags {
                            clip,
                            qrcode,
                            ..Default::default()
                        };

                        util::verify_store_exists()?;
                        uri::uri(secret_name, flags)?;
                    }
                    Otp::Validate { uri } => {
                        util::verify_store_exists()?;

                        match validate::validate(uri) {
                            Ok(_) => writeln!(io::stdout(), "URI is valid.")?,
                            Err(e) => return Err(e),
                        }
                    }
                }
            }
        },
        // If no command is specified, ls the entire password store, like pass
        // does
        None => match util::verify_store_exists() {
            Ok(_) => ls::ls(None)?,
            Err(_) => {
                Pass::clap().print_help()?;
                std::process::exit(1);
            }
        },
    }

    Ok(())
}
