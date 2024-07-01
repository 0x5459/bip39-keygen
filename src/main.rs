#![feature(split_array, io_error_more)]

use std::path;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::bail;
use bip39::Language;
use bip39::Mnemonic;
use clap::Parser;
use clap::Subcommand;
use clap::ValueEnum;
use secrecy::ExposeSecret;
use secrecy::SecretString;
use transaction::Transaction;
use zeroize::Zeroizing;

mod transaction;
mod version;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum KeyType {
    Ed25519,
}

impl KeyType {
    fn to_openssh_filename(&self) -> &'static str {
        match self {
            KeyType::Ed25519 => "id_ed25519",
        }
    }
}

#[derive(Parser)]
#[command(about, long_about, version = &**version::VERSION)]
struct Cli {
    #[command(subcommand)]
    commands: Commands,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Subcommand)]
enum Commands {
    /// Generates a random mnemonic
    New {
        /// Specify the number of words in the mnemonic
        #[arg(short, long, env, default_value_t = 12)]
        word_count: usize,
    },
    /// Generates an SSH key pair
    SSH {
        /// Specify the type of key you want to generate
        #[arg(short = 't', long, env)]
        key_type: KeyType,
        /// Specify an empty passphrase
        #[arg(short = 'N', long, env, default_value_t = false)]
        no_passphrase: bool,
        /// Specify the passphrase, if empty it will be prompted
        #[arg(short, long, env, default_value = "")]
        passphrase: SecretString,
        /// Specify the file path in which to save the key
        #[arg(short = 'f', long, env)]
        output_path: Option<PathBuf>,
        /// Specify the 12 words mnemonic, split by spaces. If not specified, it will be generated
        #[arg(short = 'm', long, env)]
        mnemonic: Option<SecretString>,
        /// Specify the comment for the key
        #[arg(short = 'C', long, default_value_t = ssh_default_comment(), env)]
        comment: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.commands {
        Commands::New { word_count } => {
            let mnemonic = Mnemonic::generate_in(Language::English, word_count)?;
            println!("{mnemonic}");
        }
        Commands::SSH {
            key_type,
            no_passphrase,
            passphrase,
            output_path,
            mnemonic: mnemonic_opt,
            comment,
        } => {
            let mnemonic = prompt_generate_mnemonic(mnemonic_opt)?;
            let passphrase = prompt_passphrase(if no_passphrase {
                Some(SecretString::new(String::new()))
            } else if passphrase.expose_secret().is_empty() {
                None
            } else {
                Some(passphrase)
            })?;

            let seckey_path = prompt_output_path(output_path, key_type)?;
            let pubkey_path = seckey_path.with_extension("pub");

            prompt_overwrite_path(&seckey_path)?;
            prompt_overwrite_path(&pubkey_path)?;

            let seed = mnemonic.to_seed(passphrase.expose_secret());
            let (seed32, _) = seed.split_array_ref::<32>();
            let keypair = ssh_key::private::KeypairData::Ed25519(
                ssh_key::private::Ed25519Keypair::from_seed(seed32),
            );
            let public_key = ssh_key::PublicKey::new(
                ssh_key::public::KeyData::try_from(&keypair)?,
                comment.clone(),
            );
            let secret_key = ssh_key::PrivateKey::new(keypair, comment)?;

            let txdir = tempfile::Builder::new().prefix("bip39-keygen").tempdir()?;
            let mut tx = Transaction::new(txdir);
            tx.write_file(pubkey_path, public_key.to_openssh()?)?;
            tx.write_file(seckey_path, secret_key.to_openssh(Default::default())?)?;
            tx.commit();
        }
    }
    Ok(())
}

fn ssh_default_output_path(key_type: KeyType) -> PathBuf {
    use std::path::MAIN_SEPARATOR;

    match home::home_dir() {
        Some(path) if !path.as_os_str().is_empty() => path.join(format!(
            ".ssh{MAIN_SEPARATOR}{}",
            key_type.to_openssh_filename()
        )),
        _ => PathBuf::new(),
    }
}

fn ssh_default_comment() -> String {
    format!(
        "{}@{}",
        whoami::username(),
        whoami::fallible::hostname().unwrap_or_else(|_| "localhost".to_string())
    )
}

#[allow(dead_code)]
fn gpg_default_output_dir() -> PathBuf {
    match home::home_dir() {
        Some(path) if !path.as_os_str().is_empty() => path.join(".gnupg"),
        _ => PathBuf::new(),
    }
}

fn prompt_passphrase(passphrase_opt: Option<SecretString>) -> anyhow::Result<SecretString> {
    match passphrase_opt {
        Some(passphrase) => Ok(passphrase),
        None => Ok(SecretString::new(
            inquire::Password::new("Enter passphrase (empty for no passphrase):")
                .with_display_mode(inquire::PasswordDisplayMode::Masked)
                .with_custom_confirmation_message("Confirmation passphrase")
                .prompt()?,
        )),
    }
}

fn prompt_output_path(outpath: Option<PathBuf>, key_type: KeyType) -> anyhow::Result<PathBuf> {
    let path = match outpath {
        Some(path) => path,
        None => {
            let mut text = inquire::Text::new("Enter file in which to save the key");
            let default_path = ssh_default_output_path(key_type);
            if let Some(default) = default_path.to_str() {
                text = text.with_default(default);
            }
            PathBuf::from(text.prompt()?)
        }
    };
    Ok(path::absolute(path)?)
}

fn prompt_overwrite_path(path: &Path) -> anyhow::Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let ans = inquire::Confirm::new(&format!("{} already exists, overwrite?", path.display()))
        .with_default(false)
        .prompt();

    match ans {
        Ok(true) => Ok(()),
        _ => bail!("Aborted"),
    }
}

fn prompt_generate_mnemonic(
    mnemonic_opt: Option<SecretString>,
) -> anyhow::Result<Zeroizing<Mnemonic>> {
    if let Some(mnemonic) = mnemonic_opt {
        return Ok(Zeroizing::new(Mnemonic::from_str(
            mnemonic.expose_secret(),
        )?));
    }

    match inquire::Select::new("Choose mnemonic option", vec![
        "Input mnemonic",
        "Generate new mnemonic",
    ])
    .prompt()?
    {
        "Input mnemonic" => {
            let mnemonic = SecretString::new(
                inquire::Text::new("Enter your 12-word mnemonic (separate words with spaces)")
                    .prompt()?,
            );
            Ok(Zeroizing::new(Mnemonic::from_str(
                mnemonic.expose_secret(),
            )?))
        }
        "Generate new mnemonic" => loop {
            let mnemonic = Mnemonic::generate_in(Language::English, 12)?;

            println!("Your new 12-word mnemonic is:");
            println!("  {}", mnemonic);
            println!("Please write it down and store it in a safe place");

            let ans = inquire::Confirm::new("Do you want to regenerate a new mnemonic?")
                .with_default(false)
                .prompt()?;
            if !ans {
                break Ok(Zeroizing::new(mnemonic));
            }
        },
        _ => unreachable!(),
    }
}
