Safe, natively async wrapper around `onepassword-sys`.

## Setup
The dynamic libraries required can be downloaded from https://github.com/1Password/onepassword-sdk-python/tree/main/src/onepassword/lib

## Example usage
```rs
const ONEPASS_SERVICE_ACCOUNT_TOKEN: &str =
    "https://developer.1password.com/docs/service-accounts/get-started";

async fn get_vaults() -> Result<Vec<VaultWrapper>, FfiError> {
    let client = Client::new(ClientConfig {
        service_account_token: ONEPASS_SERVICE_ACCOUNT_TOKEN.to_owned(),
        integration_name: env!("CARGO_PKG_NAME"),
        integration_version: env!("CARGO_PKG_VERSION"),
        ..Default::default()
    }).await?;

    client.vaults().await
}
```