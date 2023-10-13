#![feature(impl_trait_in_assoc_type)]

use futures::{stream, TryFutureExt};
use tokio_stream::Stream;

mod stream_custom {
    tonic::include_proto!("stream_custom");
}

#[tokio::test]
async fn stream_custom_test() {
    use crate::stream_custom::Message;
    use std::time::Duration;
    use stream_custom::service_server::Service;
    use stream_custom::service_server::ServiceExt;
    use tokio::sync::oneshot;

    struct Svc;

    #[tonic::async_trait]
    impl stream_custom::service_server::Service for Svc {
        type RunStreamStream = tokio_stream::Once<Result<Message, tonic::Status>>;

        async fn run_stream(
            &self,
            _request: tonic::Request<Message>,
        ) -> Result<tonic::Response<Self::RunStreamStream>, tonic::Status> {
            Ok(tonic::Response::new(tokio_stream::once(Ok(Message {
                ok: true,
            }))))
        }
    }

    let svc = stream_custom::service_server::ServiceServer::new(Svc.wrap(
        |service, method, is_going_up| println!(">> {} {} up: {}", service, method, is_going_up),
    ));

    let (tx, rx) = oneshot::channel::<()>();

    let _ = tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(svc)
            .serve_with_shutdown("127.0.0.1:50051".parse().unwrap(), async { drop(rx.await) })
            .await
            .unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let mut client =
        stream_custom::service_client::ServiceClient::connect("http://127.0.0.1:50051")
            .await
            .unwrap();

    let mut stream = client
        .run_stream(tonic::Request::new(Message { ok: true }))
        .await
        .unwrap()
        .into_inner();

    let msg = stream.message().await.unwrap().unwrap();

    assert_eq!(msg.ok, true);
    tx.send(()).unwrap();
}
