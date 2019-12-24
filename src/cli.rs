// TODO: refactor git and gpg usage into separate modules
// use git2::Repository;
// use gpgme::{Context, Protocol};
use structopt::clap::AppSettings;
use structopt::StructOpt;

// TODO: flesh out error enum and handling
// use crate::error::PassrsError;
use crate::consts::VERSION;
use crate::subcmds::*;

#[derive(Debug, StructOpt)]
#[structopt(
    set_term_width(80),
    settings = &[AppSettings::ArgsNegateSubcommands,
                 AppSettings::DeriveDisplayOrder,
                 AppSettings::VersionlessSubcommands],
    version = VERSION.as_str())]
enum Pass {
    /// Initialize new password store and use the provided gpg-id for encryption
    Init {
        #[structopt(long, short = "p")]
        path: Option<String>,
        #[structopt(required = true)]
        key: String,
    },
    // TODO: make default command
    /// List passwords
    Ls { subfolder: Option<String> },
    /// List passwords that match pass-names
    Find {
        #[structopt(required = true)]
        pass_names: Vec<String>,
    },
    /// Show existing password and optionally put it on the clipboard
    /// If put on the clipboard, the password will be cleared in 45 seconds
    Show {
        // TODO: make it so that we can provide either just the flag, or an optional line number
        #[structopt(long, short = "c")]
        clip: bool,
        // #[allow(clippy::option_option)]
        // clip: Option<Option<u32>>,
        // Some(None) -> just copy, Some(Some(u32)) -> copy line
        pass_name: String,
    },
    /// Search for password files containing search-string when decrypted
    Grep {
        #[structopt()]
        search_string: String,
        // TODO: GREPOPTIONS
        #[structopt(required = false)]
        grepoptions: Vec<String>,
    },
    /// Insert new password
    Insert {
        #[structopt(long, short = "e", conflicts_with = "multiline")]
        /// Echo the password back to the console during entry
        echo: bool,
        #[structopt(long, short = "m", conflicts_with = "echo")]
        /// Enable multiline mode
        multiline: bool,
        #[structopt(long, short = "f")]
        /// Overwriting existing password forcefully
        force: bool,
        #[structopt(required = true)]
        pass_name: String,
    },
    /// Insert a new password or edit an existing password using $EDITOR
    Edit {
        #[structopt(required = true)]
        pass_name: String,
    },
    /// Generate a new password of pass-length, or 24 if unspecified
    Generate {
        #[structopt(long, short = "n")]
        /// Disable symbols
        no_symbols: bool,
        #[structopt(long, short = "c")]
        /// Copy the generated password to the clipboard, which clears after 45 seconds
        clip: bool,
        #[structopt(long, short = "i", conflicts_with = "force")]
        /// Remove only the first line of an existing file with a new password
        in_place: bool,
        #[structopt(long, short = "f", conflicts_with = "in_place")]
        /// Overwriting existing password forcefully
        force: bool,
        #[structopt(required = true)]
        pass_name: String,
        #[structopt(default_value = "24")]
        pass_length: u32,
    },
    /// Remove existing password or directory
    Rm {
        #[structopt(long, short = "r")]
        /// Delete recursively
        recursive: bool,
        #[structopt(long, short = "f")]
        /// Delete forcefully
        force: bool,
        #[structopt(required = true)]
        pass_name: String,
    },
    /// Rename or move old-path to new-path
    Mv {
        #[structopt(long, short = "f")]
        /// Move forcefully
        force: bool,
        old_path: String,
        new_path: String,
    },
    /// Copy old-path to new-path
    Cp {
        #[structopt(long, short = "f")]
        /// Copy forcefully
        force: bool,
        old_path: String,
        new_path: String,
    },
    // TODO: investigate whether or not it's worth it to use a Rust-git library
    /// If the password store is a git repository, execute a git command
    /// specified by git-command-args
    #[structopt(setting = AppSettings::TrailingVarArg)]
    Git {
        #[structopt(required = true)]
        git_command_args: Vec<String>,
    },
    #[cfg(feature = "otp")]
    // TODO: do I really want to deal with HOTP?
    /// Manage OTP tokens
    Otp(Otp),
    #[structopt(setting = AppSettings::Hidden)]
    Unclip {
        #[structopt(required = true)]
        timeout: u64,
        #[structopt(long, short = "f")]
        force: bool,
    },
    #[structopt(setting = AppSettings::Hidden)]
    #[doc(hidden)]
    __Unexhaustive,
}

// TODO: documentation
// requies otpauth:// scheme
/// pass-otp(1)
#[cfg(feature = "otp")]
#[derive(Debug, StructOpt)]
enum Otp {
    #[structopt(setting = AppSettings::DisableVersion)]
    Code {
        #[structopt(long, short = "c")]
        clip: bool,
        #[structopt(required = true)]
        pass_name: String,
    },
    #[structopt(setting = AppSettings::DisableVersion)]
    /// Insert OTP secret to pass-name
    // TODO: if pass_name is a folder, write to pass_name/otp
    Insert {
        #[structopt(long, short = "f")]
        /// Overwriting existing secret forcefully
        force: bool,
        #[structopt(long, short = "e")]
        /// Echo the secret back to the console during entry
        echo: bool,
        #[structopt(required = true)]
        pass_name: String,
        // TODO: algo, period, and length
        /// Assumes SHA1 algorithm, 30-second period, and 6 digits
        secret: Option<String>,
    },
    #[structopt(setting = AppSettings::DisableVersion)]
    /// Append an OTP secret to pass-name
    // TODO: if pass_name is a folder, write to pass_name/otp
    Append {
        #[structopt(long, short = "e")]
        /// Echo the secret back to the console during entry
        echo: bool,
        #[structopt(required = true)]
        pass_name: String,
        // TODO: algo, period, and length
        /// Assumes SHA1 algorithm, 30-second period, and 6 digits
        secret: Option<String>,
    },
    #[structopt(setting = AppSettings::DisableVersion)]
    /// Print the key URI stored in pass-name
    Uri {
        #[structopt(long, short = "c", conflicts_with = "qrcode")]
        clip: bool,
        #[structopt(long, short = "q", conflicts_with = "clip")]
        qrcode: bool,
        #[structopt(required = true)]
        pass_name: String,
    },
    #[structopt(setting = AppSettings::DisableVersion)]
    /// Test a URI string for validity according to the Key Uri Format
    Validate {
        #[structopt(required = true)]
        uri: String,
    },
}

use failure::Fallible;

// TODO: crate::Result<T>
pub fn opt() -> Fallible<()> {
    // seed RNG early
    let mut rng = rand::thread_rng();
    let matches = Pass::from_args();
    #[cfg(debug_assertions)]
    println!("{:#?}", matches);
    let mut commit_message = None;

    match matches {
        Pass::Init { path, key } => init::init(path, key),
        Pass::Ls { subfolder } => ls::ls(subfolder),
        Pass::Find { pass_names } => find::find(pass_names),
        Pass::Show { clip, pass_name } => show::show(clip, pass_name),
        Pass::Grep {
            search_string,
            grepoptions,
        } => grep::grep(search_string, grepoptions),
        Pass::Insert {
            echo,
            multiline,
            force,
            pass_name,
        } => commit_message = insert::insert(echo, multiline, force, pass_name),
        Pass::Edit { pass_name } => commit_message = edit::edit(pass_name),
        Pass::Generate {
            no_symbols,
            clip,
            in_place,
            force,
            pass_name,
            pass_length,
        } => {
            commit_message = generate::generate(
                &mut rng,
                no_symbols,
                clip,
                in_place,
                force,
                pass_name,
                pass_length,
            )
        }
        Pass::Rm {
            recursive,
            force,
            pass_name,
        } => rm::rm(recursive, force, pass_name),
        Pass::Mv {
            force,
            old_path,
            new_path,
        } => mv::mv(force, old_path, new_path),
        Pass::Cp {
            force,
            old_path,
            new_path,
        } => cp::cp(force, old_path, new_path),
        Pass::Git { git_command_args } => git::git(git_command_args),
        Pass::Unclip { timeout, force } => unclip::unclip(timeout, force),
        Pass::Otp(otp) => {
            use crate::subcmds::otp::*;

            match otp {
                Otp::Code { clip, pass_name } => code::code(clip, pass_name)?,
                Otp::Insert {
                    force,
                    echo,
                    pass_name,
                    secret,
                } => insert::insert(force, echo, pass_name, secret)?,
                Otp::Append {
                    echo,
                    pass_name,
                    secret,
                } => append::append(echo, pass_name, secret)?,
                Otp::Uri {
                    clip,
                    qrcode,
                    pass_name,
                } => uri::uri(clip, qrcode, pass_name)?,
                Otp::Validate { uri } => validate::validate(uri)?,
            }
        }
        _ => {}
    }

    // TODO: check dirty and commit
    // if worktree is dirty {
    //     commit(commit_message)
    if let Some(message) = commit_message {
        let _ = message;
    }

    Ok(())
    // }
}
