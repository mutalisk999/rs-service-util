# rs-service-util


### How To Use: Test service register and watcher

```
use tokio;
use rs_service_util::svc::register::RegSvcClient;
use tokio::time::Duration;
use rs_service_util::svc::monitor::MonSvcClient;


fn func_put(k: String, v: String) -> () {
    println!("func_put, need to add kv: {:?}/{:?}", k, v);
}

fn func_delete(k: String) -> () {
    println!("func_delete, need to delete k: {:?}", k);
}

#[tokio::main]
async fn main() {
    // register service and keep alive
    tokio::spawn(async move {
        let mut regcli = RegSvcClient::new(
            vec![
                "http://127.0.0.1:12379".to_owned(),
                "http://127.0.0.1:22379".to_owned(),
                "http://127.0.0.1:32379".to_owned()
            ], None, None,
        ).await.unwrap();
        tokio::time::sleep(Duration::from_secs(10)).await;
        regcli.register_service(5, 30, "".to_owned(),
                                "test".to_owned(), "key".to_owned(),
                                "value".to_owned()).await.unwrap();
        tokio::time::sleep(Duration::from_secs(30)).await;
        regcli.dispose_reg_svc_client().await.unwrap();
    });

    // watch service and call put/delete callback function
    tokio::spawn(async move {
        let mut moncli = MonSvcClient::new(
            vec![
                "http://127.0.0.1:12379".to_owned(),
                "http://127.0.0.1:22379".to_owned(),
                "http://127.0.0.1:32379".to_owned()
            ], None, None,
        ).await.unwrap();
        moncli.monitor_service("".to_owned(), &func_put,
                               &func_delete).await.unwrap();
        tokio::time::sleep(Duration::from_secs(100)).await;
        moncli.dispose_reg_svc_client().await.unwrap();
    });

    tokio::time::sleep(Duration::from_secs(120)).await;
}
```

### Dependencies

```
[dependencies]
tokio = { version = "1.2.0", features = ["full"] }
rs-service-util = { git = "https://github.com/mutalisk999/rs-service-util" }
```

### Test log

```
func_put, need to add kv: "/etcd_services/test/key"/"value"
keep alive request at "2022-03-06 10:12:14.536026600 UTC"
service watcher put "/etcd_services/test/key" | "value" at "2022-03-06 10:12:14.535995300 UTC"
keep alive response: Ok(LeaseKeepAliveResponse { proto: LeaseKeepAliveResponse { header: Some(ResponseHeader { cluster_id: 17237436991929493444, member_id: 9372538179322589801, revision: 79, raft_term: 23 }), id: 3632574612849185088, ttl: 30 } }) at "2022-03-06 10:12:14.549221800 UTC"
keep alive request at "2022-03-06 10:12:19.538808600 UTC"
keep alive response: Ok(LeaseKeepAliveResponse { proto: LeaseKeepAliveResponse { header: Some(ResponseHeader { cluster_id: 17237436991929493444, member_id: 9372538179322589801, revision: 79, raft_term: 23 }), id: 3632574612849185088, ttl: 30 } }) at "2022-03-06 10:12:19.553139700 UTC"
keep alive request at "2022-03-06 10:12:24.544690600 UTC"
keep alive response: Ok(LeaseKeepAliveResponse { proto: LeaseKeepAliveResponse { header: Some(ResponseHeader { cluster_id: 17237436991929493444, member_id: 9372538179322589801, revision: 79, raft_term: 23 }), id: 3632574612849185088, ttl: 30 } }) at "2022-03-06 10:12:24.557441200 UTC"
keep alive request at "2022-03-06 10:12:29.556795600 UTC"
keep alive response: Ok(LeaseKeepAliveResponse { proto: LeaseKeepAliveResponse { header: Some(ResponseHeader { cluster_id: 17237436991929493444, member_id: 9372538179322589801, revision: 79, raft_term: 23 }), id: 3632574612849185088, ttl: 30 } }) at "2022-03-06 10:12:29.571412500 UTC"
keep alive request at "2022-03-06 10:12:34.564148600 UTC"
keep alive response: Ok(LeaseKeepAliveResponse { proto: LeaseKeepAliveResponse { header: Some(ResponseHeader { cluster_id: 17237436991929493444, member_id: 9372538179322589801, revision: 79, raft_term: 23 }), id: 3632574612849185088, ttl: 30 } }) at "2022-03-06 10:12:34.576709900 UTC"
keep alive request at "2022-03-06 10:12:39.570686500 UTC"
keep alive response: Ok(LeaseKeepAliveResponse { proto: LeaseKeepAliveResponse { header: Some(ResponseHeader { cluster_id: 17237436991929493444, member_id: 9372538179322589801, revision: 79, raft_term: 23 }), id: 3632574612849185088, ttl: 30 } }) at "2022-03-06 10:12:39.583784600 UTC"
[Cancel] keep alive response at "2022-03-06 10:12:44.546953300 UTC"
[Cancel] keep alive request at "2022-03-06 10:12:44.577212900 UTC"
func_delete, need to delete k: "/etcd_services/test/key"
service watcher delete "/etcd_services/test/key" at "2022-03-06 10:13:09.947781800 UTC"
[Cancel] service watcher at "2022-03-06 10:13:44.534192100 UTC"
```

### Log printing

* Strongly recommend to use crate flexi_logger to dup log to stdout `(by set duplicate_to_stdout Duplicate::All)`, so I remove all println! log.

```
fn init_log() {
    flexi_logger::Logger::with_str("debug")
        .log_to_file()
        .directory("log")
        .basename("app_name.log")
        .duplicate_to_stdout(Duplicate::All)
        .format_for_files(detailed_format)
        .format_for_stdout(detailed_format)
        .start()
        .unwrap_or_else(|e| panic!("logger initialization failed, err: {}", e));
}
```

### Dependencies (for log printing)

```
[dependencies]
flexi_logger = "0.17"
log = "0.4.14"
```