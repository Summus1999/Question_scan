use crate::errors::AppResult;

#[cfg(all(not(test), target_os = "windows"))]
const API_KEY_ACCOUNT_NAME: &str = "provider-api-key";
#[cfg(all(not(test), target_os = "windows"))]
const API_KEY_SERVICE_NAME: &str = "Question Scan";

#[cfg(test)]
mod platform {
    use super::*;
    use std::cell::RefCell;

    thread_local! {
        static TEST_PROVIDER_API_KEY: RefCell<Option<String>> = const { RefCell::new(None) };
    }

    pub(crate) fn load_provider_api_key() -> AppResult<Option<String>> {
        Ok(TEST_PROVIDER_API_KEY.with(|slot| slot.borrow().clone()))
    }

    pub(crate) fn save_provider_api_key(api_key: &str) -> AppResult<()> {
        TEST_PROVIDER_API_KEY.with(|slot| {
            *slot.borrow_mut() = Some(api_key.to_string());
        });
        Ok(())
    }

    pub(crate) fn clear_provider_api_key() -> AppResult<()> {
        TEST_PROVIDER_API_KEY.with(|slot| {
            *slot.borrow_mut() = None;
        });
        Ok(())
    }
}

#[cfg(all(not(test), target_os = "windows"))]
mod platform {
    use super::*;
    use windows::core::HSTRING;
    use windows::Security::Credentials::{PasswordCredential, PasswordVault};

    pub(crate) fn load_provider_api_key() -> AppResult<Option<String>> {
        let vault = PasswordVault::new().map_err(map_windows_error)?;

        match vault.Retrieve(
            &HSTRING::from(API_KEY_SERVICE_NAME),
            &HSTRING::from(API_KEY_ACCOUNT_NAME),
        ) {
            Ok(credential) => {
                credential.RetrievePassword().map_err(map_windows_error)?;
                let password = credential.Password().map_err(map_windows_error)?;
                Ok(Some(password.to_string()))
            }
            Err(error) => {
                tracing::debug!(
                    %error,
                    "Could not load the provider API key from Windows Credential Manager"
                );
                Ok(None)
            }
        }
    }

    pub(crate) fn save_provider_api_key(api_key: &str) -> AppResult<()> {
        if api_key.trim().is_empty() {
            return clear_provider_api_key();
        }

        let vault = PasswordVault::new().map_err(map_windows_error)?;
        if let Ok(existing) = vault.Retrieve(
            &HSTRING::from(API_KEY_SERVICE_NAME),
            &HSTRING::from(API_KEY_ACCOUNT_NAME),
        ) {
            let _ = vault.Remove(&existing);
        }

        let credential = PasswordCredential::CreatePasswordCredential(
            &HSTRING::from(API_KEY_SERVICE_NAME),
            &HSTRING::from(API_KEY_ACCOUNT_NAME),
            &HSTRING::from(api_key),
        )
        .map_err(map_windows_error)?;

        vault.Add(&credential).map_err(map_windows_error)
    }

    pub(crate) fn clear_provider_api_key() -> AppResult<()> {
        let vault = PasswordVault::new().map_err(map_windows_error)?;
        if let Ok(existing) = vault.Retrieve(
            &HSTRING::from(API_KEY_SERVICE_NAME),
            &HSTRING::from(API_KEY_ACCOUNT_NAME),
        ) {
            let _ = vault.Remove(&existing);
        }
        Ok(())
    }
}

#[cfg(all(not(test), not(target_os = "windows")))]
mod platform {
    use super::*;
    use crate::errors::AppError;

    const UNSUPPORTED_STORAGE_MESSAGE: &str =
        "Secure API key storage is only available on Windows in the MVP.";

    pub(crate) fn load_provider_api_key() -> AppResult<Option<String>> {
        Ok(None)
    }

    pub(crate) fn save_provider_api_key(_api_key: &str) -> AppResult<()> {
        Err(AppError::ApiKeyStorageFailed {
            reason: UNSUPPORTED_STORAGE_MESSAGE.to_string(),
        })
    }

    pub(crate) fn clear_provider_api_key() -> AppResult<()> {
        Err(AppError::ApiKeyStorageFailed {
            reason: UNSUPPORTED_STORAGE_MESSAGE.to_string(),
        })
    }
}

#[cfg(all(not(test), target_os = "windows"))]
fn map_windows_error(error: windows::core::Error) -> AppError {
    AppError::ApiKeyStorageFailed {
        reason: error.to_string(),
    }
}

pub(crate) use platform::clear_provider_api_key;
pub(crate) use platform::load_provider_api_key;
pub(crate) use platform::save_provider_api_key;
