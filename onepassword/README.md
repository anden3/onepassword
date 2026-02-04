Safe, (almost entirely[1]) natively sync wrapper around `onepassword-sys`.

## Setup
The dynamic libraries required can be downloaded from https://github.com/1Password/onepassword-sdk-python/tree/main/src/onepassword/lib

## Example usage
```rs
const ONEPASS_SERVICE_ACCOUNT_TOKEN: &str =
    "https://developer.1password.com/docs/service-accounts/get-started";

fn get_vaults() -> Result<Vec<VaultWrapper>, FfiError> {
    Client::new(ClientConfig {
        service_account_token: ONEPASS_SERVICE_ACCOUNT_TOKEN.to_owned(),
        integration_name: env!("CARGO_PKG_NAME"),
        integration_version: env!("CARGO_PKG_VERSION"),
        ..Default::default()
    })?
    .vaults()
}
```

[1]: `pollster` is used in `onepassword-sys` because getting a client ID requires polling a future no matter what, but since it's our own future we know `pollster` works fine.