use anyhow::Result;
use structopt::clap;
use structopt::clap::AppSettings;
use structopt::StructOpt;

use crate::subcmds::*;
use crate::util;

#[derive(Debug, StructOpt)]
#[structopt(
    set_term_width(80),
    settings = &[AppSettings::ArgsNegateSubcommands,
                 AppSettings::DeriveDisplayOrder,
                 AppSettings::VersionlessSubcommands],
    global_setting = AppSettings::ColoredHelp,
    version = &*crate::consts::VERSION.as_str())]
struct Pass {
    #[structopt(subcommand)]
    subcmd: Option<PassSubcmd>,
}

#[derive(Debug, StructOpt)]
#[structopt(no_version)]
enum PassSubcmd {
    /// Initialize new password store and use the provided gpg-id for
    /// encryption.
    Init {
        /// The gpg-id(s) to encrypt the store with.
        gpg_ids: Vec<String>,
        #[structopt(long, short = "p")]
        /// The specified gpg-id(s) is assigned to the specified subfolder.
        path: Option<String>,
    },
    /// List secrets.
    Ls { subfolder: Option<String> },
    /// List secrets that match secret-name.
    Find {
        #[structopt(required = true)]
        secret_name: String,
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
        #[structopt(required = true)]
        secret_name: String,
        #[structopt(long, short = "e", conflicts_with = "multiline")]
        /// Echo the secret back to the console during entry.
        echo: bool,
        #[structopt(long, short = "m", conflicts_with = "echo")]
        /// Enable multiline mode.
        multiline: bool,
        #[structopt(long, short = "f")]
        /// Overwriting existing secret forcefully.
        force: bool,
    },
    /// Insert a new secret or edit an existing secret using $EDITOR.
    Edit {
        #[structopt(required = true)]
        secret_name: String,
    },
    /// Generate a new secret of pass-length, or 24 if unspecified.
    Generate {
        #[structopt(required = true)]
        secret_name: String,
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
        #[structopt(long, short = "f", conflicts_with = "in-place")]
        /// Overwriting existing secret forcefully.
        force: bool,
        /// The length of the secret.
        length: Option<usize>,
    },
    /// Remove existing secret or directory.
    Rm {
        #[structopt(required = true)]
        secret_name: String,
        #[structopt(long, short = "r")]
        /// Delete recursively.
        recursive: bool,
        #[structopt(long, short = "f")]
        /// Delete forcefully.
        force: bool,
    },
    /// Rename/move old-path to new-path.
    Mv {
        #[structopt(required = true)]
        old_path: String,
        #[structopt(required = true)]
        new_path: String,
        #[structopt(long, short = "f")]
        /// Move forcefully.
        force: bool,
    },
    /// Copy old-path to new-path.
    Cp {
        #[structopt(required = true)]
        old_path: String,
        #[structopt(required = true)]
        new_path: String,
        #[structopt(long, short = "f")]
        /// Copy forcefully.
        force: bool,
    },
    #[structopt(settings = &[AppSettings::TrailingVarArg, AppSettings::AllowLeadingHyphen])]
    /// Execute a git command specified by git-command-args inside the password
    /// store.
    Git { git_command_args: Vec<String> },
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
    /// Prints completion information to stdout for the specified shell.
    Complete {
        #[structopt(required = true)]
        /// One of `bash`, `fish`, `zsh`, `powershell`, or `elvish`
        /// (case-insensitive)
        shell: clap::Shell,
    },
}

/// For managing one-time-password (OTP) tokens with passrs
#[cfg(feature = "otp")]
#[derive(Debug, StructOpt)]
#[structopt(no_version)]
enum Otp {
    /// Generate and print an OTP code from the secret key in pass-name.
    Code {
        #[structopt(required = true)]
        secret_name: String,
        #[structopt(long, short = "c")]
        /// Optionally, put the generated code on the clipboard. If put on the
        /// clipboard, the code will be cleared in PASSWORD_STORE_CLIP_TIME in
        /// seconds, or 45 seconds if unspecified.
        clip: bool,
    },
    /// Insert OTP secret to pass-name.
    Insert {
        #[structopt(required = true)]
        secret_name: String,
        #[structopt(long, short = "f")]
        /// Overwriting existing secret forcefully.
        force: bool,
        #[structopt(long, short = "e")]
        /// Echo the secret back to the console during entry.
        echo: bool,
        #[structopt(long, short = "s")]
        /// Create an OTP URI from the provided secret. Assumes SHA1 algorithm,
        /// 30-second period, and 6 digits.
        from_secret: bool,
        #[structopt(long, requires = "from-secret")]
        /// One of SHA1, SHA256, or SHA512.
        algo: Option<String>,
        #[structopt(long, requires = "from-secret")]
        /// How often the OTP refreshes.
        period: Option<u32>,
        #[structopt(long, requires = "from-secret")]
        /// The length of the generated OTP code.
        digits: Option<usize>,
    },
    /// Append an OTP secret to pass-name.
    Append {
        #[structopt(required = true)]
        secret_name: String,
        #[structopt(long, short = "e")]
        /// Echo the secret back to the console during entry.
        echo: bool,
        #[structopt(long, short = "s")]
        /// Create an OTP URI from the provided secret. Assumes SHA1 algorithm,
        /// 30-second period, and 6 digits.
        from_secret: bool,
        #[structopt(long, requires = "from-secret")]
        /// One of SHA1, SHA256, or SHA512.
        algo: Option<String>,
        #[structopt(long, requires = "from-secret")]
        /// How often the OTP refreshes.
        period: Option<u32>,
        #[structopt(long, requires = "from-secret")]
        /// The length of the OTP code.
        digits: Option<usize>,
    },
    /// Print the key URI stored in pass-name.
    Uri {
        #[structopt(required = true)]
        secret_name: String,
        #[structopt(long, short = "c", conflicts_with = "qrcode")]
        /// Copy the URI to the clipboard.
        clip: bool,
        #[structopt(long, short = "q", conflicts_with = "clip")]
        /// Generate a QR code to the specified path.
        qrcode: Option<String>,
    },
    /// Test a URI string for validity according to the Key Uri Format.
    Validate {
        #[structopt(required = true)]
        uri: String,
    },
}

pub fn opt() -> Result<()> {
    let matches = Pass::from_args();
    dbg!(&matches);

    // NOTE: committing is handled inside any subcommand that may modify the
    // store
    match matches.subcmd {
        Some(sub) => match sub {
            PassSubcmd::Init { path, gpg_ids } => {
                init::init(path, gpg_ids)?;
            }
            PassSubcmd::Ls { subfolder } => {
                util::verify_store_exists()?;
                ls::ls(subfolder)?;
            }
            PassSubcmd::Find { secret_name } => {
                util::verify_store_exists()?;
                find::find(secret_name)?;
            }
            PassSubcmd::Show { clip, secret_name } => {
                util::verify_store_exists()?;
                show::show(clip, secret_name)?;
            }
            PassSubcmd::Grep { search_string } => {
                util::verify_store_exists()?;
                grep::grep(search_string)?;
            }
            PassSubcmd::Insert {
                echo,
                multiline,
                force,
                secret_name,
            } => {
                util::verify_store_exists()?;
                insert::insert(echo, multiline, force, secret_name)?;
            }
            PassSubcmd::Edit { secret_name } => {
                util::verify_store_exists()?;
                edit::edit(secret_name)?;
            }
            PassSubcmd::Generate {
                no_symbols,
                clip,
                in_place,
                force,
                secret_name,
                length,
            } => {
                util::verify_store_exists()?;
                generate::generate(no_symbols, clip, in_place, force, secret_name, length)?;
            }
            PassSubcmd::Rm {
                recursive,
                force,
                secret_name,
            } => {
                util::verify_store_exists()?;
                rm::rm(recursive, force, secret_name)?;
            }
            PassSubcmd::Mv {
                force,
                old_path,
                new_path,
            } => {
                util::verify_store_exists()?;
                mv::mv(force, old_path, new_path)?;
            }
            PassSubcmd::Cp {
                force,
                old_path,
                new_path,
            } => {
                util::verify_store_exists()?;
                cp::cp(force, old_path, new_path)?;
            }
            PassSubcmd::Git { git_command_args } => {
                util::verify_store_exists()?;
                git::git(git_command_args)?;
            }
            PassSubcmd::Unclip { timeout, force } => {
                util::verify_store_exists()?;
                unclip::unclip(timeout, force)?;
            }
            PassSubcmd::Complete { shell } => {
                Pass::clap().gen_completions_to(clap::crate_name!(), shell, &mut std::io::stdout());
            }
            #[cfg(feature = "otp")]
            PassSubcmd::Otp(otp) => {
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
                        from_secret,
                        algo,
                        period,
                        digits,
                    } => {
                        util::verify_store_exists()?;
                        insert::insert(
                            force,
                            echo,
                            secret_name,
                            from_secret,
                            algo,
                            period,
                            digits,
                        )?;
                    }
                    Otp::Append {
                        echo,
                        secret_name,
                        from_secret,
                        algo,
                        period,
                        digits,
                    } => {
                        util::verify_store_exists()?;
                        append::append(echo, secret_name, from_secret, algo, period, digits)?;
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
        },
        // If no command is specified, `ls` the entire password store, like
        // `pass` does
        None => {
            util::verify_store_exists()?;
            ls::ls(None)?;
        }
    }

    Ok(())
}
