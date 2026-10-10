use std::{future::{ready,Ready},task::{Context,Poll},sync::atomic::Ordering};
use tower::{Layer,Service,ServiceExt};
#[derive(Default)]struct ReadyProbe {ready:bool}
impl Clone for ReadyProbe {fn clone(&self)->Self {Self::default()}}
impl Service<()> for ReadyProbe {
    type Response=();type Error=&'static str;type Future=Ready<Result<(),Self::Error>>;
    fn poll_ready(&mut self,_:&mut Context<'_>)->Poll<Result<(),Self::Error>>{self.ready=true;Poll::Ready(Ok(()))}
    fn call(&mut self,_:())->Self::Future{let outcome=if self.ready {Ok(())}else{Err("unready clone called")};self.ready=false;ready(outcome)}
}
#[tokio::test]async fn wrapper_calls_the_ready_instance() {
    let counts=aspects::Counts::default();
    let service=aspects::CountLayer(counts.clone()).layer(ReadyProbe::default());
    assert_eq!(service.oneshot(()).await,Ok(()));
    assert_eq!(counts.completed.load(Ordering::Relaxed),1);
}
#[tokio::test]async fn dropped_future_has_no_completed_result() {
    let counts=aspects::Counts::default();
    let inner=tower::service_fn(|_:()|async {std::future::pending::<Result<(),&'static str>>().await});
    let mut service=aspects::CountLayer(counts.clone()).layer(inner);
    let response=service.ready().await.unwrap().call(());drop(response);
    assert_eq!(counts.started.load(Ordering::Relaxed),1);assert_eq!(counts.completed.load(Ordering::Relaxed),0);
}
#[aspects::audit]async fn audited(value:u8)->Result<u8,&'static str> {let accepted=if value==0{Err("zero")}else{Ok(value)}?;Ok(accepted+1)}
#[tokio::test]async fn async_audit_preserves_question_mark() {assert_eq!(audited(0).await,Err("zero"));assert_eq!(audited(3).await,Ok(4));}