use crate::model::shremdup_server::{Shremdup, ShremdupServer};
use crate::model::{
  CreateCaptureReply, CreateCaptureRequest, DeleteCaptureReply, DeleteCaptureRequest,
  GetDisplayReply, GetDisplayRequest, ListDisplaysReply, ListDisplaysRequest, TakeCaptureReply,
  TakeCaptureRequest,
};
use crate::model::{ServerMutex, ShremdupReply, ShremdupRequest};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tonic::transport::Server;
use tonic::{Request, Response, Status};

#[derive(Debug)]
pub struct TheShremdup {
  mutex: ServerMutex,
}

impl TheShremdup {
  pub fn new(mutex: ServerMutex) -> Self {
    Self { mutex }
  }
}

#[tonic::async_trait]
impl Shremdup for TheShremdup {
  async fn list_displays(
    &self,
    _request: Request<ListDisplaysRequest>,
  ) -> Result<Response<ListDisplaysReply>, Status> {
    let mut guard = self.mutex.lock().await;
    if let Err(err) = (guard.0).send(ShremdupRequest::ListDisplays).await {
      return Err(Status::internal(err.to_string()));
    }
    match (guard.1).recv().await {
      None => Err(Status::internal("failed to receive reply")),
      Some(ShremdupReply::ListDisplays(Ok(infos))) => {
        Ok(Response::new(ListDisplaysReply { infos }))
      }
      Some(ShremdupReply::ListDisplays(Err(err))) => Err(Status::internal(err.to_string())),
      Some(_) => Err(Status::internal("invalid reply")),
    }
  }

  async fn get_display(
    &self,
    request: Request<GetDisplayRequest>,
  ) -> Result<Response<GetDisplayReply>, Status> {
    let mut guard = self.mutex.lock().await;
    let request = request.into_inner();
    if let Err(err) = (guard.0)
      .send(ShremdupRequest::GetDisplay(request.id))
      .await
    {
      return Err(Status::internal(err.to_string()));
    }
    match (guard.1).recv().await {
      None => Err(Status::internal("failed to receive reply")),
      Some(ShremdupReply::GetDisplay(Ok(info))) => {
        Ok(Response::new(GetDisplayReply { info: Some(info) }))
      }
      Some(ShremdupReply::GetDisplay(Err(err))) => Err(Status::internal(err.to_string())),
      Some(_) => Err(Status::internal("invalid reply")),
    }
  }

  async fn create_capture(
    &self,
    request: Request<CreateCaptureRequest>,
  ) -> Result<Response<CreateCaptureReply>, Status> {
    let mut guard = self.mutex.lock().await;
    let request = request.into_inner();
    if let Err(err) = (guard.0)
      .send(ShremdupRequest::CreateCapture(
        request.id,
        request.name,
        request.open,
      ))
      .await
    {
      return Err(Status::internal(err.to_string()));
    }
    match (guard.1).recv().await {
      None => Err(Status::internal("failed to receive reply")),
      Some(ShremdupReply::CreateCapture(Ok(_))) => Ok(Response::new(CreateCaptureReply {})),
      Some(ShremdupReply::CreateCapture(Err(err))) => Err(Status::internal(err.to_string())),
      Some(_) => Err(Status::internal("invalid reply")),
    }
  }

  async fn delete_capture(
    &self,
    request: Request<DeleteCaptureRequest>,
  ) -> Result<Response<DeleteCaptureReply>, Status> {
    let mut guard = self.mutex.lock().await;
    let request = request.into_inner();
    if let Err(err) = (guard.0)
      .send(ShremdupRequest::DeleteCapture(request.id))
      .await
    {
      return Err(Status::internal(err.to_string()));
    }
    match (guard.1).recv().await {
      None => Err(Status::internal("failed to receive reply")),
      Some(ShremdupReply::DeleteCapture(Ok(_))) => Ok(Response::new(DeleteCaptureReply {})),
      Some(ShremdupReply::DeleteCapture(Err(err))) => Err(Status::internal(err.to_string())),
      Some(_) => Err(Status::internal("invalid reply")),
    }
  }

  async fn take_capture(
    &self,
    request: Request<TakeCaptureRequest>,
  ) -> Result<Response<TakeCaptureReply>, Status> {
    let mut guard = self.mutex.lock().await;
    let request = request.into_inner();
    if let Err(err) = (guard.0)
      .send(ShremdupRequest::TakeCapture(request.id))
      .await
    {
      return Err(Status::internal(err.to_string()));
    }
    match (guard.1).recv().await {
      None => Err(Status::internal("failed to receive reply")),
      Some(ShremdupReply::TakeCapture(Ok((desktop_updated, pointer_position, pointer_shape)))) => {
        Ok(Response::new(TakeCaptureReply {
          desktop_updated,
          pointer_position,
          pointer_shape,
        }))
      }
      Some(ShremdupReply::TakeCapture(Err(err))) => Err(Status::internal(err.to_string())),
      Some(_) => Err(Status::internal("invalid reply")),
    }
  }
}

pub async fn server_thread(mutex: ServerMutex, port: u16) {
  let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
  let shremdup = TheShremdup::new(mutex);

  Server::builder()
    .add_service(ShremdupServer::new(shremdup))
    .serve(addr)
    .await
    .unwrap();
}
