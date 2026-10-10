//! Metrics primary; order appendix endpoints bridge the capstone.
//! A synthetic HS256 issuer is for loopback teaching only, not production identity.
use axum::{body::Body,extract::{DefaultBodyLimit,Query,State,Path,ws::{Message,WebSocket,WebSocketUpgrade}},http::{Request,StatusCode,header::AUTHORIZATION},middleware::{self,Next},response::{IntoResponse,Response},routing::{get,post},Json,Router};
use jsonwebtoken::{Algorithm,DecodingKey,Validation};
use serde::{Deserialize,Serialize};
use shared_kernel::{ContractError,IngestBatch,MetricColumns,Tenant,Window,downsample,EventEnvelope,OrderCreated,valid_id};
use sqlx::{PgPool,Row};
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_governor::{governor::GovernorConfigBuilder,key_extractor::KeyExtractor,GovernorError,GovernorLayer};
use tower_http::{compression::CompressionLayer,request_id::{MakeRequestUuid,SetRequestIdLayer,PropagateRequestIdLayer},trace::TraceLayer};
use tracing_opentelemetry::OpenTelemetrySpanExt;
#[derive(Clone)] pub struct AppState(pub Arc<Inner>);
pub struct Inner { pub pool:PgPool,pub tail:broadcast::Sender<Arc<Vec<shared_kernel::Point>>>,pub secret:String }
#[derive(Clone,Debug)] pub struct AuthTenant(pub Tenant);
#[derive(Serialize,Deserialize)] pub struct Claims { pub sub:String,pub tenant:Tenant,pub exp:usize,pub iss:String,pub aud:String }
pub fn decode_tenant(token:&str,secret:&str)->Result<Tenant,StatusCode> {
    let mut validation=Validation::new(Algorithm::HS256);validation.leeway=0;
    validation.set_issuer(&["course-local"]);validation.set_audience(&["course-api"]);
    jsonwebtoken::decode::<Claims>(token,&DecodingKey::from_secret(secret.as_bytes()),&validation)
        .map(|token|token.claims.tenant).map_err(|_|StatusCode::UNAUTHORIZED)
}
async fn authenticate(State(state):State<AppState>,mut request:Request<Body>,next:Next)->Result<Response,StatusCode> {
    let bearer=request.headers().get(AUTHORIZATION).and_then(|v|v.to_str().ok()).and_then(|v|v.strip_prefix("Bearer ")).ok_or(StatusCode::UNAUTHORIZED)?;
    let tenant=decode_tenant(bearer,&state.0.secret)?;
    request.extensions_mut().insert(AuthTenant(tenant));Ok(next.run(request).await)
}
#[derive(Clone)] struct TenantKey;
impl KeyExtractor for TenantKey {
    type Key=Tenant;
    fn extract<T>(&self,request:&Request<T>)->Result<Self::Key,GovernorError> {
        request.extensions().get::<AuthTenant>().map(|t|t.0.clone()).ok_or(GovernorError::UnableToExtractKey)
    }
}
#[derive(Debug)] pub struct ApiError(pub StatusCode,pub &'static str);
impl IntoResponse for ApiError { fn into_response(self)->Response { (self.0,Json(serde_json::json!({"error":self.1}))).into_response() } }
impl From<ContractError> for ApiError {
    fn from(e:ContractError)->Self {match e {ContractError::TenantMismatch=>Self(StatusCode::FORBIDDEN,"tenant mismatch"),_=>Self(StatusCode::UNPROCESSABLE_ENTITY,"invalid input")}}
}
fn storage_error(_e:sqlx::Error)->ApiError { tracing::error!("storage operation failed");ApiError(StatusCode::SERVICE_UNAVAILABLE,"storage unavailable") }
/// Build auth before authenticated tenant quota. Trace/request-id also cover rejection.
pub fn router(state:AppState)->anyhow::Result<Router> {
    let governor=GovernorConfigBuilder::default().key_extractor(TenantKey).per_second(1).burst_size(20).finish().ok_or_else(||anyhow::anyhow!("invalid quota"))?;
    let protected=Router::new().route("/ingest",post(ingest)).route("/query",get(query)).route("/tail",get(tail))
        .route("/orders",post(create_order)).route("/orders/{id}",get(order))
        .layer(GovernorLayer::new(governor))
        .layer(middleware::from_fn_with_state(state.clone(),authenticate))
        .layer(DefaultBodyLimit::max(1024*1024));
    Ok(Router::new().merge(protected).route("/healthz",get(||async {"alive"})).route("/readyz",get(ready))
        .with_state(state).layer(tower::ServiceBuilder::new()
            .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
            .layer(PropagateRequestIdLayer::x_request_id())
            .layer(TraceLayer::new_for_http()).layer(CompressionLayer::new())))
}
#[tracing::instrument(skip(state,batch),fields(points=batch.points.len()))]
async fn ingest(State(state):State<AppState>,axum::Extension(AuthTenant(tenant)):axum::Extension<AuthTenant>,Json(batch):Json<IngestBatch>)->Result<StatusCode,ApiError> {
    let columns=MetricColumns::from_batch(batch,&tenant)?;
    // Arrays are equal length by construction. Typed columns cross one explicit effect join.
    let tags:Vec<serde_json::Value>=columns.tags().iter().map(|t|serde_json::json!(t)).collect();
    let mut tx=state.0.pool.begin().await.map_err(storage_error)?;
    sqlx::query("INSERT INTO metric_point(tenant,metric,timestamp_ms,value,tags) SELECT $1,m,t,v,j FROM UNNEST($2::text[],$3::bigint[],$4::float8[],$5::jsonb[]) AS b(m,t,v,j) ON CONFLICT(tenant,metric,timestamp_ms) DO UPDATE SET value=EXCLUDED.value,tags=EXCLUDED.tags")
        .bind(tenant.as_str()).bind(columns.metrics()).bind(columns.timestamps()).bind(columns.values()).bind(&tags)
        .execute(&mut *tx).await.map_err(storage_error)?;
    tx.commit().await.map_err(storage_error)?;
    metrics::counter!("course_ingested_points_total").increment(columns.len() as u64);
    // Publish ONLY after commit. Ephemeral tail can miss history; query is authoritative.
    let _=state.0.tail.send(Arc::new(columns.into_points(tenant)));Ok(StatusCode::NO_CONTENT)
}
#[derive(Deserialize)] pub struct QueryRequest { pub metric:String,pub from:i64,pub to:i64,pub width:i64 }
async fn query(State(state):State<AppState>,axum::Extension(AuthTenant(tenant)):axum::Extension<AuthTenant>,Query(q):Query<QueryRequest>)->Result<Json<Vec<Window>>,ApiError> {
    if !valid_id(&q.metric)||q.from<0||q.to<=q.from||q.to-q.from>86400000||q.width<=0 {return Err(ApiError(StatusCode::UNPROCESSABLE_ENTITY,"invalid query"));}
    let rows=sqlx::query("SELECT timestamp_ms,value FROM metric_point WHERE tenant=$1 AND metric=$2 AND timestamp_ms >= $3 AND timestamp_ms < $4 ORDER BY timestamp_ms LIMIT 10001")
        .bind(tenant.as_str()).bind(&q.metric).bind(q.from).bind(q.to).fetch_all(&state.0.pool).await.map_err(storage_error)?;
    if rows.len()>10000{return Err(ApiError(StatusCode::PAYLOAD_TOO_LARGE,"query requires smaller range"));}
    let points:Result<Vec<(i64,f64)>,sqlx::Error>=rows.iter().map(|r|Ok((r.try_get("timestamp_ms")?,r.try_get("value")?))).collect();
    Ok(Json(downsample(&points.map_err(storage_error)?,q.width)?))
}
async fn tail(State(state):State<AppState>,axum::Extension(AuthTenant(tenant)):axum::Extension<AuthTenant>,ws:WebSocketUpgrade)->impl IntoResponse {
    let receiver=state.0.tail.subscribe();ws.on_upgrade(move |socket|stream_tail(socket,receiver,tenant))
}
async fn stream_tail(mut socket:WebSocket,mut receiver:broadcast::Receiver<Arc<Vec<shared_kernel::Point>>>,tenant:Tenant) {
    loop {tokio::select! {
        received=receiver.recv()=>match received {
            Ok(batch)=>{
                let visible:Vec<_>=batch.iter().filter(|p|p.tenant==tenant).collect();if visible.is_empty(){continue;}
                let Ok(text)=serde_json::to_string(&visible) else {break;};
                if !matches!(tokio::time::timeout(std::time::Duration::from_secs(2),socket.send(Message::Text(text.into()))).await,Ok(Ok(()))){break;}
            }
            Err(broadcast::error::RecvError::Lagged(n))=>{
                tracing::warn!(missed=n,"tail lag: resync required");
                let _=tokio::time::timeout(std::time::Duration::from_secs(2),
                    socket.send(Message::Text("{\"resync_required\":true}".into()))).await;
                break;
            }
            Err(broadcast::error::RecvError::Closed)=>break,
        },
        received=socket.recv()=>match received {Some(Ok(Message::Close(_)))|None|Some(Err(_))=>break,_=>{}},
    }}
}
async fn ready(State(state):State<AppState>)->StatusCode {
    match tokio::time::timeout(std::time::Duration::from_secs(1),sqlx::query("SELECT 1").execute(&state.0.pool)).await {
        Ok(Ok(_))=>StatusCode::OK,_=>StatusCode::SERVICE_UNAVAILABLE,
    }
}
#[derive(Deserialize)] pub struct NewOrder {pub order_id:String,pub units:u32}
async fn create_order(State(state):State<AppState>,axum::Extension(AuthTenant(tenant)):axum::Extension<AuthTenant>,Json(input):Json<NewOrder>)->Result<StatusCode,ApiError> {
    let envelope=EventEnvelope{version:1,event_id:input.order_id.clone(),tenant:tenant.clone(),payload:OrderCreated{order_id:input.order_id.clone(),units:input.units}};
    envelope.validate()?;
    let mut tx=state.0.pool.begin().await.map_err(storage_error)?;
    let existing=sqlx::query("SELECT units FROM course_order WHERE tenant=$1 AND order_id=$2").bind(tenant.as_str()).bind(&input.order_id).fetch_optional(&mut *tx).await.map_err(storage_error)?;
    if let Some(row)=existing {let units:i32=row.try_get("units").map_err(storage_error)?;if units!=input.units as i32{return Err(ApiError(StatusCode::CONFLICT,"order key reused with different units"));}return Ok(StatusCode::OK);}
    // PK conflict from concurrent requests is a conflict, never a second semantic order.
    let result=sqlx::query("INSERT INTO course_order(tenant,order_id,units,status) VALUES($1,$2,$3,'accepted')").bind(tenant.as_str()).bind(&input.order_id).bind(input.units as i32).execute(&mut *tx).await;
    match result {Ok(_)=>{},Err(sqlx::Error::Database(e)) if e.is_unique_violation()=>return Err(ApiError(StatusCode::CONFLICT,"concurrent order conflict")),Err(e)=>return Err(storage_error(e))}
    let mut carrier=aspects::HeaderCarrier(Default::default());
    opentelemetry::global::get_text_map_propagator(|p|p.inject_context(&tracing::Span::current().context(),&mut carrier));
    sqlx::query("INSERT INTO outbox(tenant,event_id,payload,trace_headers) VALUES($1,$2,$3,$4)").bind(tenant.as_str()).bind(&envelope.event_id).bind(serde_json::json!(envelope)).bind(serde_json::json!(carrier.0)).execute(&mut *tx).await.map_err(storage_error)?;
    tx.commit().await.map_err(storage_error)?;Ok(StatusCode::CREATED)
}
async fn order(State(state):State<AppState>,axum::Extension(AuthTenant(tenant)):axum::Extension<AuthTenant>,Path(id):Path<String>)->Result<Json<serde_json::Value>,ApiError> {
    if !valid_id(&id){return Err(ApiError(StatusCode::UNPROCESSABLE_ENTITY,"invalid order id"));}
    let row=sqlx::query("SELECT units,status FROM course_order WHERE tenant=$1 AND order_id=$2").bind(tenant.as_str()).bind(&id).fetch_optional(&state.0.pool).await.map_err(storage_error)?.ok_or(ApiError(StatusCode::NOT_FOUND,"order not found"))?;
    Ok(Json(serde_json::json!({"order_id":id,"units":row.try_get::<i32,_>("units").map_err(storage_error)?,"status":row.try_get::<String,_>("status").map_err(storage_error)?})))
}
