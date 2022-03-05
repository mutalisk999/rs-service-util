use etcd_rs::{Client, ClientConfig};
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::{ClientTlsConfig};
use std::error::Error;

#[derive(Clone)]
pub struct RegSvcClient {
    client: Arc<Mutex<Client>>,
}

impl RegSvcClient{
    pub async fn new(endpoints: &Vec<String>, tls_config: &Option<ClientTlsConfig>,
               auth_config: &Option<(String, String)>) -> Result<RegSvcClient, Box<dyn Error>> {
        let cli_config = ClientConfig {
            endpoints: endpoints.clone(),
            auth: auth_config.clone(),
            tls: tls_config.clone(),
        };

        let client = Client::connect(cli_config).await?;
        Ok(RegSvcClient {
            client: Arc::new(Mutex::new(client)),
        })
    }

    pub fn get_instance_handle(self: &mut Self) -> Arc<Mutex<Client>> {
        self.client.clone()
    }

    pub async fn dispose_reg_svc_client(self: &mut Self) {
        self.client.lock().await.shutdown().await;
    }
}
