- document everything (see main.rs for links)
- clean up code
- check collects and see if they can't be removed or replaced
- completion should be able to complete entries (see `pass.fish-completion`)
    https://raw.githubusercontent.com/zx2c4/password-store/master/src/completion/pass.fish-completion
- setup tests (ref: https://git.zx2c4.com/password-store/tree/tests)

# At a later date
- gopass functions
  - audit (scans for weak passwords -- maybe?)
    - search for duplicate passwords? -- calculate hash of contents, store in
        HashMap<filename, hash>, if entry already exists, report filename
  - binary (binary -> base64, then enc)
