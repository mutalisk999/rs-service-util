use etcd_rs::{Client, ClientConfig, LeaseGrantRequest, PutRequest, LeaseKeepAliveRequest};
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::{ClientTlsConfig};
use std::error::Error;
use tokio::time::Duration;
use tokio_stream::StreamExt;
use log::{debug, warn};
use std::time::SystemTime;


pub struct RegSvcClient {
    client: Arc<Mutex<Client>>,
}

impl RegSvcClient {
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

    pub async fn dispose_reg_svc_client(self: &mut Self) -> Result<(), Box<dyn Error>> {
        self.client.lock().await.shutdown().await?;
        Ok(())
    }

    pub async fn register_service(self: &mut Self, keep_alive_sec: u64, service_ttl_sec: u64, key_prefix: &String,
                                  svc_id: &String, svc_name: &String, svc_addr: &String) -> Result<(), Box<dyn Error>> {
        let svc_id = if svc_id == "" { "/etcd_services".to_owned() } else { svc_id.clone() };
        let service_ttl_sec = if service_ttl_sec == 0 { 30 as u64 } else { service_ttl_sec };
        let keep_alive_sec= if keep_alive_sec == 0 { 5 as u64 } else { keep_alive_sec };

        {
            // watch keep alive event
            let mut inbound = self.client.lock().await.lease().keep_alive_responses().await.unwrap();
            tokio::spawn(async move {
                loop {
                    match inbound.next().await {
                        Some(resp) => {
                            debug!("keep alive response: {:?} at {:?}", resp, SystemTime::now());
                        }
                        None => {
                            warn!("[Cancel] keep alive at {:?}", SystemTime::now());
                        }
                    }
                }
            });
        }

        let lease_grant = self.client.lock().await.lease()
            .grant(LeaseGrantRequest::new(Duration::from_secs(service_ttl_sec))).await?;

        let key = [key_prefix.clone(), svc_id.clone(), svc_name.clone()].join("/");
        let mut req_with_lease = PutRequest::new(key, svc_addr.clone());
        req_with_lease.set_lease(lease_grant.id());
        self.client.lock().await.kv().put(req_with_lease).await?;

        {
            // keep alive the lease
            let arc_client = self.client.clone();
            tokio::spawn(async move {
                loop {
                    tokio::time::interval(Duration::from_secs(keep_alive_sec)).tick().await;
                    arc_client.lock().await.lease().keep_alive(LeaseKeepAliveRequest::new(lease_grant.id())).await.unwrap();
                }
            });
        }

        Ok(())
    }
}
