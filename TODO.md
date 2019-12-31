- = todo
+ = done
/ = somewhat done
. = not going to implement

- notify when clipboard is cleared

TODO: ensure canonicalize_path is used
TODO: ensure directories exist in functions that try to create encrypted files

/ init
+ ls
/ find
/ show
  - 'e' to spawn editor
+ grep
/ insert
/ edit
+ generate
  + The generated password for **Network/nextcloud/cole/password** (underlined) is:
      **password_here!lol** (in yellow)
+ rm
+ mv
+ cp
+ git
+ unclip
- otp
  + code
  - insert
  - append
  + uri
  + validate

- unix pass functions
  -    pass init [--path=subfolder,-p subfolder] gpg-id...
          Initialize new password storage and use gpg-id for encryption.
          Selectively reencrypt existing passwords using new gpg-id.
  /    pass [ls] [subfolder]
          List passwords.
  -    pass find pass-names...
        List passwords that match pass-names.
  -    pass [show] [--clip[=line-number],-c[line-number]] pass-name
          Show existing password and optionally put it on the clipboard.
          If put on the clipboard, it will be cleared in 45 seconds.
  +    pass grep [GREPOPTIONS] search-string
          Search for password files containing search-string when decrypted.
  -    pass insert [--echo,-e | --multiline,-m] [--force,-f] pass-name
          Insert new password. Optionally, echo the password back to the console
          during entry. Or, optionally, the entry may be multiline. Prompt before
          overwriting existing password unless forced.
  -    pass edit pass-name
          Insert a new password or edit an existing password using nvim.
  -    pass generate [--no-symbols,-n] [--clip,-c] [--in-place,-i | --force,-f] pass-name [pass-length]
          Generate a new password of pass-length (or 25 if unspecified) with optionally no symbols.
          Optionally put it on the clipboard and clear board after 45 seconds.
          Prompt before overwriting existing password unless forced.
          Optionally replace only the first line of an existing file with a new password.
  -    pass rm [--recursive,-r] [--force,-f] pass-name
          Remove existing password or directory, optionally forcefully.
  -    pass mv [--force,-f] old-path new-path
          Renames or moves old-path to new-path, optionally forcefully, selectively reencrypting.
  -    pass cp [--force,-f] old-path new-path
          Copies old-path to new-path, optionally forcefully, selectively reencrypting.
  +    pass git git-command-args...
          If the password store is a git repository, execute a git command
          specified by git-command-args.
  +    pass help
          Show this text.
  +    pass version
          Show version information.
- unix pass env vars
  +    PASSWORD_STORE_DIR
              Overrides the default password storage directory.
  /    PASSWORD_STORE_KEY
              Overrides the default gpg key identification set by init. Keys
              must not contain spaces and thus use of the hexadecimal key
              signature is recommended.  Multiple keys may be specified
              separated by spaces.
  -    PASSWORD_STORE_GPG_OPTS
              Additional options to be passed to all invocations of GPG.
  - (mod::clipboard)    PASSWORD_STORE_X_SELECTION
              Overrides the selection passed to xclip, by default clipboard. See
              xclip(1) for more info.
  +    PASSWORD_STORE_CLIP_TIME
              Specifies the number of seconds to wait before restoring the
              clipboard, by default 45 seconds.
  -    PASSWORD_STORE_UMASK
              Sets the umask of all files modified by pass, by default 077.
  +    PASSWORD_STORE_GENERATED_LENGTH
              The default password length if the pass-length parameter to
              generate is unspecified.
  -    PASSWORD_STORE_CHARACTER_SET
              The  character  set to be used in password generation for
              generate. This value is to be interpreted by tr. See tr(1) for
              more info.
  -    PASSWORD_STORE_CHARACTER_SET_NO_SYMBOLS
              The character set to be used in no-symbol password generation for
              generate, when --no-symbols,  -n  is specified. This value is to
              be interpreted by tr. See tr(1) for more info.
  .    PASSWORD_STORE_ENABLE_EXTENSIONS
              This environment variable must be set to "true" for extensions to
              be enabled.
  .    PASSWORD_STORE_EXTENSIONS_DIR
              The location to look for executable extension files, by default
              PASSWORD_STORE_DIR/.extensions.
  /    PASSWORD_STORE_SIGNING_KEY
              If  this  environment  variable  is set, then all .gpg-id files
              and non-system extension files must be signed using a detached
              signature using the GPG key specified by the full 40 character
              upper-case fingerprint  in  this  variable.  If  multiple
              fingerprints are specified, each separated by a whitespace
              character, then signatures must match at least one.  The init
              command will keep signatures of  .gpg-id files up to date.

- gopass functions
  - audit (scans for weak passwords -- maybe?)
    - search for duplicate passwords? -- calculate hash of contents, store in
        HashMap<filename, hash>, if entry already exists, report filename
  - binary (binary -> base64, then enc)
  - completions
  - *fuzzy search*
  - **fuzzy search if no args specified**
