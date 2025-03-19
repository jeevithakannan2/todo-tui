use aes_gcm::{Aes256Gcm, KeyInit, Nonce, aead::Aead};
use core::panic;
use directories::BaseDirs;
use rand::RngCore;
use std::{fs, io::Write, path::PathBuf};

use crate::tasks::Task;

/// Path to store the encryption key
fn key_path() -> PathBuf {
    let dirs = BaseDirs::new().expect("Failed to find home directory");
    dirs.home_dir().join(".todotui_key")
}

/// Generate a new encryption key
pub fn generate_key() {
    let key_file = key_path();
    let mut key = vec![0u8; 32]; // AES-256 key
    rand::thread_rng().fill_bytes(&mut key);

    let mut file = fs::File::create(&key_file).expect("Failed to create key file");
    file.write_all(&key).expect("Failed to write key");

    #[cfg(target_family = "unix")]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(&key_file).expect("Failed to read metadata");
        let mut perms = metadata.permissions();
        perms.set_mode(0o600); // Owner read/write only equals to chmod 600 cmd
        fs::set_permissions(&key_file, perms).expect("Failed to set permissions");
    }

    #[cfg(target_os = "windows")]
    {
        use std::env;
        use std::process::Command;

        let key_path = key_file.to_str().unwrap();
        let username = env::var("USERNAME").expect("Failed to get USERNAME environment variable");

        // PowerShell command to set NTFS ACLs like SSH keys
        let ps_script = format!(
            "icacls \"{}\" /inheritance:r /remove:g *S-1-1-0 /grant \"{}:RW\"",
            key_path, username
        );

        Command::new("powershell")
            .args(&["-Command", &ps_script])
            .status()
            .expect("Failed to set file permissions");
    }
}

/// Generate and store the key if it doesn't exist
pub fn load_key() -> Vec<u8> {
    let key_file = key_path();
    if key_file.exists() {
        fs::read(&key_file).expect("Failed to read encryption key")
    } else {
        panic!("Encryption key not found. Use --generate-key to create one.")
    }
}

/// Encrypts JSON tasks with a locally stored key
pub fn encrypt_tasks(tasks: &[Task]) -> Vec<u8> {
    let key = load_key();
    let cipher = Aes256Gcm::new_from_slice(&key).expect("Invalid key length");

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let plaintext = serde_json::to_vec(tasks).expect("Failed to serialize tasks");
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .expect("Encryption failed");

    [nonce_bytes.to_vec(), ciphertext].concat()
}

/// Decrypts JSON tasks with a locally stored key
pub fn decrypt_tasks(encrypted_data: &[u8]) -> Vec<Task> {
    let key = load_key();
    let cipher = Aes256Gcm::new_from_slice(&key).expect("Invalid key length");

    let (nonce, ciphertext) = encrypted_data.split_at(12);
    let plaintext = cipher
        .decrypt(Nonce::from_slice(nonce), ciphertext)
        .expect("Decryption failed");

    serde_json::from_slice(&plaintext).unwrap_or_default()
}
