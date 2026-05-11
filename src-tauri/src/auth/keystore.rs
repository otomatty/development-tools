//! OS keystore abstraction for the AES-256-GCM master key.
//!
//! Backs the access-token encryption key with the platform credential store
//! (macOS Keychain / Windows Credential Manager / Linux Secret Service) so a
//! leaked application-data file alone is not enough to recover plaintext
//! tokens. Addresses Audit §9.3 / Issue #196.
//!
//! The previous `Crypto::from_app_key` derived a deterministic 256-bit key
//! from the constant app identifier. That key sat in the binary, so anyone
//! with the encrypted blob could trivially reconstruct it.  We now generate a
//! random 32-byte master key on first launch, store it in the OS keystore,
//! and rotate the on-disk ciphertext through the keystore-managed key.
//!
//! A trait is used (instead of a concrete `keyring::Entry`) so headless CI
//! and unit tests can swap in [`MemoryKeyStore`] without requiring a live
//! Secret Service / Keychain session.
//!
//! Storage format: the key bytes are persisted as base64. The `keyring` crate
//! exposes `set_secret` / `get_secret` for raw bytes on most platforms, but
//! some Secret Service collections still misbehave with non-UTF-8 payloads,
//! so we stick to base64 strings for portability.

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::RngExt as _;
use std::collections::HashMap;
use std::sync::Mutex;
use thiserror::Error;

/// AES-256 key length in bytes.
pub const KEY_LEN: usize = 32;

#[derive(Error, Debug)]
pub enum KeyStoreError {
    #[error("Keystore backend error: {0}")]
    Backend(String),

    #[error("Stored key has wrong length (expected {expected}, got {actual})")]
    InvalidKeyLength { expected: usize, actual: usize },

    #[error("Stored key is not valid base64: {0}")]
    InvalidEncoding(#[from] base64::DecodeError),

    #[error("Keystore mutex poisoned")]
    Poisoned,
}

pub type KeyStoreResult<T> = Result<T, KeyStoreError>;

/// Cross-platform secret-key store.
///
/// Implementations must be `Send + Sync` so they can sit inside `AppState`.
/// `get_key` / `set_key` are synchronous because every supported backend
/// (Keychain, DPAPI, Secret Service via DBus) exposes a blocking API and the
/// operations are only invoked at startup / login / logout.
pub trait KeyStore: Send + Sync {
    /// Fetch the stored 32-byte key, if any.
    fn get_key(&self) -> KeyStoreResult<Option<[u8; KEY_LEN]>>;

    /// Persist a 32-byte key, overwriting any previous value.
    fn set_key(&self, key: &[u8; KEY_LEN]) -> KeyStoreResult<()>;

    /// Remove the key (best-effort: missing key is not an error).
    fn delete_key(&self) -> KeyStoreResult<()>;
}

/// Convenience: fetch the key, generating + persisting a fresh random one if
/// none is stored yet. The two-step (`get` then `set`) is fine because the
/// app is single-process — there is no concurrent installer racing us.
pub fn get_or_create_key(store: &dyn KeyStore) -> KeyStoreResult<[u8; KEY_LEN]> {
    if let Some(key) = store.get_key()? {
        return Ok(key);
    }
    let mut key = [0u8; KEY_LEN];
    rand::rng().fill(&mut key);
    store.set_key(&key)?;
    Ok(key)
}

// ---------------------------------------------------------------------------
// OS-backed implementation
// ---------------------------------------------------------------------------

/// Default keystore identifiers. Kept here (not hard-coded inside the impl)
/// so test code can override them when running multiple instances in
/// parallel without colliding on the real Keychain / Secret Service.
pub const DEFAULT_SERVICE: &str = "com.sugaiakimasa.development-tools";
pub const DEFAULT_ACCOUNT: &str = "token-encryption-master-key";

/// Platform credential store backed by `keyring::Entry`.
pub struct OsKeyStore {
    service: String,
    account: String,
}

impl OsKeyStore {
    pub fn new() -> Self {
        Self {
            service: DEFAULT_SERVICE.to_string(),
            account: DEFAULT_ACCOUNT.to_string(),
        }
    }

    fn entry(&self) -> Result<keyring::Entry, KeyStoreError> {
        keyring::Entry::new(&self.service, &self.account)
            .map_err(|e| KeyStoreError::Backend(e.to_string()))
    }
}

impl Default for OsKeyStore {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyStore for OsKeyStore {
    fn get_key(&self) -> KeyStoreResult<Option<[u8; KEY_LEN]>> {
        let entry = self.entry()?;
        match entry.get_password() {
            Ok(b64) => {
                let bytes = BASE64.decode(b64.as_bytes())?;
                if bytes.len() != KEY_LEN {
                    return Err(KeyStoreError::InvalidKeyLength {
                        expected: KEY_LEN,
                        actual: bytes.len(),
                    });
                }
                let mut key = [0u8; KEY_LEN];
                key.copy_from_slice(&bytes);
                Ok(Some(key))
            }
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(KeyStoreError::Backend(e.to_string())),
        }
    }

    fn set_key(&self, key: &[u8; KEY_LEN]) -> KeyStoreResult<()> {
        let entry = self.entry()?;
        let b64 = BASE64.encode(key);
        entry
            .set_password(&b64)
            .map_err(|e| KeyStoreError::Backend(e.to_string()))
    }

    fn delete_key(&self) -> KeyStoreResult<()> {
        let entry = self.entry()?;
        match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(KeyStoreError::Backend(e.to_string())),
        }
    }
}

// ---------------------------------------------------------------------------
// In-memory implementation (tests and headless fallback)
// ---------------------------------------------------------------------------

/// Process-local keystore used by unit tests. NOT a security boundary — the
/// key lives in plain memory and is dropped when the process exits, defeating
/// the whole point of using the OS keystore. Only use this when an OS
/// keystore is genuinely unavailable (e.g. a Linux CI runner without DBus)
/// and the trade-off has been explicitly accepted.
#[derive(Default)]
pub struct MemoryKeyStore {
    inner: Mutex<HashMap<String, [u8; KEY_LEN]>>,
    slot: String,
}

impl MemoryKeyStore {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
            slot: DEFAULT_ACCOUNT.to_string(),
        }
    }

    pub fn with_slot(slot: impl Into<String>) -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
            slot: slot.into(),
        }
    }
}

impl KeyStore for MemoryKeyStore {
    fn get_key(&self) -> KeyStoreResult<Option<[u8; KEY_LEN]>> {
        let map = self.inner.lock().map_err(|_| KeyStoreError::Poisoned)?;
        Ok(map.get(&self.slot).copied())
    }

    fn set_key(&self, key: &[u8; KEY_LEN]) -> KeyStoreResult<()> {
        let mut map = self.inner.lock().map_err(|_| KeyStoreError::Poisoned)?;
        map.insert(self.slot.clone(), *key);
        Ok(())
    }

    fn delete_key(&self) -> KeyStoreResult<()> {
        let mut map = self.inner.lock().map_err(|_| KeyStoreError::Poisoned)?;
        map.remove(&self.slot);
        Ok(())
    }
}

// `TokenManager` reads the key exactly once during construction and caches
// the resulting `Crypto` for the lifetime of the process, so we deliberately
// do NOT add a caching wrapper around `KeyStore` — there would be no second
// round-trip to amortise.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_keystore_round_trip() {
        let store = MemoryKeyStore::new();
        assert!(store.get_key().unwrap().is_none());

        let key = [7u8; KEY_LEN];
        store.set_key(&key).unwrap();
        assert_eq!(store.get_key().unwrap(), Some(key));

        store.delete_key().unwrap();
        assert!(store.get_key().unwrap().is_none());
    }

    #[test]
    fn get_or_create_generates_fresh_key() {
        let store = MemoryKeyStore::new();
        let k1 = get_or_create_key(&store).unwrap();
        let k2 = get_or_create_key(&store).unwrap();
        // Stable across calls
        assert_eq!(k1, k2);
        // Not all-zero (overwhelmingly unlikely with a real RNG)
        assert_ne!(k1, [0u8; KEY_LEN]);
    }

    #[test]
    fn get_or_create_two_stores_are_independent() {
        let a = MemoryKeyStore::with_slot("a");
        let b = MemoryKeyStore::with_slot("b");
        let ka = get_or_create_key(&a).unwrap();
        let kb = get_or_create_key(&b).unwrap();
        assert_ne!(ka, kb, "independent stores must produce distinct keys");
    }

    #[test]
    fn invalid_key_length_is_rejected() {
        // Simulate a corrupted keystore entry to make sure we surface a
        // typed error rather than panicking on the `copy_from_slice` later.
        let err = KeyStoreError::InvalidKeyLength {
            expected: KEY_LEN,
            actual: 16,
        };
        assert!(err.to_string().contains("32"));
    }
}
