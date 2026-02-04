use onepassword_shared::types::{Invocation, InvocationWrapper};

pub mod wrappers;

pub use onepassword_shared::types::ClientConfig;
pub use onepassword_sys::{Error as FfiError, version};
pub use wrappers::Client;

pub async fn invoke<T: serde::de::DeserializeOwned>(invocation: Invocation) -> Result<T, FfiError> {
    let serialized_config = serde_json::to_string(&InvocationWrapper { invocation }).unwrap();
    let result = onepassword_sys::invoke(&serialized_config).await?;
    let value = serde_json::from_reader(result.as_ref()).unwrap();
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ONEPASS_SERVICE_ACCOUNT_TOKEN: &str =
        "https://developer.1password.com/docs/service-accounts/get-started";

    #[tokio::test]
    async fn connect() {
        let vaults = Client::new(ClientConfig {
            service_account_token: ONEPASS_SERVICE_ACCOUNT_TOKEN.to_owned(),
            integration_name: env!("CARGO_PKG_NAME"),
            integration_version: env!("CARGO_PKG_VERSION"),
            ..Default::default()
        })
        .await
        .unwrap()
        .vaults()
        .await
        .unwrap();

        eprintln!("{vaults:?}");
    }
}
