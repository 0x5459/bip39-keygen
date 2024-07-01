# Bip39-keygen
![CI](https://github.com/0x5459/bip39-keygen/actions/workflows/release.yml/badge.svg)

A tool for converting BIP39 mnemonic phrases to SSH, GPG... key pairs.

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
| ssh-dsa        |   🚧    |
| ssh-ecdh       |   🚧    |
| ssh-ecdsa      |   🚧    |
| ssh-eddsa      |   🚧    |
