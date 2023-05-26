use rusty_duplication::model::Result;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, Mutex};

pub type RequestSender = mpsc::Sender<(ShremdupRequest, oneshot::Sender<ShremdupReply>)>;
pub type ReplyReceiver = mpsc::Receiver<(ShremdupRequest, oneshot::Sender<ShremdupReply>)>;
pub type ServerMutex = Arc<Mutex<RequestSender>>;

tonic::include_proto!("shremdup");

#[derive(Debug)]
pub enum ShremdupRequest {
  ListDisplays,
  GetDisplay(u32),
  CreateCapture(u32, String, bool),
  DeleteCapture(u32),
  TakeCapture(u32),
}

#[derive(Debug)]
pub enum ShremdupReply {
  ListDisplays(Result<Vec<DisplayInfo>>),
  GetDisplay(Result<DisplayInfo>),
  CreateCapture(Result<()>),
  DeleteCapture(Result<()>),
  TakeCapture(Result<(bool, Option<PointerPosition>, Option<PointerShape>)>),
}
