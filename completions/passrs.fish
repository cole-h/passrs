#!/usr/bin/env fish
set BIN 'passrs'

function __fish_passrs_store
    set -l store "$PASSWORD_STORE_DIR"
    if [ -z "$store" ]
        set store "$HOME/.password-store"
    end
    echo "$store"
end

function __fish_passrs_keys
    gpg2 --list-secret-keys | grep uid | string replace -r '.*<(.*)>' '$1'
end

function __fish_passrs_entries
    set -l store (__fish_passrs_store)
    printf "%s\n" "$store"/**.gpg | string replace -r "$store"'/(.*)' '$1' | string replace -r '(.*)\.gpg' '$1'
end

function __fish_passrs_dirs
    set -l store (__fish_passrs_store)
    printf "%s\n" "$store"/**/ | string replace -r "$store"'/(.*)' '$1'
end

function __fish_passrs_all
    __fish_passrs_dirs
    __fish_passrs_entries
end

# erase any completions that might already exist
complete -c $BIN -e

# passrs
complete -c $BIN -f -n "__fish_use_subcommand" \
    -a "help" -d 'Prints this message or the help of the given subcommand(s)'
complete -c $BIN -f -n "__fish_use_subcommand" -s h -l help -d 'Prints help information'
complete -c $BIN -f -n "__fish_use_subcommand" -s V -l version -d 'Prints version information'

## init
complete -c $BIN -f -n "__fish_use_subcommand" -f -a "init" -d 'Initialize a new store or substore'
complete -c $BIN -f -n "__fish_seen_subcommand_from init" \
    -s p -l path -d 'The specified gpg-id(s) is assigned to the specified subfolder'
complete -c $BIN -f -n "__fish_seen_subcommand_from init" -s h -l help -d 'Prints help information'
complete -c $BIN -f \
    -n "__fish_seen_subcommand_from init; and not __fish_seen_subcommand_from (__fish_passrs_keys)" \
    -a "(__fish_passrs_keys)"

## ls
complete -c $BIN -f -n "__fish_use_subcommand" -f -a "ls" -d 'List all secrets'
complete -c $BIN -f -n "__fish_seen_subcommand_from ls" -s h -l help -d 'Prints help information'
complete -c $BIN -f \
    -n "__fish_seen_subcommand_from ls; and not __fish_seen_subcommand_from (__fish_passrs_dirs)" \
    -a "(__fish_passrs_dirs)"

## find
complete -c $BIN -f -n "__fish_use_subcommand" -f -a "find" -d 'List secrets that match secret-name'
complete -c $BIN -f -n "__fish_seen_subcommand_from find" -s h -l help -d 'Prints help information'

## show
complete -c $BIN -f -n "__fish_use_subcommand" -f -a "show" -d 'Show existing secret'
complete -c $BIN -f -n "__fish_seen_subcommand_from show" \
    -s c -l clip -d 'Optionally, put the secret on the clipboard'
complete -c $BIN -f -n "__fish_seen_subcommand_from show" -s h -l help -d 'Prints help information'
complete -c $BIN -f \
    -n "__fish_seen_subcommand_from show; and not __fish_seen_subcommand_from (__fish_passrs_entries)" \
    -a "(__fish_passrs_entries)"

## grep
complete -c $BIN -f -n "__fish_use_subcommand" -a "grep" -d 'Search for pattern in secrets'
complete -c $BIN -f -n "__fish_seen_subcommand_from grep" -s h -l help -d 'Prints help information'

## insert
complete -c $BIN -f -n "__fish_use_subcommand" -a "insert" -d 'Insert a new secret'
complete -c $BIN -f -n "__fish_seen_subcommand_from insert" \
    -s e -l echo -d 'Echo the secret back to the console during entry'
complete -c $BIN -f -n "__fish_seen_subcommand_from insert" \
    -s f -l force -d 'Overwriting existing secret forcefully'
complete -c $BIN -f -n "__fish_seen_subcommand_from insert" \
    -s m -l multiline -d 'Enable multiline mode'
complete -c $BIN -f -n "__fish_seen_subcommand_from insert" \
    -s h -l help -d 'Prints help information'
complete -c $BIN -f \
    -n "__fish_seen_subcommand_from insert; and not __fish_seen_subcommand_from (__fish_passrs_dirs)" \
    -a "(__fish_passrs_dirs)"

## edit
complete -c $BIN -f -n "__fish_use_subcommand" \
    -a "edit" -d 'Insert a new secret or edit an existing one using $EDITOR'
complete -c $BIN -f -n "__fish_seen_subcommand_from edit" -s h -l help -d 'Prints help information'
complete -c $BIN -f \
    -n "__fish_seen_subcommand_from edit; and not __fish_seen_subcommand_from (__fish_passrs_entries)" \
    -a "(__fish_passrs_entries)"

## generate
complete -c $BIN -f -n "__fish_use_subcommand" -a "generate" -d 'Generate a new secret'
complete -c $BIN -f -n "__fish_seen_subcommand_from generate" \
    -s c -l clip -d 'Optionally, put the secret on the clipboard'
complete -c $BIN -f -n "__fish_seen_subcommand_from generate" \
    -s f -l force -d 'Overwriting existing secret forcefully'
complete -c $BIN -f -n "__fish_seen_subcommand_from generate" \
    -s i -l in-place -d 'Remove only the first line of an existing file with a new secret'
complete -c $BIN -f -n "__fish_seen_subcommand_from generate" \
    -s n -l no-symbols -d 'Disable special symbols'
complete -c $BIN -f -n "__fish_seen_subcommand_from generate" \
    -s h -l help -d 'Prints help information'
complete -c $BIN -f \
    -n "__fish_seen_subcommand_from generate; and not __fish_seen_subcommand_from (__fish_passrs_dirs)" \
    -a "(__fish_passrs_dirs)"

## rm
complete -c $BIN -f -n "__fish_use_subcommand" -a "rm" -d 'Remove existing secret or directory'
complete -c $BIN -f -n "__fish_seen_subcommand_from rm" -s f -l force -d 'Delete forcefully'
complete -c $BIN -f -n "__fish_seen_subcommand_from rm" -s r -l recursive -d 'Delete recursively'
complete -c $BIN -f -n "__fish_seen_subcommand_from rm" -s h -l help -d 'Prints help information'
complete -c $BIN -f \
    -n "__fish_seen_subcommand_from rm; and not __fish_seen_subcommand_from (__fish_passrs_all)" \
    -a "(__fish_passrs_all)"

## mv
complete -c $BIN -f -n "__fish_use_subcommand" -a "mv" -d 'Move old-path to new-path'
complete -c $BIN -f -n "__fish_seen_subcommand_from mv" -s f -l force -d 'Move forcefully'
complete -c $BIN -f -n "__fish_seen_subcommand_from mv" -s h -l help -d 'Prints help information'
complete -c $BIN -f \
    -n "__fish_seen_subcommand_from mv; and not __fish_seen_subcommand_from (__fish_passrs_all)" \
    -a "(__fish_passrs_all)"

## cp
complete -c $BIN -f -n "__fish_use_subcommand" -a "cp" -d 'Copy old-path to new-path'
complete -c $BIN -f -n "__fish_seen_subcommand_from cp" -s f -l force -d 'Copy forcefully'
complete -c $BIN -f -n "__fish_seen_subcommand_from cp" -s h -l help -d 'Prints help information'
complete -c $BIN -f \
    -n "__fish_seen_subcommand_from cp; and not __fish_seen_subcommand_from (__fish_passrs_all)" \
    -a "(__fish_passrs_all)"

## git
complete -c $BIN -f -n "__fish_use_subcommand" -a "git" -d 'Execute a git command inside the password store'
complete -c $BIN -f -n "__fish_seen_subcommand_from git" -s h -l help -d 'Prints help information'
complete -c $BIN -f -n "__fish_seen_subcommand_from git" -a "(__fish_complete_subcommand)"


# OTP
set -l otp_subcmds code insert append uri validate help
complete -c $BIN -f -n "__fish_use_subcommand" -a "otp" -d 'Manage TOTP secrets'
complete -c $BIN -f -n "__fish_seen_subcommand_from otp" -s h -l help -d 'Prints help information'

## otp code
complete -c $BIN -f \
    -n "__fish_seen_subcommand_from otp; and not __fish_seen_subcommand_from $otp_subcmds" \
    -a "code" -d 'Generate and print a TOTP code from the key in secret-name'
complete -c $BIN -f \
    -n "__fish_seen_subcommand_from code" -s c -l clip -d 'Optionally, put the generated code on the clipboard'
complete -c $BIN -f \
    -n "__fish_seen_subcommand_from code" -s h -l help -d 'Prints help information'
complete -c $BIN -f \
    -n "__fish_seen_subcommand_from code; and not __fish_seen_subcommand_from (__fish_passrs_entries)" \
    -a "(__fish_passrs_entries)"

## otp insert
complete -c $BIN -f \
    -n "__fish_seen_subcommand_from otp; and not __fish_seen_subcommand_from $otp_subcmds" \
    -a "insert" -d 'Insert TOTP secret to secret-name'
complete -c $BIN -f \
    -n "__fish_seen_subcommand_from insert" -l algorithm -d 'One of SHA1, SHA256, or SHA512'
complete -c $BIN -f \
    -n "__fish_seen_subcommand_from insert" -l period -d 'How often the TOTP refreshes'
complete -c $BIN -f \
    -n "__fish_seen_subcommand_from insert" -l digits -d 'The length of the generated TOTP code'
complete -c $BIN -f \
    -n "__fish_seen_subcommand_from insert" -s e -l echo -d 'Echo the secret back to the console during entry'
complete -c $BIN -f \
    -n "__fish_seen_subcommand_from insert" -s f -l force -d 'Overwriting existing secret forcefully'
complete -c $BIN -f \
    -n "__fish_seen_subcommand_from insert" -s g -l generate -d 'Generate a TOTP code from the newly-inserted secret'
complete -c $BIN -f \
    -n "__fish_seen_subcommand_from insert" -s s -l from-secret -d 'Create a TOTP URI from the provided secret. Assumes SHA1 algorithm, 30-second period, and 6 digits'
complete -c $BIN -f \
    -n "__fish_seen_subcommand_from insert" -s h -l help -d 'Prints help information'
complete -c $BIN -f \
    -n "__fish_seen_subcommand_from insert; and not __fish_seen_subcommand_from (__fish_passrs_dirs)" \
    -a "(__fish_passrs_dirs)"

## otp append
complete -c $BIN -f \
    -n "__fish_seen_subcommand_from otp; and not __fish_seen_subcommand_from $otp_subcmds" \
    -a "append" -d 'Append a TOTP secret to secret-name'
complete -c $BIN -f \
    -n "__fish_seen_subcommand_from append" -l algorithm -d 'One of SHA1, SHA256, or SHA512'
complete -c $BIN -f \
    -n "__fish_seen_subcommand_from append" -l period -d 'How often the TOTP refreshes'
complete -c $BIN -f \
    -n "__fish_seen_subcommand_from append" -l digits -d 'The length of the TOTP code'
complete -c $BIN -f \
    -n "__fish_seen_subcommand_from append" -s e -l echo -d 'Echo the secret back to the console during entry'
complete -c $BIN -f \
    -n "__fish_seen_subcommand_from append" -s s -l from-secret -d 'Create a TOTP URI from the provided secret. Assumes SHA1 algorithm, 30-second period, and 6 digits'
complete -c $BIN -f \
    -n "__fish_seen_subcommand_from append" -s g -l generate -d 'Generate a TOTP code from the newly-appended secret'
complete -c $BIN -f \
    -n "__fish_seen_subcommand_from append" -s h -l help -d 'Prints help information'
complete -c $BIN -f \
    -n "__fish_seen_subcommand_from append; and not __fish_seen_subcommand_from (__fish_passrs_entries)" \
    -a "(__fish_passrs_entries)"

## otp uri
complete -c $BIN -f \
    -n "__fish_seen_subcommand_from otp; and not __fish_seen_subcommand_from $otp_subcmds" \
    -a "uri" -d 'Print the key URI stored in secret-name'
complete -c $BIN -f \
    -n "__fish_seen_subcommand_from uri" -s c -l clip -d 'Copy the URI to the clipboard'
complete -c $BIN -f \
    -n "__fish_seen_subcommand_from uri" -s q -l qrcode -d 'Generate a QR code to stdout'
complete -c $BIN -f \
    -n "__fish_seen_subcommand_from uri" -s h -l help -d 'Prints help information'
complete -c $BIN -f \
    -n "__fish_seen_subcommand_from uri; and not __fish_seen_subcommand_from (__fish_passrs_entries)" \
    -a "(__fish_passrs_entries)"

## otp validate
complete -c $BIN -f \
    -n "__fish_seen_subcommand_from otp; and not __fish_seen_subcommand_from $otp_subcmds" \
    -a "validate" -d 'Test a URI for validity according to the Key Uri Format'
complete -c $BIN -f \
    -n "__fish_seen_subcommand_from validate" -s h -l help -d 'Prints help information'

## otp help
complete -c $BIN -f \
    -n "__fish_seen_subcommand_from otp; and not __fish_seen_subcommand_from $otp_subcmds" \
    -a "help" -d 'Prints this message or the help of the given subcommand(s)'
complete -c $BIN -f \
    -n "__fish_seen_subcommand_from help" -s h -l help -d 'Prints help information'
