use security_framework::os::macos::keychain::SecKeychain;
use security_framework::os::macos::passwords::find_generic_password;

use crate::{Error as KeyError, KeyringError, Platform, PlatformIdentity, Result};

pub fn platform() -> Platform {
    Platform::MacOs
}

pub use security_framework::base::Error;

fn get_keychain() -> Result<SecKeychain> {
    match SecKeychain::default() {
        Ok(keychain) => Ok(keychain),
        Err(err) => Err(KeyError::new_from_platform(KeyringError::NoStorage, err)),
    }
}

pub fn set_password(map: &PlatformIdentity, password: &str) -> Result<()> {
    if let PlatformIdentity::Mac(map) = map {
        get_keychain()?
            .set_generic_password(&map.service, &map.account, password.as_bytes())
            .map_err(|err| KeyError::new_from_platform(KeyringError::PlatformFailure, err))?;
        Ok(())
    } else {
        Err(KeyringError::BadIdentityMapPlatform.into())
    }
}

pub fn get_password(map: &PlatformIdentity) -> Result<String> {
    if let PlatformIdentity::Mac(map) = map {
        let (password_bytes, _) =
            find_generic_password(Some(&[get_keychain()?]), &map.service, &map.account)
                .map_err(|err| KeyError::new_from_platform(KeyringError::NoEntry, err))?;
        // Mac keychain allows non-UTF8 values, but this library only supports adding UTF8 items
        // to the keychain, so this should only fail if we are trying to retrieve a non-UTF8
        // password that was added to the keychain by another library
        let password = String::from_utf8(password_bytes.to_vec())
            .map_err(|err| KeyError::new_from_encoding(err.utf8_error()))?;
        Ok(password)
    } else {
        Err(KeyringError::BadIdentityMapPlatform.into())
    }
}

pub fn delete_password(map: &PlatformIdentity) -> Result<()> {
    if let PlatformIdentity::Mac(map) = map {
        let (_, item) = find_generic_password(Some(&[get_keychain()?]), &map.service, &map.account)
            .map_err(|err| KeyError::new_from_platform(KeyringError::NoEntry, err))?;
        item.delete();
        Ok(())
    } else {
        Err(KeyringError::BadIdentityMapPlatform.into())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::attrs::default_identity_mapper;
    use crate::Platform;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_basic() {
        let password_1 = "大根";
        let password_2 = "0xE5A4A7E6A0B9"; // Above in hex string
        let map = default_identity_mapper(Platform::MacOs, "test-service", "test-user");

        set_password(&map, password_1).unwrap();
        let response_1 = get_password(&map).unwrap();
        assert_eq!(
            response_1, password_1,
            "Stored and retrieved passwords don't match"
        );

        set_password(&map, password_2).unwrap();
        let response_2 = get_password(&map).unwrap();
        assert_eq!(
            response_2, password_2,
            "Stored and retrieved passwords don't match"
        );

        delete_password(&map).unwrap();
        assert!(
            get_password(&map).is_err(),
            "Able to read a deleted password"
        )
    }
}
