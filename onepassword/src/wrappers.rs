use crate::invoke;
use onepassword_shared::types::{ClientConfig, Invocation, InvocationParameters, Item, Vault};
use onepassword_sys::Error as FfiError;
use secrecy::SecretString;
use std::{ops::Deref, sync::Arc};

type FfiResult<T> = Result<T, FfiError>;

#[derive(Clone)]
pub struct Client(Arc<ClientInner>);

impl Deref for Client {
    type Target = ClientInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
pub struct ClientInner {
    pub(crate) id: u64,
}

impl Drop for ClientInner {
    fn drop(&mut self) {
        let id_str = self.id.to_string();
        onepassword_sys::free_client(&id_str);
    }
}

impl Client {
    pub fn new(config: ClientConfig) -> FfiResult<Client> {
        let client = Arc::new(ClientInner {
            id: pollster::block_on(Self::get_client_id(config))?,
        });

        Ok(Client(client))
    }

    async fn get_client_id(config: ClientConfig) -> FfiResult<u64> {
        onepassword_sys::validate_checksums();

        let serialized_config = serde_json::to_string(&config).unwrap();
        let id_buffer = onepassword_sys::get_client_id_buffer_sync(&serialized_config)?;

        Ok(id_buffer.to_string().parse().unwrap())
    }
}

impl Client {
    pub fn vaults(&self) -> FfiResult<Vec<VaultWrapper>> {
        let vaults: Vec<Vault> = invoke(Invocation {
            client_id: self.id,
            parameters: InvocationParameters::VaultsList { _marker: () },
        })?;

        let wrapped_vaults = vaults
            .into_iter()
            .map(|vault| VaultWrapper {
                vault,
                client: self.clone(),
            })
            .collect();

        Ok(wrapped_vaults)
    }

    pub fn get_vault_by_title(&self, title: &str) -> FfiResult<Option<VaultWrapper>> {
        let vault = self.vaults()?.into_iter().find(|v| v.title == title);
        Ok(vault)
    }
}

pub struct VaultWrapper {
    pub vault: Vault,
    client: Client,
}

impl std::fmt::Debug for VaultWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VaultWrapper")
            .field("vault", &self.vault)
            .finish()
    }
}

impl Deref for VaultWrapper {
    type Target = Vault;

    fn deref(&self) -> &Self::Target {
        &self.vault
    }
}

impl VaultWrapper {
    pub fn items(&self) -> FfiResult<Vec<ItemWrapper>> {
        let items = invoke::<Vec<Item>>(Invocation {
            client_id: self.client.id,
            parameters: InvocationParameters::ItemsList {
                vault_id: self.vault.id.clone(),
                filters: vec![],
            },
        })?;

        let items = items
            .into_iter()
            .map(|item| ItemWrapper {
                item,
                client: self.client.clone(),
                vault_id: self.vault.id.clone(),
            })
            .collect();

        Ok(items)
    }

    pub fn items_for_website(&self, website: &str) -> FfiResult<Vec<ItemWrapper>> {
        let trim_protocol = !website.contains("://");
        let items = self
            .items()?
            .into_iter()
            .filter(|it| {
                it.websites.iter().any(|w| {
                    if trim_protocol {
                        w.url
                            .split_once("://")
                            .is_some_and(|(_, url)| website.starts_with(url))
                    } else {
                        website.starts_with(&w.url)
                    }
                })
            })
            .collect();

        Ok(items)
    }
}

pub struct ItemWrapper {
    pub item: Item,
    client: Client,
    vault_id: String,
}

impl std::fmt::Debug for ItemWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ItemWrapper")
            .field("item", &self.item)
            .field("vault_id", &self.vault_id)
            .finish()
    }
}

impl Deref for ItemWrapper {
    type Target = Item;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl ItemWrapper {
    fn construct_secret_ref(&self, field: &str) -> String {
        format!("op://{}/{}/{field}", self.vault_id, self.item.id)
    }
}

impl ItemWrapper {
    pub fn password(&self) -> FfiResult<Option<SecretString>> {
        let secret_reference = self.construct_secret_ref("password");

        let result = invoke::<SecretString>(Invocation {
            client_id: self.client.id,
            parameters: InvocationParameters::SecretsResolve { secret_reference },
        });

        match result {
            Ok(secret) => Ok(Some(secret)),
            Err(e) if e.code() == 133 => Ok(None),
            Err(e) => Err(e),
        }
    }
}
