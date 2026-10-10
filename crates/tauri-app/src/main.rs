#![cfg_attr(not(debug_assertions),windows_subsystem="windows")]
use std::sync::Mutex;
use tauri::{Emitter,Manager,State};
use futures_util::StreamExt;
struct Core(Mutex<desktop_core::Model>);
struct Client {http:reqwest::Client,token:String}
#[tauri::command] fn view(core:State<Core>)->Result<desktop_core::View,String> {Ok(core.0.lock().map_err(|_|"state poisoned")?.view().clone())}
#[tauri::command] fn dispatch(command:desktop_core::Command,core:State<Core>)->Result<desktop_core::View,String> {
    core.0.lock().map_err(|_|"state poisoned")?.dispatch(command).map_err(|e|e.to_string())
}
#[tauri::command] async fn create_order(id:String,units:u32,client:State<'_,Client>)->Result<serde_json::Value,String> {
    let response=client.http.post("http://127.0.0.1:3000/orders").bearer_auth(&client.token).json(&serde_json::json!({"order_id":id,"units":units})).send().await.map_err(|e|e.to_string())?;
    if !response.status().is_success(){return Err(format!("API refused {}",response.status()));}
    Ok(serde_json::json!({"submitted":true}))
}
#[tauri::command] async fn refresh_order(id:String,client:State<'_,Client>)->Result<serde_json::Value,String> {
    if !id.bytes().all(|b|b.is_ascii_alphanumeric()||b"_-.".contains(&b))||id.is_empty()||id.len()>64 {return Err("invalid order id".into());}
    client.http.get(format!("http://127.0.0.1:3000/orders/{id}")).bearer_auth(&client.token).send().await.map_err(|e|e.to_string())?.error_for_status().map_err(|e|e.to_string())?.json().await.map_err(|e|e.to_string())
}
#[tauri::command] async fn metric_query(client:State<'_,Client>)->Result<serde_json::Value,String> {
    client.http.get("http://127.0.0.1:3000/query?metric=cpu&from=0&to=60000&width=1000").bearer_auth(&client.token).send().await.map_err(|e|e.to_string())?.error_for_status().map_err(|e|e.to_string())?.json().await.map_err(|e|e.to_string())
}
fn main() {
    tauri::Builder::default().setup(|app|{
        let directory=app.path().app_local_data_dir()?.join("learner-snapshot");
        let model=desktop_core::Model::open(sled::open(directory)?)?;
        app.manage(Core(Mutex::new(model)));
        let token=std::env::var("COURSE_TOKEN")?;
        let http=reqwest::Client::builder().timeout(std::time::Duration::from_secs(5)).no_proxy().redirect(reqwest::redirect::Policy::none()).build()?;
        app.manage(Client{http,token});
        let handle=app.handle().clone();
        tauri::async_runtime::spawn(async move {
            // A display observer, not the durable fulfillment consumer. Reconcile via API after gaps.
            let result:Result<(),Box<dyn std::error::Error+Send+Sync>>=async {
                let events=handle.clone();
                let broker=async_nats::ConnectOptions::new().event_callback(move |event| {
                    let events=events.clone();
                    async move {
                        let status=match event {
                            async_nats::Event::Connected=>Some(serde_json::json!({"connected":true,"refresh_required":true})),
                            async_nats::Event::Disconnected|async_nats::Event::Closed=>Some(serde_json::json!({"disconnected":true,"refresh_required":true})),
                            async_nats::Event::SlowConsumer(_)=>Some(serde_json::json!({"gap_possible":true,"refresh_required":true})),
                            _=>None,
                        };
                        if let Some(status)=status {let _=events.emit("orders-event",status);}
                    }
                }).connect("nats://127.0.0.1:4222").await?;
                let mut subscription=broker.subscribe("orders.created").await?;
                while let Some(message)=subscription.next().await {
                    if message.payload.len()<=1024*1024 {handle.emit("orders-event",serde_json::json!({"refresh_required":true}))?;}
                }
                Ok(())
            }.await;
            if result.is_err(){let _=handle.emit("orders-event",serde_json::json!({"disconnected":true,"refresh_required":true}));}
        });
        Ok(())
    }).invoke_handler(tauri::generate_handler![view,dispatch,create_order,refresh_order,metric_query])
      .run(tauri::generate_context!()).expect("Tauri course app startup failed");
}
