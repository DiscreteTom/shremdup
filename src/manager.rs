use crate::model::{DxgiOutputDescExt, ReplyReceiver, ShremdupReply, ShremdupRequest};
use rusty_duplication::{
  capturer::{model::Capturer, shared::SharedCapturer},
  manager::Manager,
  utils::FrameInfoExt,
};
use std::collections::HashMap;

pub async fn manager_thread(mut rx: ReplyReceiver) {
  let manager = Manager::default().unwrap();
  let mut capturer_map: HashMap<u32, SharedCapturer> = HashMap::new();

  loop {
    let (req, tx) = rx.recv().await.unwrap();
    let reply = match req {
      ShremdupRequest::ListDisplays => {
        let mut displays = Vec::new();
        manager.contexts.iter().for_each(|ctx| match ctx.desc() {
          Ok(desc) => displays.push(desc.to_info()),
          Err(err) => println!("ListDisplays: {:?}", err),
        });
        ShremdupReply::ListDisplays(Ok(displays))
      }
      ShremdupRequest::GetDisplay(id) => match manager.contexts.get(id as usize) {
        None => ShremdupReply::GetDisplay(Err("invalid id".to_string())),
        Some(ctx) => match ctx.desc() {
          Ok(desc) => ShremdupReply::GetDisplay(Ok(desc.to_info())),
          Err(err) => ShremdupReply::GetDisplay(Err(err)),
        },
      },
      ShremdupRequest::CreateCapture(id, name) => {
        if capturer_map.contains_key(&id) {
          ShremdupReply::CreateCapture(Err("already exists".to_string()))
        } else {
          match manager.contexts.get(id as usize) {
            None => ShremdupReply::CreateCapture(Err("invalid id".to_string())),
            Some(ctx) => match ctx.shared_capturer(&name) {
              Err(err) => ShremdupReply::CreateCapture(Err(err)),
              Ok(capturer) => {
                capturer_map.insert(id, capturer);
                println!("CreateCapturer: id: {}, name: {}", id, name);
                ShremdupReply::CreateCapture(Ok(()))
              }
            },
          }
        }
      }
      ShremdupRequest::DeleteCapture(id) => {
        capturer_map.remove(&id);
        println!("DeleteCapturer: id: {}", id);
        ShremdupReply::DeleteCapture(Ok(()))
      }
      ShremdupRequest::TakeCapture(id) => match capturer_map.get_mut(&id) {
        None => ShremdupReply::TakeCapture(Err("invalid id".to_string())),
        Some(capturer) => match capturer.capture() {
          Err(err) => ShremdupReply::TakeCapture(Err(err)),
          Ok(capture) => {
            if capture.desktop_updated() {
              ShremdupReply::TakeCapture(Ok(true))
            } else {
              ShremdupReply::TakeCapture(Ok(false))
            }
          }
        },
      },
    };
    tx.send(reply).unwrap();
  }
}
