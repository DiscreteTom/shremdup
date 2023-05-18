use crate::model::{
  DxgiOutputDescExt, PointerPosition, PointerShape, ReplyReceiver, ShremdupReply, ShremdupRequest,
};
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
      ShremdupRequest::CreateCapture(id, name, open) => {
        if capturer_map.contains_key(&id) {
          ShremdupReply::CreateCapture(Err("already exists".to_string()))
        } else {
          match manager.contexts.get(id as usize) {
            None => ShremdupReply::CreateCapture(Err("invalid id".to_string())),
            Some(ctx) => match {
              if open {
                ctx.shared_capturer_open(&name)
              } else {
                ctx.shared_capturer(&name)
              }
            } {
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
        Some(capturer) => match capturer.safe_capture_with_pointer_shape() {
          Err(err) => ShremdupReply::TakeCapture(Err(err)),
          Ok((frame_info, pointer_shape_info)) => {
            if !frame_info.mouse_updated() {
              ShremdupReply::TakeCapture(Ok((frame_info.desktop_updated(), None, None)))
            } else {
              let pointer_shape_info = pointer_shape_info.unwrap();
              ShremdupReply::TakeCapture(Ok((
                frame_info.desktop_updated(),
                Some(PointerPosition {
                  visible: frame_info.PointerPosition.Visible.as_bool(),
                  x: frame_info.PointerPosition.Position.x,
                  y: frame_info.PointerPosition.Position.y,
                }),
                Some(PointerShape {
                  shape_type: pointer_shape_info.Type,
                  width: pointer_shape_info.Width,
                  height: pointer_shape_info.Height,
                  pitch: pointer_shape_info.Pitch,
                  data: capturer.pointer_shape_buffer().to_vec(),
                }),
              )))
            }
          }
        },
      },
    };
    tx.send(reply).unwrap();
  }
}
