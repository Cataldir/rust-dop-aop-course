//! Explicit aspect joins. Application/domain decisions do not live in these wrappers.
use std::{future::Future,pin::Pin,sync::{Arc,atomic::{AtomicUsize,Ordering}},task::{Context,Poll}};
use tower::{Layer,Service};
pub use audit_macro::audit;
/// Only W3C context fields cross the wire; application bodies never enter this carrier.
pub struct HeaderCarrier(pub std::collections::BTreeMap<String,String>);
impl opentelemetry::propagation::Injector for HeaderCarrier {fn set(&mut self,key:&str,value:String){self.0.insert(key.into(),value);}}
impl opentelemetry::propagation::Extractor for HeaderCarrier {
    fn get(&self,key:&str)->Option<&str>{self.0.get(key).map(String::as_str)}
    fn keys(&self)->Vec<&str>{self.0.keys().map(String::as_str).collect()}
}
#[derive(Clone,Default)] pub struct Counts { pub started:Arc<AtomicUsize>,pub completed:Arc<AtomicUsize>,pub failed:Arc<AtomicUsize> }
#[derive(Clone)] pub struct CountLayer(pub Counts);
#[derive(Clone)] pub struct CountService<S> { inner:S,counts:Counts }
impl<S> Layer<S> for CountLayer {
    type Service=CountService<S>;
    fn layer(&self,inner:S)->Self::Service { CountService{inner,counts:self.0.clone()} }
}
impl<S,R> Service<R> for CountService<S>
where S:Service<R>, S::Future:Send+'static,S::Response:Send+'static,S::Error:Send+'static {
    type Response=S::Response;type Error=S::Error;
    type Future=Pin<Box<dyn Future<Output=Result<Self::Response,Self::Error>>+Send>>;
    fn poll_ready(&mut self,cx:&mut Context<'_>)->Poll<Result<(),Self::Error>> { self.inner.poll_ready(cx) }
    fn call(&mut self,request:R)->Self::Future {
        self.counts.started.fetch_add(1,Ordering::Relaxed);
        // Call the SAME ready inner service before moving its future. No clone readiness bug.
        let future=self.inner.call(request);let counts=self.counts.clone();
        Box::pin(async move { let result=future.await;counts.completed.fetch_add(1,Ordering::Relaxed);if result.is_err(){counts.failed.fetch_add(1,Ordering::Relaxed);}result })
    }
}
/// Construct once inside a Tokio runtime; retain provider until shutdown.
pub fn telemetry(service:&str)->anyhow::Result<opentelemetry_sdk::trace::SdkTracerProvider> {
    use opentelemetry::trace::TracerProvider;
    use opentelemetry_otlp::WithExportConfig;
    use tracing_subscriber::prelude::*;
    let endpoint=std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").unwrap_or_else(|_|"http://127.0.0.1:4317".into());
    let exporter=opentelemetry_otlp::SpanExporter::builder().with_tonic().with_endpoint(endpoint).build()?;
    let provider=opentelemetry_sdk::trace::SdkTracerProvider::builder().with_batch_exporter(exporter).build();
    let tracer=provider.tracer(service.to_owned());
    tracing_subscriber::registry().with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_|"info".into()))
        .with(tracing_subscriber::fmt::layer()).with(tracing_opentelemetry::layer().with_tracer(tracer)).try_init()?;
    opentelemetry::global::set_text_map_propagator(opentelemetry_sdk::propagation::TraceContextPropagator::new());
    Ok(provider)
}
#[cfg(test)] mod tests {
    use super::*;use tower::{service_fn,ServiceExt};
    #[tokio::test] async fn errors_are_counted_and_preserved() {
        let counts=Counts::default();let service=CountLayer(counts.clone()).layer(service_fn(|n:u8|async move {if n==0 {Err("zero")} else {Ok(n+1)}}));
        assert_eq!(service.clone().oneshot(0).await,Err("zero"));assert_eq!(service.oneshot(4).await,Ok(5));
        assert_eq!(counts.started.load(Ordering::Relaxed),2);assert_eq!(counts.completed.load(Ordering::Relaxed),2);assert_eq!(counts.failed.load(Ordering::Relaxed),1);
    }
    #[audit] fn audited(n:u8)->Result<u8,&'static str> { if n==0{return Err("zero");}Ok(n+1) }
    #[test] fn macro_preserves_early_return_and_result() {assert_eq!(audited(0),Err("zero"));assert_eq!(audited(2),Ok(3));}
}
