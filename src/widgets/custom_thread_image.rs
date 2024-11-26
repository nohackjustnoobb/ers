// Source: https://github.com/benjajaja/ratatui-image/blob/master/src/thread/mod.rs
// Changed to fit into this project

//! Widget that separates resize+encode from rendering.
//! This allows for rendering to be non-blocking, offloading resize+encode into another thread.
//! See examples/async.rs for how to setup the threads and channels.
//! At least one worker thread for resize+encode is required, the example shows how to combine
//! the needs-resize-polling with other terminal events into one event loop.

use std::sync::mpsc::Sender;

use ratatui::{
    prelude::{Buffer, Rect},
    widgets::StatefulWidget,
};
use ratatui_image::{protocol::StatefulProtocol, Resize};

/// A widget that uses a custom ThreadProtocol as state to offload resizing and encoding to a
/// background thread.
pub struct ThreadImage {
    resize: Resize,
    id: String,
}

impl ThreadImage {
    pub fn resize(mut self, resize: Resize) -> ThreadImage {
        self.resize = resize;
        self
    }

    pub fn new(id: String) -> Self {
        ThreadImage {
            resize: Resize::Fit(None),
            id,
        }
    }
}

impl StatefulWidget for ThreadImage {
    type State = ThreadProtocol;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.inner = match state.inner.take() {
            // We have the `protocol` and should either resize or render.
            Some(mut protocol) => {
                // If it needs resizing (grow or shrink) then send it away instead of rendering.
                // Send the requested area instead of the calculated area
                // to ensure consistent calculations between the render thread and the UI thread.
                if protocol.needs_resize(&self.resize, area).is_some() {
                    state
                        .tx
                        .send((self.id, protocol, self.resize, area))
                        .unwrap();
                    None
                } else {
                    protocol.render(area, buf);
                    Some(protocol)
                }
            }
            // We are waiting to get back the protocol.
            None => None,
        };
    }
}

/// The state of a ThreadImage.
///
/// Has `inner` [ResizeProtocol] that is sent off to the `tx` mspc channel to do the
/// `resize_encode()` work.
pub struct ThreadProtocol {
    inner: Option<StatefulProtocol>,
    tx: Sender<(String, StatefulProtocol, Resize, Rect)>,
}

impl ThreadProtocol {
    pub fn new(
        tx: Sender<(String, StatefulProtocol, Resize, Rect)>,
        inner: StatefulProtocol,
    ) -> ThreadProtocol {
        ThreadProtocol {
            inner: Some(inner),
            tx,
        }
    }
    pub fn set_protocol(&mut self, proto: StatefulProtocol) {
        self.inner = Some(proto);
    }
}
