- document everything (see main.rs for links)
- clean up code
- ensure directories exist in functions that try to create encrypted files
    (fairly confident this is done, but should be verified)
- check collects and see if they can't be removed or replaced

# At a later date
- gopass functions
  - audit (scans for weak passwords -- maybe?)
    - search for duplicate passwords? -- calculate hash of contents, store in
        HashMap<filename, hash>, if entry already exists, report filename
  - binary (binary -> base64, then enc)
  - completions
