use failure::Fallible;
use structopt::{clap::AppSettings, StructOpt};

use crate::subcmds::*;
use crate::util;

#[derive(Debug, StructOpt)]
#[structopt(
    set_term_width(80),
    settings = &[AppSettings::ArgsNegateSubcommands,
                 AppSettings::DeriveDisplayOrder,
                 AppSettings::VersionlessSubcommands],
    version = crate::consts::VERSION.as_str())]
struct Pass {
    #[structopt(subcommand)]
    subcmd: Option<PassSub>,
}

#[derive(Debug, StructOpt)]
#[structopt(no_version)]
enum PassSub {
    /// Initialize new password store and use the provided gpg-id for
    /// encryption.
    Init {
        #[structopt(long, short = "p")]
        /// The specified gpg-id is assigned to the specified subfolder.
        path: Option<String>,
        gpg_id: Option<String>,
    },
    /// List secrets.
    Ls { subfolder: Option<String> },
    /// List secrets that match secret-names.
    Find {
        #[structopt(required = true)]
        secret_names: Vec<String>,
    },
    /// Show existing secret.
    Show {
        #[structopt(required = true)]
        secret_name: String,
        #[structopt(long, short = "c")]
        #[allow(clippy::option_option)]
        /// Optionally, put the secret on the clipboard. If a line number is
        /// specified, that line (1-based for better UX) will be copied.
        /// Otherwise, the first line of the file will be copied.. If put on the
        /// clipboard, the secret will be cleared in PASSWORD_STORE_CLIP_TIME in
        /// seconds, or 45 seconds if unspecified.
        // Some(Some(usize)) => contents of line at usize.wrapping_sub(1)
        // Some(None) => contents of first line
        // None => don't clip
        clip: Option<Option<usize>>,
    },
    /// Search for secret files containing search-string when decrypted.
    /// GREPOPTIONS are explicitly *NOT* supported.
    Grep {
        #[structopt(required = true)]
        search_string: String,
    },
    /// Insert new secret.
    Insert {
        #[structopt(long, short = "e", conflicts_with = "multiline")]
        /// Echo the secret back to the console during entry.
        echo: bool,
        #[structopt(long, short = "m", conflicts_with = "echo")]
        /// Enable multiline mode.
        multiline: bool,
        #[structopt(long, short = "f")]
        /// Overwriting existing secret forcefully.
        force: bool,
        #[structopt(required = true)]
        secret_name: String,
    },
    /// Insert a new secret or edit an existing secret using $EDITOR.
    Edit {
        #[structopt(required = true)]
        secret_name: String,
    },
    /// Generate a new secret of pass-length, or 24 if unspecified.
    Generate {
        #[structopt(long, short = "n")]
        /// Disable special symbols.
        no_symbols: bool,
        #[structopt(long, short = "c")]
        /// Optionally, put the secret on the clipboard. If put on the
        /// clipboard, the secret will be cleared in PASSWORD_STORE_CLIP_TIME in
        /// seconds, or 45 seconds if unspecified.
        clip: bool,
        #[structopt(long, short = "i", conflicts_with = "force")]
        /// Remove only the first line of an existing file with a new secret.
        in_place: bool,
        #[structopt(long, short = "f", conflicts_with = "in_place")]
        /// Overwriting existing secret forcefully.
        force: bool,
        #[structopt(required = true)]
        secret_name: String,
        /// The length of the secret.
        secret_length: Option<usize>,
    },
    /// Remove existing secret or directory.
    Rm {
        #[structopt(long, short = "r")]
        /// Delete recursively.
        recursive: bool,
        #[structopt(long, short = "f")]
        /// Delete forcefully.
        force: bool,
        #[structopt(required = true)]
        secret_name: String,
    },
    /// Rename or move old-path to new-path.
    Mv {
        #[structopt(long, short = "f")]
        /// Move forcefully.
        force: bool,
        old_path: String,
        new_path: String,
    },
    /// Copy old-path to new-path.
    Cp {
        #[structopt(long, short = "f")]
        /// Copy forcefully.
        force: bool,
        old_path: String,
        new_path: String,
    },
    /// Execute a git command specified by git-command-args inside the password
    /// store.
    #[structopt(setting = AppSettings::TrailingVarArg)]
    Git {
        #[structopt(required = true)]
        git_command_args: Vec<String>,
    },
    #[cfg(feature = "otp")]
    /// Manage OTP tokens
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
    #[structopt(setting = AppSettings::Hidden)]
    #[doc(hidden)]
    __Unexhaustive,
}

/// For managing one-time-password (OTP) tokens with {{TODO: better exe name}}
#[cfg(feature = "otp")]
#[derive(Debug, StructOpt)]
#[structopt(no_version)]
enum Otp {
    /// Generate and print an OTP code from the secret key in pass-name.
    Code {
        #[structopt(long, short = "c")]
        /// Optionally, put the generated code on the clipboard. If put on the
        /// clipboard, the code will be cleared in PASSWORD_STORE_CLIP_TIME in
        /// seconds, or 45 seconds if unspecified.
        clip: bool,
        #[structopt(required = true)]
        secret_name: String,
    },
    /// Insert OTP secret to pass-name.
    Insert {
        #[structopt(long, short = "f")]
        /// Overwriting existing secret forcefully.
        force: bool,
        #[structopt(long, short = "e")]
        /// Echo the secret back to the console during entry.
        echo: bool,
        #[structopt(required = true)]
        secret_name: String,
        /// Assumes SHA1 algorithm, 30-second period, and 6 digits.
        secret: Option<String>,
        /// One of SHA1, SHA256, or SHA512.
        algo: Option<String>,
        /// How often the OTP refreshes.
        period: Option<u32>,
        /// The length of the OTP code.
        length: Option<usize>,
    },
    /// Append an OTP secret to pass-name.
    Append {
        #[structopt(long, short = "e")]
        /// Echo the secret back to the console during entry.
        echo: bool,
        #[structopt(required = true)]
        secret_name: String,
        /// Assumes SHA1 algorithm, 30-second period, and 6 digits.
        secret: Option<String>,
        /// One of SHA1, SHA256, or SHA512.
        algo: Option<String>,
        /// How often the OTP refreshes.
        period: Option<u32>,
        /// The length of the OTP code.
        length: Option<usize>,
    },
    /// Print the key URI stored in pass-name.
    Uri {
        #[structopt(long, short = "c", conflicts_with = "qrcode")]
        /// Copy the URI to the clipboard.
        clip: bool,
        #[structopt(long, short = "q", conflicts_with = "clip")]
        /// Generate a QR code located at the specified path.
        qrcode: Option<String>,
        #[structopt(required = true)]
        secret_name: String,
    },
    /// Test a URI string for validity according to the Key Uri Format.
    Validate {
        #[structopt(required = true)]
        uri: String,
    },
}

pub fn opt() -> Fallible<()> {
    let matches = Pass::from_args();
    dbg!(&matches);

    // NOTE: committing is handled in any subcommand that may modify the store
    match matches.subcmd {
        Some(sub) => {
            match sub {
                PassSub::Init { path, gpg_id } => {
                    init::init(path, gpg_id)?;
                }
                PassSub::Ls { subfolder } => {
                    util::verify_store_exists()?;
                    ls::ls(subfolder)?;
                }
                PassSub::Find { secret_names } => {
                    util::verify_store_exists()?;
                    find::find(secret_names)?;
                }
                PassSub::Show { clip, secret_name } => {
                    util::verify_store_exists()?;
                    show::show(clip, secret_name)?;
                }
                PassSub::Grep { search_string } => {
                    util::verify_store_exists()?;
                    grep::grep(search_string)?;
                }
                PassSub::Insert {
                    echo,
                    multiline,
                    force,
                    secret_name,
                } => {
                    #[cfg(not(debug_assertions))]
                    panic!(
                        "Functions that may modify the store are currently disabled for safety reasons."
                    );
                    util::verify_store_exists()?;
                    insert::insert(echo, multiline, force, secret_name)?;
                }
                // TODO: PassSub::Append { echo, multiline, secret_name } => {}
                PassSub::Edit { secret_name } => {
                    #[cfg(not(debug_assertions))]
                    panic!(
                        "Functions that may modify the store are currently disabled for safety reasons."
                    );
                    util::verify_store_exists()?;
                    edit::edit(secret_name)?;
                }
                PassSub::Generate {
                    no_symbols,
                    clip,
                    in_place,
                    force,
                    secret_name,
                    secret_length,
                } => {
                    #[cfg(not(debug_assertions))]
                    panic!(
                        "Functions that may modify the store are currently disabled for safety reasons."
                    );
                    util::verify_store_exists()?;
                    generate::generate(
                        no_symbols,
                        clip,
                        in_place,
                        force,
                        secret_name,
                        secret_length,
                    )?;
                }
                PassSub::Rm {
                    recursive,
                    force,
                    secret_name,
                } => {
                    #[cfg(not(debug_assertions))]
                    panic!(
                        "Functions that may modify the store are currently disabled for safety reasons."
                    );
                    util::verify_store_exists()?;
                    rm::rm(recursive, force, secret_name)?;
                }
                PassSub::Mv {
                    force,
                    old_path,
                    new_path,
                } => {
                    #[cfg(not(debug_assertions))]
                    panic!(
                        "Functions that may modify the store are currently disabled for safety reasons."
                    );
                    util::verify_store_exists()?;
                    mv::mv(force, old_path, new_path)?;
                }
                PassSub::Cp {
                    force,
                    old_path,
                    new_path,
                } => {
                    #[cfg(not(debug_assertions))]
                    panic!(
                        "Functions that may modify the store are currently disabled for safety reasons."
                    );
                    util::verify_store_exists()?;
                    cp::cp(force, old_path, new_path)?;
                }
                PassSub::Git { git_command_args } => {
                    util::verify_store_exists()?;
                    git::git(git_command_args)?;
                }
                PassSub::Unclip { timeout, force } => {
                    util::verify_store_exists()?;
                    unclip::unclip(timeout, force)?;
                }
                #[cfg(feature = "otp")]
                PassSub::Otp(otp) => {
                    use crate::subcmds::otp::*;
                    match otp {
                        Otp::Code { clip, secret_name } => {
                            util::verify_store_exists()?;
                            code::code(clip, secret_name)?;
                        }
                        Otp::Insert {
                            force,
                            echo,
                            secret_name,
                            secret,
                            algo,
                            period,
                            length,
                        } => {
                            #[cfg(not(debug_assertions))]
                            panic!(
                                "Functions that may modify the store are currently disabled for safety reasons."
                            );
                            let _ = (algo, period, length);
                            util::verify_store_exists()?;
                            insert::insert(force, echo, secret_name, secret)?;
                        }
                        Otp::Append {
                            echo,
                            secret_name,
                            secret,
                            algo,
                            period,
                            length,
                        } => {
                            #[cfg(not(debug_assertions))]
                            panic!(
                                "Functions that may modify the store are currently disabled for safety reasons."
                            );
                            let _ = (algo, period, length);
                            util::verify_store_exists()?;
                            append::append(echo, secret_name, secret)?;
                        }
                        Otp::Uri {
                            clip,
                            qrcode,
                            secret_name,
                        } => {
                            util::verify_store_exists()?;
                            uri::uri(clip, qrcode, secret_name)?;
                        }
                        Otp::Validate { uri } => {
                            util::verify_store_exists()?;
                            validate::validate(uri)?;
                        }
                    }
                }
                _ => {}
            }
        }
        // If no command is specified, `ls` the entire password store, like
        // `pass` does
        None => {
            util::verify_store_exists()?;
            ls::ls(None)?;
        }
    }

    Ok(())
}
