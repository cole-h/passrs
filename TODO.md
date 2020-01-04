- = todo
+ = done
/ = somewhat done
. = not going to implement

- notify when clipboard is cleared

TODO: ensure canonicalize_path is used
TODO: ensure directories exist in functions that try to create encrypted files

/ show
  - 'e' to spawn editor
- otp
  + code
  - insert
  - append
  + uri
  + validate

- unix pass functions
  -    pass [show] [--clip[=line-number],-c[line-number]] pass-name
          Show existing password and optionally put it on the clipboard.
          If put on the clipboard, it will be cleared in 45 seconds.
- unix pass env vars
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
