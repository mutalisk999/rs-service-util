use std::sync::Arc;
use tokio::sync::Mutex;
use etcd_rs::{Client, ClientConfig, RangeRequest, KeyRange, EventType};
use tonic::transport::ClientTlsConfig;
use std::error::Error;
use std::collections::HashMap;
use log::{debug, warn};
use std::time::SystemTime;
use tokio_stream::StreamExt;


pub struct MonSvcClient {
    client: Arc<Mutex<Client>>,
    svc_map: Arc<Mutex<HashMap<String, String>>>,
}

impl MonSvcClient {
    pub async fn new(endpoints: Vec<String>, tls_config: Option<ClientTlsConfig>,
                     auth_config: Option<(String, String)>) -> Result<MonSvcClient, Box<dyn Error>> {
        let cli_config = ClientConfig {
            endpoints: endpoints.clone(),
            auth: auth_config.clone(),
            tls: tls_config.clone(),
        };

        let client = Client::connect(cli_config).await?;
        Ok(MonSvcClient {
            client: Arc::new(Mutex::new(client)),
            svc_map: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn get_instance_handle(self: &mut Self) -> Arc<Mutex<Client>> {
        self.client.clone()
    }

    pub async fn get_service(self: &Self) -> HashMap<String, String> {
        self.svc_map.lock().await.clone()
    }

    pub async fn dispose_reg_svc_client(self: &mut Self) -> Result<(), Box<dyn Error>> {
        self.client.lock().await.shutdown().await?;
        Ok(())
    }

    pub async fn monitor_service(self: &mut Self, key_prefix: String, put_callback: &'static (dyn Fn(String, String) -> () + Sync),
                                 delete_callback: &'static (dyn Fn(String) -> () + Sync)) -> Result<(), Box<dyn Error>> {
        let key_prefix = if key_prefix == "" { "/etcd_services".to_owned() } else { key_prefix.clone() };

        let req_range = RangeRequest::new(KeyRange::prefix(key_prefix.clone()));
        let mut resp_range = self.client.lock().await.kv().range(req_range).await?;

        for kv in resp_range.take_kvs() {
            self.svc_map.lock().await.insert(kv.key_str().to_owned(), kv.value_str().to_owned());
            put_callback(kv.key_str().to_owned(), kv.value_str().to_owned());
        }

        {
            let client = self.client.clone();
            let svc_map = self.svc_map.clone();
            // deal with all received watch responses
            tokio::spawn(async move {
                let mut inbound = client.lock().await.watch(KeyRange::prefix(key_prefix.clone())).await.unwrap();
                loop {
                    match inbound.next().await {
                        Some(resp_res) => {
                            match resp_res {
                                Ok(resp) => {
                                    if resp.is_some() {
                                        for mut e in resp.unwrap().take_events()
                                        {
                                            let kv = e.take_kvs().unwrap();
                                            match e.event_type() {
                                                EventType::Put => {
                                                    svc_map.lock().await.insert(kv.key_str().to_owned(), kv.value_str().to_owned());
                                                    put_callback(kv.key_str().to_owned(), kv.value_str().to_owned());
                                                    println!("service watcher put {:?} | {:?} at {:?}", kv.key_str(), kv.value_str(), SystemTime::now());
                                                    debug!("service watcher put {:?} | {:?} at {:?}", kv.key_str(), kv.value_str(), SystemTime::now());
                                                }
                                                EventType::Delete => {
                                                    svc_map.lock().await.remove(&kv.key_str().to_owned());
                                                    delete_callback(kv.key_str().to_owned());
                                                    println!("service watcher delete {:?} at {:?}", kv.key_str(), SystemTime::now());
                                                    debug!("service watcher delete {:?} at {:?}", kv.key_str(), SystemTime::now());
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    println!("[Cancel] service watcher at {:?}, err: {:?}", SystemTime::now(), e);
                                    warn!("[Cancel] service watcher at {:?}, err: {:?}", SystemTime::now(), e);
                                    break;
                                }
                            }
                        }
                        None => {
                            println!("[Cancel] service watcher at {:?}", SystemTime::now());
                            warn!("[Cancel] service watcher at {:?}", SystemTime::now());
                            break;
                        }
                    }
                }
            });
        }

        Ok(())
    }
}