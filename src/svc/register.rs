use chrono::prelude::*;
use etcd_rs::{Client, ClientConfig, LeaseGrantRequest, LeaseKeepAliveRequest, PutRequest};
use log::{debug, warn};
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::Duration;
use tokio_stream::StreamExt;
use tonic::transport::ClientTlsConfig;

pub struct RegSvcClient {
    client: Arc<Mutex<Client>>,
    status: Arc<Mutex<u8>>,
}

impl RegSvcClient {
    pub async fn new(
        endpoints: Vec<String>,
        tls_config: Option<ClientTlsConfig>,
        auth_config: Option<(String, String)>,
    ) -> Result<RegSvcClient, Box<dyn Error>> {
        let cli_config = ClientConfig {
            endpoints: endpoints.clone(),
            auth: auth_config.clone(),
            tls: tls_config.clone(),
        };

        let client = Client::connect(cli_config).await?;
        Ok(RegSvcClient {
            client: Arc::new(Mutex::new(client)),
            status: Arc::new(Mutex::new(1)),
        })
    }

    pub async fn get_instance_handle(self: &mut Self) -> Option<Arc<Mutex<Client>>> {
        match self.status.lock().await.clone() {
            1 => Some(self.client.clone()),
            _ => None,
        }
    }

    pub async fn dispose_reg_svc_client(self: &mut Self) -> Result<(), Box<dyn Error>> {
        self.client.lock().await.shutdown().await?;
        *(self.status.lock().await) = 0 as u8;
        Ok(())
    }

    pub async fn register_service(
        self: &mut Self,
        keep_alive_sec: u64,
        service_ttl_sec: u64,
        key_prefix: String,
        svc_id: String,
        svc_name: String,
        svc_addr: String,
    ) -> Result<(), Box<dyn Error>> {
        let key_prefix = if key_prefix == "" {
            "/etcd_services".to_owned()
        } else {
            key_prefix.clone()
        };
        let service_ttl_sec = if service_ttl_sec == 0 {
            30 as u64
        } else {
            service_ttl_sec
        };
        let keep_alive_sec = if keep_alive_sec == 0 {
            5 as u64
        } else {
            keep_alive_sec
        };

        {
            let mut inbound = self
                .client
                .lock()
                .await
                .lease()
                .keep_alive_responses()
                .await
                .unwrap();
            let status = self.status.clone();
            // watch keep alive event
            tokio::spawn(async move {
                loop {
                    if *(status.lock().await) == 0 {
                        warn!(
                            "[Cancel] keep alive response at {:?}",
                            Utc::now().to_string()
                        );
                        break;
                    }
                    match inbound.next().await {
                        Some(resp) => {
                            debug!(
                                "keep alive response: {:?} at {:?}",
                                resp,
                                Utc::now().to_string()
                            );
                        }
                        None => {
                            warn!(
                                "[Cancel] keep alive response at {:?}",
                                Utc::now().to_string()
                            );
                            break;
                        }
                    }
                }
            });
        }

        let resp_lease_grant = self
            .client
            .lock()
            .await
            .lease()
            .grant(LeaseGrantRequest::new(Duration::from_secs(service_ttl_sec)))
            .await?;

        let key = [key_prefix.clone(), svc_id.clone(), svc_name.clone()].join("/");
        let mut req_put = PutRequest::new(key, svc_addr.clone());
        req_put.set_lease(resp_lease_grant.id());
        self.client.lock().await.kv().put(req_put).await?;

        {
            // keep alive the lease
            let arc_client = self.client.clone();
            let status = self.status.clone();
            tokio::spawn(async move {
                loop {
                    if *(status.lock().await) == 0 {
                        warn!(
                            "[Cancel] keep alive request at {:?}",
                            Utc::now().to_string()
                        );
                        break;
                    }
                    let keep_alive_res = arc_client
                        .lock()
                        .await
                        .lease()
                        .keep_alive(LeaseKeepAliveRequest::new(resp_lease_grant.id()))
                        .await;
                    match keep_alive_res {
                        Ok(_) => {
                            debug!("keep alive request at {:?}", Utc::now().to_string());
                            tokio::time::sleep(Duration::from_secs(keep_alive_sec)).await;
                        }
                        Err(_) => {
                            warn!(
                                "[Cancel] keep alive request at {:?}",
                                Utc::now().to_string()
                            );
                            break;
                        }
                    }
                }
            });
        }

        Ok(())
    }
}
