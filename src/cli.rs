//! [structopt]-powered command line interface
//!
//! # cli
//!
//! [structopt]: https://docs.rs/structopt

use std::io::{self, Write};

use clap::{AppSettings, Clap, IntoApp};

use crate::subcmds::{cp, edit, find, generate, git, grep, init, insert, ls, mv, rm, show, unclip};
use crate::util;
use crate::Result;

#[derive(Clap, Debug)]
#[clap(
    name = "passrs",
    set_term_width(80),
    version = env!("PASSRS_VERSION")
)]
struct Pass {
    #[clap(subcommand)]
    subcmd: Option<PassSubcmd>,
}

#[derive(Clap, Debug)]
/// A crabby rewrite of `pass`, the standard unix password manager.
pub(crate) enum PassSubcmd {
    /// Initialize a new store or substore.
    Init {
        /// The gpg-id(s) to encrypt the store with (default:
        /// $PASSWORD_STORE_KEY).
        gpg_ids: Vec<String>,
        #[clap(long, short = "p")]
        /// The path of the substore to initialize.
        /// The specified gpg-id(s) is assigned to the specified subfolder.
        path: Option<String>,
    },
    /// List all secrets.
    Ls {
        /// The subfolder to list.
        subfolder: Option<String>,
    },
    /// List all secrets that match secret-name.
    Find {
        /// The name of the secret to find.
        secret_name: String,
    },
    /// Show existing secret.
    Show {
        /// The secret to show.
        secret_name: String,
        #[clap(long, short = "c", next_line_help = true)]
        #[allow(clippy::option_option)]
        /// Copy the secret to the clipboard. If a line number is specified,
        /// that line (1-based) will be copied. Otherwise, the first line of the
        /// file will be copied. The secret will be cleared in
        /// $PASSWORD_STORE_CLIP_TIME seconds (default: 45).
        /// NOTE: This flag must be the final argument.
        // Some(Some(usize)) => contents of line at usize.saturating_sub(1)
        // Some(None) => contents of first line
        // None => don't clip
        clip: Option<Option<usize>>,
    },
    /// Search for pattern in secrets.
    Grep {
        /// The pattern to grep for.
        search_string: String,
    },
    /// Insert a new secret.
    Insert {
        /// The name of the secret to insert.
        /// The secret to insert into.
        secret_name: String,
        #[clap(long, short = "e", conflicts_with = "multiline")]
        /// Echo the secret back to the console during entry.
        echo: bool,
        #[clap(long, short = "f")]
        /// Overwrite existing secret.
        force: bool,
        #[clap(long, short = "m", conflicts_with = "echo")]
        /// Enable multiline mode.
        multiline: bool,
    },
    /// Edit a secret using $EDITOR.
    Edit {
        /// The name of the secret to edit.
        secret_name: String,
    },
    /// Generate a new secret.
    Generate {
        /// The name of the secret to generate.
        secret_name: String,
        /// The length of the secret in characters (default: 24).
        length: Option<usize>,
        #[clap(long, short = "c", next_line_help = true)]
        /// Copy the secret to the clipboard. The secret will be cleared in
        /// $PASSWORD_STORE_CLIP_TIME seconds (default: 45).
        clip: bool,
        #[clap(long, short = "f", conflicts_with = "in-place")]
        /// Overwrite existing secret forcefully.
        force: bool,
        #[clap(long, short = "i", conflicts_with = "force")]
        /// Replace the first line of an existing file.
        in_place: bool,
        #[clap(long, short = "n")]
        /// Disable special symbols.
        no_symbols: bool,
    },
    /// Remove existing secret or directory.
    Rm {
        /// The name of the secret to remove.
        secret_name: String,
        #[clap(long, short = "f")]
        /// Remove forcefully.
        force: bool,
        #[clap(long, short = "r")]
        /// Remove recursively.
        recursive: bool,
    },
    /// Move old-path to new-path.
    Mv {
        /// The path to move from.
        old_path: String,
        /// The path to move to.
        new_path: String,
        #[clap(long, short = "f")]
        /// Force moving.
        force: bool,
    },
    /// Copy old-path to new-path.
    Cp {
        /// The path to copy from.
        old_path: String,
        /// The path to copy to.
        new_path: String,
        #[clap(long, short = "f")]
        /// Force copying.
        force: bool,
    },
    #[clap(setting = AppSettings::TrailingVarArg, setting = AppSettings::AllowLeadingHyphen)]
    /// Execute a git command inside the password store.
    Git {
        /// Arguments to pass to the git binary.
        git_command_args: Vec<String>,
    },
    #[cfg(feature = "otp")]
    /// Manage TOTP secrets.
    Otp(Otp),
    #[clap(setting = AppSettings::Hidden)]
    /// Clipboard daemon functionality.
    Unclip {
        /// How long until the clipboard gets cleared, in seconds.
        timeout: u32,
        #[clap(long, short = "f")]
        /// Clear clipboard even if the checksum doesn't match.
        force: bool,
    },
}

/// For managing one-time-password (OTP) tokens with passrs.
#[cfg(feature = "otp")]
#[derive(Clap, Debug)]
#[clap(setting = AppSettings::DeriveDisplayOrder)]
pub(crate) enum Otp {
    /// Generate a TOTP code from the key in secret-name.
    Code {
        /// The secret to generate the code from.
        secret_name: String,
        #[clap(long, short = "c")]
        /// Copy the secret to the clipboard. The secret will be cleared in
        /// $PASSWORD_STORE_CLIP_TIME seconds (default: 45).
        clip: bool,
    },
    /// Insert a TOTP secret to secret-name.
    Insert {
        /// The name of the secret to insert into.
        secret_name: String,
        #[clap(long, short = "e")]
        /// Echo the secret back to the console during entry.
        echo: bool,
        #[clap(long, short = "f")]
        /// Overwriting existing secret forcefully.
        force: bool,
        #[clap(long, short = "g")]
        /// Generate a TOTP code from the newly-inserted secret.
        generate: bool,
        #[clap(long, short = "s")]
        /// Create a TOTP URI from the provided secret (assumes SHA1 algorithm,
        /// 30 second period, and 6 digits).
        from_secret: bool,
        #[clap(long, short = "a", requires = "from-secret")]
        /// One of SHA1, SHA256, or SHA512.
        algorithm: Option<String>,
        #[clap(long, short = "p", requires = "from-secret")]
        /// How often the TOTP refreshes in seconds.
        period: Option<u32>,
        #[clap(long, short = "d", requires = "from-secret")]
        /// The length of the generated TOTP code in characters.
        digits: Option<usize>,
    },
    /// Append a TOTP secret to secret-name.
    Append {
        /// The name of the secret to append to.
        secret_name: String,
        #[clap(long, short = "e")]
        /// Echo the secret back to the console during entry.
        echo: bool,
        #[clap(long, short = "s")]
        /// Create a TOTP URI from the provided secret (assumes SHA1 algorithm,
        /// 30 second period, and 6 digits).
        from_secret: bool,
        #[clap(long, short = "g")]
        /// Generate a TOTP code from the newly-appended secret.
        generate: bool,
        #[clap(long, short = "a", requires = "from-secret")]
        /// One of SHA1, SHA256, or SHA512.
        algorithm: Option<String>,
        #[clap(long, short = "p", requires = "from-secret")]
        /// How often the TOTP refreshes in seconds.
        period: Option<u32>,
        #[clap(long, short = "d", requires = "from-secret")]
        /// The length of the TOTP code in characters.
        digits: Option<usize>,
    },
    /// Print the key URI stored in secret-name.
    Uri {
        /// The name of the secret that contains the URI to print.
        secret_name: String,
        #[clap(long, short = "c", conflicts_with = "qrcode")]
        /// Copy the URI to the clipboard. The URI will be cleared in
        /// $PASSWORD_STORE_CLIP_TIME seconds (default: 45).
        clip: bool,
        #[clap(long, short = "q", conflicts_with = "clip")]
        /// Generate a QR code to stdout.
        qrcode: bool,
    },
    /// Test a URI for validity according to the Key Uri Format.
    Validate {
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
    let matches = Pass::parse();

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
                Pass::into_app()
                    .print_help()
                    .map_err(|e| format!("Failed to display help: {:?}", e))?;

                std::process::exit(1);
            }
        },
    }

    Ok(())
}
