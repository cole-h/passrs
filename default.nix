(import
  (fetchTarball {
    url = "https://github.com/edolstra/flake-compat/archive/c71d063a2fce96001c7ef99b301dfbd0c29ebde1.tar.gz";
    sha256 = "0vnbhqf0lc4mf2zmzqbfv6kqj9raijxz8xfaimxihz3c6s5gma2x";
  })
  { src = ./.; }).defaultNix
