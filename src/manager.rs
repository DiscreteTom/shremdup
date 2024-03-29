use crate::model::{
  DisplayInfo, PointerPosition, PointerShape, ReplySender, RequestReceiver, ShremdupReply,
  ShremdupRequest,
};
use rusty_duplication::{
  capturer::{model::Capturer, shared::SharedCapturer},
  duplication_context::DuplicationContext,
  error::Error,
  manager::Manager,
  model::Result,
  utils::{FrameInfoExt, MonitorInfoExt},
};
use std::collections::HashMap;
use windows::Win32::Graphics::Dxgi::{DXGI_ERROR_WAIT_TIMEOUT, DXGI_OUTPUT_DESC};

pub async fn manager_thread(mut req_rx: RequestReceiver, res_tx: ReplySender) {
  let mut manager = Manager::new(0).unwrap();

  loop {
    manager.refresh().unwrap();
    println!("manager refreshed");
    let mut capturer_map: HashMap<u32, SharedCapturer> = HashMap::new();
    loop {
      let req = req_rx.recv().await.unwrap();
      let reply = match req {
        ShremdupRequest::Restart => ShremdupReply::Restart(Ok(())), // will break the loop below
        ShremdupRequest::ListDisplays => {
          let mut displays = Vec::new();
          manager
            .contexts
            .iter()
            .for_each(|ctx| match ctx.dxgi_output_desc() {
              Ok(dxgi_output_desc) => match get_display_info(&dxgi_output_desc, ctx) {
                Ok(info) => displays.push(info),
                Err(err) => println!("ListDisplays: {:?}", err),
              },
              Err(err) => println!("ListDisplays: {:?}", err),
            });
          ShremdupReply::ListDisplays(Ok(displays))
        }
        ShremdupRequest::GetDisplay(id) => match manager.contexts.get(id as usize) {
          None => ShremdupReply::GetDisplay(Err(Error::new("invalid id"))),
          Some(ctx) => match ctx.dxgi_output_desc() {
            Ok(dxgi_output_desc) => match get_display_info(&dxgi_output_desc, ctx) {
              Ok(info) => ShremdupReply::GetDisplay(Ok(info)),
              Err(err) => ShremdupReply::GetDisplay(Err(err)),
            },
            Err(err) => ShremdupReply::GetDisplay(Err(err)),
          },
        },
        ShremdupRequest::CreateCapture(id, name, open) => {
          if capturer_map.contains_key(&id) {
            ShremdupReply::CreateCapture(Err(Error::new("already exists")))
          } else {
            match manager.contexts.get(id as usize) {
              None => ShremdupReply::CreateCapture(Err(Error::new("invalid id"))),
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
                  println!("CreateCapturer: id: {}, name: {}, open: {}", id, name, open);
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
          None => ShremdupReply::TakeCapture(Err(Error::new("invalid id"))),
          Some(capturer) => match capturer.safe_capture_with_pointer_shape() {
            Err(err) => {
              if err.windows.is_some()
                && err.windows.as_ref().unwrap().code() == DXGI_ERROR_WAIT_TIMEOUT
              {
                // if error is timeout, return ok
                ShremdupReply::TakeCapture(Ok((false, None, None)))
              } else {
                ShremdupReply::TakeCapture(Err(err))
              }
            }
            Ok((frame_info, pointer_shape_info)) => {
              if !frame_info.mouse_updated().position_updated {
                ShremdupReply::TakeCapture(Ok((frame_info.desktop_updated(), None, None)))
              } else {
                ShremdupReply::TakeCapture(Ok((
                  frame_info.desktop_updated(),
                  Some(PointerPosition {
                    visible: frame_info.PointerPosition.Visible.as_bool(),
                    x: frame_info.PointerPosition.Position.x,
                    y: frame_info.PointerPosition.Position.y,
                  }),
                  match pointer_shape_info {
                    None => None,
                    Some(pointer_shape_info) => Some(PointerShape {
                      shape_type: pointer_shape_info.Type,
                      width: pointer_shape_info.Width,
                      height: pointer_shape_info.Height,
                      pitch: pointer_shape_info.Pitch,
                      data: capturer.pointer_shape_buffer().to_vec(),
                    }),
                  },
                )))
              }
            }
          },
        },
      };
      let restart = match &reply {
        ShremdupReply::Restart(Ok(())) => true,
        _ => false,
      };
      res_tx.send(reply).await.unwrap();
      if restart {
        break;
      }
    }
  }
}

fn get_display_info(
  dxgi_output_desc: &DXGI_OUTPUT_DESC,
  ctx: &DuplicationContext,
) -> Result<DisplayInfo> {
  let dxgi_outdupl_desc = ctx.dxgi_outdupl_desc();
  Ok(DisplayInfo {
    bottom: dxgi_output_desc.DesktopCoordinates.bottom,
    top: dxgi_output_desc.DesktopCoordinates.top,
    left: dxgi_output_desc.DesktopCoordinates.left,
    right: dxgi_output_desc.DesktopCoordinates.right,
    name: {
      let mut real_len = dxgi_output_desc.DeviceName.len();
      for i in 0..dxgi_output_desc.DeviceName.len() {
        let c = dxgi_output_desc.DeviceName[i];
        // c-string is terminated by 0
        if c == 0 {
          real_len = i;
          break;
        }
      }
      String::from_utf16_lossy(&dxgi_output_desc.DeviceName[..real_len])
    },
    rotation: dxgi_output_desc.Rotation.0,
    pixel_width: dxgi_outdupl_desc.ModeDesc.Width,
    pixel_height: dxgi_outdupl_desc.ModeDesc.Height,
    is_primary: ctx.monitor_info()?.is_primary(),
  })
}
