use std::{fs, path::PathBuf};

use crate::config::SecretStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretKey {
    AiProviderApiKey,
}

impl SecretKey {
    fn file_name(self) -> &'static str {
        match self {
            Self::AiProviderApiKey => "ai-provider-api-key.dpapi",
        }
    }
}

pub trait SecretStore {
    fn put(&self, key: SecretKey, value: &str) -> Result<(), String>;
    fn get(&self, key: SecretKey) -> Result<Option<String>, String>;
    fn delete(&self, key: SecretKey) -> Result<(), String>;
    fn status(&self, key: SecretKey) -> Result<SecretStatus, String>;
}

#[derive(Debug, Clone)]
pub struct DpapiSecretStore {
    secret_dir: PathBuf,
}

impl DpapiSecretStore {
    pub fn new(secret_dir: PathBuf) -> Self {
        Self { secret_dir }
    }

    pub fn secret_path(&self, key: SecretKey) -> PathBuf {
        self.secret_dir.join(key.file_name())
    }
}

impl SecretStore for DpapiSecretStore {
    fn put(&self, key: SecretKey, value: &str) -> Result<(), String> {
        if value.trim().is_empty() {
            return Err("Secret value cannot be empty.".into());
        }
        fs::create_dir_all(&self.secret_dir)
            .map_err(|error| format!("Unable to create secret directory: {error}"))?;
        let protected = protect_secret(value.as_bytes())?;
        fs::write(self.secret_path(key), protected)
            .map_err(|error| format!("Unable to write secret file: {error}"))
    }

    fn get(&self, key: SecretKey) -> Result<Option<String>, String> {
        let path = self.secret_path(key);
        if !path.exists() {
            return Ok(None);
        }
        let protected =
            fs::read(path).map_err(|error| format!("Unable to read secret file: {error}"))?;
        let bytes = unprotect_secret(&protected)?;
        let value = String::from_utf8(bytes)
            .map_err(|error| format!("Secret file did not contain UTF-8 data: {error}"))?;
        Ok(Some(value))
    }

    fn delete(&self, key: SecretKey) -> Result<(), String> {
        let path = self.secret_path(key);
        if path.exists() {
            fs::remove_file(path)
                .map_err(|error| format!("Unable to delete secret file: {error}"))?;
        }
        Ok(())
    }

    fn status(&self, key: SecretKey) -> Result<SecretStatus, String> {
        let Some(value) = self.get(key)? else {
            return Ok(SecretStatus {
                present: false,
                masked: None,
            });
        };
        Ok(SecretStatus {
            present: true,
            masked: Some(mask_secret(&value)),
        })
    }
}

fn mask_secret(value: &str) -> String {
    let suffix: String = value
        .chars()
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("••••{suffix}")
}

#[cfg(windows)]
fn protect_secret(bytes: &[u8]) -> Result<Vec<u8>, String> {
    use std::{io, ptr, slice};

    use windows_sys::Win32::{
        Foundation::LocalFree,
        Security::Cryptography::{CRYPT_INTEGER_BLOB, CRYPTPROTECT_UI_FORBIDDEN, CryptProtectData},
    };

    let mut input = CRYPT_INTEGER_BLOB {
        cbData: bytes.len() as u32,
        pbData: bytes.as_ptr() as *mut u8,
    };
    let mut output = CRYPT_INTEGER_BLOB::default();
    let ok = unsafe {
        CryptProtectData(
            &mut input,
            ptr::null(),
            ptr::null(),
            ptr::null(),
            ptr::null(),
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output,
        )
    };
    if ok == 0 {
        return Err(format!(
            "Windows DPAPI CryptProtectData failed: {}",
            io::Error::last_os_error()
        ));
    }

    let protected =
        unsafe { slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec() };
    unsafe {
        LocalFree(output.pbData.cast());
    }
    Ok(protected)
}

#[cfg(windows)]
fn unprotect_secret(bytes: &[u8]) -> Result<Vec<u8>, String> {
    use std::{io, ptr, slice};

    use windows_sys::Win32::{
        Foundation::LocalFree,
        Security::Cryptography::{
            CRYPT_INTEGER_BLOB, CRYPTPROTECT_UI_FORBIDDEN, CryptUnprotectData,
        },
    };

    let mut input = CRYPT_INTEGER_BLOB {
        cbData: bytes.len() as u32,
        pbData: bytes.as_ptr() as *mut u8,
    };
    let mut output = CRYPT_INTEGER_BLOB::default();
    let ok = unsafe {
        CryptUnprotectData(
            &mut input,
            ptr::null_mut(),
            ptr::null(),
            ptr::null(),
            ptr::null(),
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output,
        )
    };
    if ok == 0 {
        return Err(format!(
            "Windows DPAPI CryptUnprotectData failed: {}",
            io::Error::last_os_error()
        ));
    }

    let secret = unsafe { slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec() };
    unsafe {
        LocalFree(output.pbData.cast());
    }
    Ok(secret)
}

#[cfg(not(windows))]
fn protect_secret(_bytes: &[u8]) -> Result<Vec<u8>, String> {
    Err("DPAPI secret storage is only available on Windows.".into())
}

#[cfg(not(windows))]
fn unprotect_secret(_bytes: &[u8]) -> Result<Vec<u8>, String> {
    Err("DPAPI secret storage is only available on Windows.".into())
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf, time::SystemTime};

    use super::{DpapiSecretStore, SecretKey, SecretStore};

    fn temp_root(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("tsr-desktop-secret-{name}-{nanos}"))
    }

    #[test]
    fn dpapi_secret_store_masks_round_trips_and_deletes_api_keys() {
        let secret = "sk-live-secret-value";
        let store = DpapiSecretStore::new(temp_root("dpapi"));

        store.put(SecretKey::AiProviderApiKey, secret).unwrap();

        let status = store.status(SecretKey::AiProviderApiKey).unwrap();
        assert!(status.present);
        assert_eq!(status.masked.as_deref(), Some("••••alue"));
        assert_eq!(
            store.get(SecretKey::AiProviderApiKey).unwrap().as_deref(),
            Some(secret)
        );

        let encrypted = fs::read(store.secret_path(SecretKey::AiProviderApiKey)).unwrap();
        assert!(!String::from_utf8_lossy(&encrypted).contains(secret));

        store.delete(SecretKey::AiProviderApiKey).unwrap();
        assert!(!store.status(SecretKey::AiProviderApiKey).unwrap().present);
        assert!(store.get(SecretKey::AiProviderApiKey).unwrap().is_none());
    }
}
