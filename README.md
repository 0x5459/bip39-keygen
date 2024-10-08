# Bip39-keygen
![CI](https://github.com/0x5459/bip39-keygen/actions/workflows/release.yml/badge.svg)

A tool for converting [BIP39](https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki) mnemonic phrases to SSH, GPG... key pairs.

## Usage

### Generates a random mnemonic
```
./bip39-keygen new
```

### Generate SSH key pair
```
Generates an SSH key pair

Usage: bip39-keygen ssh [OPTIONS] --key-type <KEY_TYPE>

Options:
  -t, --key-type <KEY_TYPE>        Specify the type of key you want to generate [env: KEY_TYPE=] [possible values: ed25519]
  -N, --no-passphrase              Specify an empty passphrase [env: NO_PASSPHRASE=]
  -p, --passphrase <PASSPHRASE>    Specify the passphrase, if empty it will be prompted [env: PASSPHRASE=] [default: ]
  -f, --output-path <OUTPUT_PATH>  Specify the file path in which to save the key [env: OUTPUT_PATH=]
  -m, --mnemonic <MNEMONIC>        Specify the 12 words mnemonic, split by spaces. If not specified, it will be generated [env: MNEMONIC=]
  -C, --comment <COMMENT>          Specify the comment for the key [env: COMMENT=] [default: taoyu@DESKTOP-K0MPBM7]
  -h, --help                       Print help
  ```

### Features

| Feature name   | Status |
| :------------- | :----: |
| ssh-ED25519    |   ✅    |
| ssh-dsa        |   🚧    |
| ssh-ecdsa      |   🚧    |
| ssh-ecdsa-sk   |   🚧    |
| ssh-ed25519-sk |   🚧    |
| ssh-rsa        |   🚧    |
| gpg-rsa        |   🚧    |
| gpg-elg        |   🚧    |
| gpg-dsa        |   🚧    |
| gpg-ecdh       |   🚧    |
| gpg-ecdsa      |   🚧    |
| gpg-eddsa      |   🚧    |


#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>

