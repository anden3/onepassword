#[derive(Debug, serde::Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ClientConfig {
    pub service_account_token: String,
    pub integration_name: &'static str,
    pub integration_version: &'static str,
    pub sdk_version: &'static str,
    pub request_library_name: &'static str,
    pub request_library_version: &'static str,
    pub os: &'static str,
    pub os_version: &'static str,
    pub architecture: &'static str,
    pub programming_language: &'static str,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            service_account_token: Default::default(),
            integration_name: option_env!("CARGO_PKG_NAME").unwrap_or_default(),
            integration_version: option_env!("CARGO_PKG_VERSION").unwrap_or_default(),
            sdk_version: "0030101",
            os: "windows",
            os_version: "0.0.0",
            architecture: "x86_64",
            programming_language: "Rust",
            request_library_name: "reqwest",
            request_library_version: "0.11.24",
        }
    }
}

#[derive(Debug, serde::Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct InvocationWrapper {
    pub invocation: Invocation,
}

#[derive(Debug, serde::Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Invocation {
    pub client_id: u64,
    pub parameters: InvocationParameters,
}

#[derive(Debug, serde::Serialize)]
#[serde(tag = "name", content = "parameters")]
pub enum InvocationParameters {
    VaultsList {
        _marker: (),
    },
    ItemsList {
        vault_id: String,
        filters: Vec<String>,
    },
    SecretsResolve {
        secret_reference: String,
    },
}

#[derive(Debug, serde::Deserialize)]
pub struct Vault {
    pub id: String,
    pub title: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct Item {
    pub id: String,
    pub title: String,
    pub category: String,
    pub websites: Vec<Website>,
}

#[derive(Debug, serde::Deserialize)]
pub struct Website {
    pub url: String,
}
