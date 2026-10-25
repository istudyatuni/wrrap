# Run executable in isolated environment

What's done:

- share network
- inherit some env variables
<!-- - inherit `/nix/store` paths from `$PATH` -->
- ro bind `/nix`, `/bin`, `/usr/bin` folders, ssl certificates
- rw bind current directory
- ro bind `.git` and `.jj`
- hide git-ignored paths in current directory (bind as empty dirs/files).
  - can be overriden by profiles
- when running inside git-ignored dir, bind it as-is
- bind dirs/files defined by profiles

## Profiles

### Go

- bind `~/go`

### NodeJS (npm)

- bind `~/.npm`
- not hide `node_modules`

### NodeJS (npm)

- bind `~/.config/yarn`, `~/.yarn`
- not hide `node_modules`

### Rust

- bind
  - `~/.cargo/bin`
  - `~/.cargo/build`
  - `~/.cargo/git`
  - `~/.cargo/registry`
  - `~/.rustup`
- not hide `target`
- create custom `~/.cargo/config.toml`:
  ```toml
  [target.x86_64-unknown-linux-gnu]
  linker = "cc"
  rustflags = ["-C", "link-arg=-fuse-ld=mold"]
  ```
