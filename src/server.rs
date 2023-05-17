use crate::model::shremdup_server::{Shremdup, ShremdupServer};
use crate::model::{
  CreateCaptureReply, CreateCaptureRequest, DeleteCaptureReply, DeleteCaptureRequest,
  GetDisplayReply, GetDisplayRequest, ListDisplaysReply, ListDisplaysRequest, TakeCaptureReply,
  TakeCaptureRequest,
};
use crate::model::{RequestSender, ServerMutex, ShremdupReply, ShremdupRequest};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::sync::oneshot;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

#[derive(Debug)]
pub struct TheShremdup {
  mutex: ServerMutex,
  sender: RequestSender,
}

impl TheShremdup {
  pub fn new(mutex: ServerMutex, sender: RequestSender) -> Self {
    Self { mutex, sender }
  }
}

#[tonic::async_trait]
impl Shremdup for TheShremdup {
  async fn list_displays(
    &self,
    _request: Request<ListDisplaysRequest>,
  ) -> Result<Response<ListDisplaysReply>, Status> {
    let _ = self.mutex.lock().await;
    let (tx, rx) = oneshot::channel();
    if let Err(err) = self.sender.send((ShremdupRequest::ListDisplays, tx)).await {
      return Err(Status::internal(err.to_string()));
    }
    match rx.await {
      Err(_) => Err(Status::internal("failed to receive reply")),
      Ok(ShremdupReply::ListDisplays(Ok(displays))) => {
        Ok(Response::new(ListDisplaysReply { infos: displays }))
      }
      Ok(ShremdupReply::ListDisplays(Err(err))) => Err(Status::internal(err.to_string())),
      Ok(_) => Err(Status::internal("invalid reply")),
    }
  }

  async fn get_display(
    &self,
    request: Request<GetDisplayRequest>,
  ) -> Result<Response<GetDisplayReply>, Status> {
    let _ = self.mutex.lock().await;
    let (tx, rx) = oneshot::channel();
    let request = request.into_inner();
    if let Err(err) = self
      .sender
      .send((ShremdupRequest::GetDisplay(request.id), tx))
      .await
    {
      return Err(Status::internal(err.to_string()));
    }
    match rx.await {
      Err(_) => Err(Status::internal("failed to receive reply")),
      Ok(ShremdupReply::GetDisplay(Ok(info))) => {
        Ok(Response::new(GetDisplayReply { info: Some(info) }))
      }
      Ok(ShremdupReply::GetDisplay(Err(err))) => Err(Status::internal(err.to_string())),
      Ok(_) => Err(Status::internal("invalid reply")),
    }
  }

  async fn create_capture(
    &self,
    request: Request<CreateCaptureRequest>,
  ) -> Result<Response<CreateCaptureReply>, Status> {
    let _ = self.mutex.lock().await;
    let (tx, rx) = oneshot::channel();
    let request = request.into_inner();
    if let Err(err) = self
      .sender
      .send((ShremdupRequest::CreateCapture(request.id, request.name), tx))
      .await
    {
      return Err(Status::internal(err.to_string()));
    }
    match rx.await {
      Err(_) => Err(Status::internal("failed to receive reply")),
      Ok(ShremdupReply::CreateCapture(Ok(_))) => Ok(Response::new(CreateCaptureReply {})),
      Ok(ShremdupReply::CreateCapture(Err(err))) => Err(Status::internal(err.to_string())),
      Ok(_) => Err(Status::internal("invalid reply")),
    }
  }

  async fn delete_capture(
    &self,
    request: Request<DeleteCaptureRequest>,
  ) -> Result<Response<DeleteCaptureReply>, Status> {
    let _ = self.mutex.lock().await;
    let (tx, rx) = oneshot::channel();
    let request = request.into_inner();
    if let Err(err) = self
      .sender
      .send((ShremdupRequest::DeleteCapture(request.id), tx))
      .await
    {
      return Err(Status::internal(err.to_string()));
    }
    match rx.await {
      Err(_) => Err(Status::internal("failed to receive reply")),
      Ok(ShremdupReply::DeleteCapture(Ok(_))) => Ok(Response::new(DeleteCaptureReply {})),
      Ok(ShremdupReply::DeleteCapture(Err(err))) => Err(Status::internal(err.to_string())),
      Ok(_) => Err(Status::internal("invalid reply")),
    }
  }

  async fn take_capture(
    &self,
    request: Request<TakeCaptureRequest>,
  ) -> Result<Response<TakeCaptureReply>, Status> {
    let _ = self.mutex.lock().await;
    let (tx, rx) = oneshot::channel();
    let request = request.into_inner();
    if let Err(err) = self
      .sender
      .send((ShremdupRequest::TakeCapture(request.id), tx))
      .await
    {
      return Err(Status::internal(err.to_string()));
    }
    match rx.await {
      Err(_) => Err(Status::internal("failed to receive reply")),
      Ok(ShremdupReply::TakeCapture(Ok(update))) => Ok(Response::new(TakeCaptureReply { update })),
      Ok(ShremdupReply::TakeCapture(Err(err))) => Err(Status::internal(err.to_string())),
      Ok(_) => Err(Status::internal("invalid reply")),
    }
  }
}

pub async fn server_thread(mutex: ServerMutex, tx: RequestSender, port: u16) {
  let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
  let shremdup = TheShremdup::new(mutex, tx);

  Server::builder()
    .add_service(ShremdupServer::new(shremdup))
    .serve(addr)
    .await
    .unwrap();
}
