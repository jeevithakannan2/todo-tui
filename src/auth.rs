use aes_gcm::{Aes256Gcm, KeyInit, Nonce, aead::Aead};
use argon2::Argon2;
use keyring::Entry;
use rand::RngCore;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    widgets::Widget,
};
use tui_textarea::TextArea;

use crate::handle_json::Task;
use crate::helpers::PopupSize;
use crate::{
    app::{PRIMARY_STYLE, SECONDARY_STYLE},
    helpers::create_popup_area,
};

/// Service name used in the OS key store.
const PASSWORD_SERVICE: &str = "todotui";
/// Salt length in bytes.
const SALT_LEN: usize = 16;

/// Retrieves the current user name using the `whoami` crate.
fn current_username() -> String {
    whoami::username()
}

/// Store the password using the current user as the account.
pub fn store_password(new_password: &str) {
    let account = current_username();
    let keyring = Entry::new(PASSWORD_SERVICE, &account).unwrap();
    keyring
        .set_password(&new_password)
        .expect("Failed to store password");
}

/// Retrieves the password from storage (or prompts if missing).
pub fn get_password() -> String {
    let account = current_username();
    let keyring = Entry::new(PASSWORD_SERVICE, &account).unwrap();
    if let Ok(stored) = keyring.get_password() {
        stored
    } else {
        String::new()
    }
}

/// Derives a 32-byte key from the given password using Argon2 and the provided salt.
fn derive_key_from_password(password: &str, salt: &[u8]) -> [u8; 32] {
    let argon2 = Argon2::default();
    let mut key = [0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .expect("Failed to derive key");
    key
}

/// Encrypts the tasks list using a unique salt.  
/// Output format: [salt (16 bytes)] || [nonce (12 bytes)] || [ciphertext].
pub fn encrypt_tasks(tasks: &[Task], password: &str) -> Vec<u8> {
    // Generate a unique salt.
    let mut salt = [0u8; SALT_LEN];
    rand::thread_rng().fill_bytes(&mut salt);

    // Derive a key from the password and unique salt.
    let key = derive_key_from_password(password, &salt);
    let cipher = Aes256Gcm::new_from_slice(&key).expect("Invalid key length");

    // Generate a random nonce.
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Serialize tasks to JSON.
    let plaintext = serde_json::to_vec(tasks).expect("Failed to serialize tasks");
    // Encrypt the plaintext.
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .expect("Encryption failed");

    [salt.to_vec(), nonce_bytes.to_vec(), ciphertext].concat()
}

/// Decrypts the encrypted data to retrieve the tasks list using the salt stored in the data.
pub fn decrypt_tasks(encrypted_data: &[u8], password: &str) -> Vec<Task> {
    // Extract salt.
    let (salt, rest) = encrypted_data.split_at(SALT_LEN);
    // Extract nonce.
    let (nonce, ciphertext) = rest.split_at(12);
    // Derive the key from the password and extracted salt.
    let key = derive_key_from_password(password, salt);
    let cipher = Aes256Gcm::new_from_slice(&key).expect("Invalid key length");

    // Decrypt the ciphertext.
    let plaintext = cipher.decrypt(Nonce::from_slice(nonce), ciphertext).expect("Decryption failed! Password maybe incorrect. If you forgot your password, you cannot recover your tasks. Please delete the data file by running with the --reset flag.");

    match serde_json::from_slice(&plaintext) {
        Ok(tasks) => tasks,
        Err(_) => Vec::new(),
    }
}

/// Deletes the stored password from the OS key store.
pub fn delete_stored_password() {
    let account = current_username();
    let keyring = Entry::new(PASSWORD_SERVICE, &account).unwrap();
    let _ = keyring.delete_credential();
}

pub struct PasswordPrompt<'a> {
    password: TextArea<'a>,
    confirm_password: TextArea<'a>,
    focus: Focus,
}

#[derive(PartialEq)]
enum Focus {
    Password,
    ConfirmPassword,
}

impl Widget for &mut PasswordPrompt<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let area = create_popup_area(area, &PopupSize::FixedHeight { x: 100, height: 6 });
        let [area1, area2] =
            Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(area);
        self.render_password(area1, buf);
        self.render_confirm_password(area2, buf);
    }
}

impl PasswordPrompt<'_> {
    pub fn new() -> Self {
        Self {
            password: TextArea::default(),
            confirm_password: TextArea::default(),
            focus: Focus::Password,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            return true;
        }
        match self.focus {
            Focus::Password => {
                if key.code == KeyCode::Tab {
                    self.focus = Focus::ConfirmPassword;
                    return false;
                }
                if key.code == KeyCode::Enter {
                    if self.check_passwords_match() {
                        let password = &self.password.lines()[0];
                        store_password(password);
                        return true;
                    }
                    return false;
                }
                self.password.input(key);
                false
            }
            Focus::ConfirmPassword => {
                if key.code == KeyCode::Tab {
                    self.focus = Focus::Password;
                    return false;
                }
                if key.code == KeyCode::Enter {
                    if self.check_passwords_match() {
                        let password = &self.confirm_password.lines()[0];
                        store_password(password);
                        return true;
                    }
                    return false;
                }
                self.confirm_password.input(key);
                false
            }
        }
    }

    fn get_style(&self, focus: Focus) -> (Style, Style) {
        if focus == self.focus {
            (PRIMARY_STYLE, Style::default().reversed())
        } else {
            (SECONDARY_STYLE, Style::default())
        }
    }

    fn check_passwords_match(&self) -> bool {
        self.password.lines()[0] == self.confirm_password.lines()[0]
    }

    fn render_password(&mut self, area: Rect, buf: &mut Buffer) {
        let style = self.get_style(Focus::Password);
        let popup_area =
            crate::helpers::create_popup_area(area, &PopupSize::FixedHeight { x: 100, height: 3 });
        let block = crate::helpers::rounded_block(" Password ", style.0);
        self.password.set_cursor_style(style.1);
        self.password.set_cursor_line_style(Style::default());
        self.password.set_block(block);
        self.password.set_mask_char('*');
        self.password.set_placeholder_text("Enter your password");
        self.password.render(popup_area, buf);
    }

    fn render_confirm_password(&mut self, area: Rect, buf: &mut Buffer) {
        let style = self.get_style(Focus::ConfirmPassword);
        let popup_area =
            crate::helpers::create_popup_area(area, &PopupSize::FixedHeight { x: 100, height: 3 });
        let title = if !self.check_passwords_match() {
            " Confirm password - Passwords do not match "
        } else {
            " Confirm password "
        };
        let block = crate::helpers::rounded_block(title, style.0);
        self.confirm_password.set_cursor_style(style.1);
        self.confirm_password
            .set_cursor_line_style(Style::default());
        self.confirm_password.set_block(block);
        self.confirm_password.set_mask_char('*');
        self.confirm_password
            .set_placeholder_text("Confirm your password");
        self.confirm_password.render(popup_area, buf);
    }
}
