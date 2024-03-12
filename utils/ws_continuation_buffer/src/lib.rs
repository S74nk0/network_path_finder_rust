use bytes::{BufMut, Bytes, BytesMut};
use thiserror::Error;

const DEFAULT_RESERVED_BYTES: usize = 4096;

/// A WebSocket continuation frame item.
#[derive(Debug)]
pub enum ContinuationFrameItem {
    FirstText(Bytes),
    FirstBinary(Bytes),
    Continue(Bytes),
    Last(Bytes),
}

/// The result of [WsContinuationBuffer](WsContinuationBuffer)
#[derive(Debug)]
pub enum HandledItem {
    Continue,
    Binary(Vec<u8>),
    Text(String),
}

#[derive(Error, Debug)]
pub enum WsContinuationBufferError {
    #[error("Invalid transition 'First' to 'Continue'")]
    TransitionFromFirstToContinue,
    #[error("Invalid transition 'First' to 'Last'")]
    TransitionFromFirstToLast,
    #[error("Invalid transition 'Continue' to 'FirstText'")]
    TransitionFromContinueToFirstText,
    #[error("Invalid transition 'Continue' to 'FirstBinary'")]
    TransitionFromContinueToFirstBinary,
    #[error(transparent)]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
}

enum FirstType {
    Binary,
    Text,
}

impl Default for FirstType {
    fn default() -> Self {
        Self::Binary
    }
}
enum BufferTransitionState {
    AcceptingFirstItem,  
    AcceptingContinueOrLast(FirstType),
}

/// A helper struct used for handling WebSocket continuation frames.
pub struct WsContinuationBuffer {
    state: BufferTransitionState,
    bytes: BytesMut,
}

impl Default for WsContinuationBuffer {
    fn default() -> Self {
        Self {
            state: BufferTransitionState::AcceptingFirstItem,
            bytes: BytesMut::with_capacity(DEFAULT_RESERVED_BYTES),
        }
    }
}

impl WsContinuationBuffer {

    #[inline(always)]
    fn clear(&mut self) {
        self.state = BufferTransitionState::AcceptingFirstItem;
        self.bytes.clear();
    }

    pub fn handle_msg(
        &mut self,
        item: ContinuationFrameItem,
    ) -> Result<HandledItem, WsContinuationBufferError> {
        use BufferTransitionState::*;
        use ContinuationFrameItem::*;
        use WsContinuationBufferError::*;

        match (&mut self.state, item) {
            (AcceptingFirstItem, FirstText(b)) => {
                self.state = AcceptingContinueOrLast(FirstType::Text);
                self.bytes.put(b);
                Ok(HandledItem::Continue)
            }
            (AcceptingFirstItem, FirstBinary(b)) => {
                self.state = AcceptingContinueOrLast(FirstType::Binary);
                self.bytes.put(b);
                Ok(HandledItem::Continue)
            }
            (AcceptingFirstItem, Continue(_)) => {
                self.clear();
                Err(TransitionFromFirstToContinue)
            }
            (AcceptingFirstItem, Last(_)) => {
                self.clear();
                Err(TransitionFromFirstToLast)
            }
            (AcceptingContinueOrLast(_), FirstText(_)) => {
                self.clear();
                Err(TransitionFromContinueToFirstText)
            }
            (AcceptingContinueOrLast(_), FirstBinary(_)) => {
                self.clear();
                Err(TransitionFromContinueToFirstBinary)
            }
            (AcceptingContinueOrLast(_), Continue(b)) => {
                self.bytes.put(b);
                Ok(HandledItem::Continue)
            }
            (AcceptingContinueOrLast(return_type), Last(b)) => {
                self.bytes.put(b);
                let full_message_bytes = self.bytes.to_vec();                
                let return_type = std::mem::take(return_type);
                self.clear();
                
                match return_type {
                    FirstType::Binary => Ok(HandledItem::Binary(full_message_bytes)),
                    FirstType::Text => {
                        Ok(HandledItem::Text(String::from_utf8(full_message_bytes)?))
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_continuation_buffer() {
        use ContinuationFrameItem::*;
        use WsContinuationBufferError::*;

        let mut buffer = WsContinuationBuffer::default();

        // error cases
        let first_error_item = Continue("starting with Continue".as_bytes().into());
        assert!(matches!(
            buffer.handle_msg(first_error_item),
            Err(TransitionFromFirstToContinue)
        ));

        let first_error_item = Last("starting with Last".into());
        assert!(matches!(
            buffer.handle_msg(first_error_item),
            Err(TransitionFromFirstToLast)
        ));

        let item = FirstBinary("First Binary".into());
        assert!(matches!(buffer.handle_msg(item), Ok(HandledItem::Continue)));
        let item = FirstBinary("First Binary Again".into());
        assert!(matches!(
            buffer.handle_msg(item),
            Err(TransitionFromContinueToFirstBinary)
        ));

        let item = FirstText("First Text".into());
        assert!(matches!(buffer.handle_msg(item), Ok(HandledItem::Continue)));
        let item = FirstText("First Text Again".into());
        assert!(matches!(
            buffer.handle_msg(item),
            Err(TransitionFromContinueToFirstText)
        ));

        let bytes_in_order: Vec<u8> = (0u8..255u8).into_iter().collect();
        let chunks = bytes_in_order.chunks(10);
        assert!(
            chunks.len() >= 3,
            "We must have at least 3 chunks to execute this test"
        );
        let last_index = chunks.len() - 1;
        for (idx, chunk) in chunks.enumerate() {
            let bytes = chunk.to_owned().into();
            match (idx == 0, idx == last_index) {
                (true, _) => {
                    let r = buffer.handle_msg(FirstBinary(bytes));
                    assert!(matches!(r, Ok(HandledItem::Continue)));
                }
                (_, false) => {
                    let r = buffer.handle_msg(Continue(bytes));
                    assert!(matches!(r, Ok(HandledItem::Continue)));
                }
                (_, true) => {
                    let r = buffer.handle_msg(Last(bytes));
                    assert!(matches!(r,
                        Ok(HandledItem::Binary(vec_result)) if vec_result.eq(&bytes_in_order)));
                }
            }
        }

        let cont = buffer.handle_msg(FirstText(Bytes::from_static(b"first")));
        assert!(matches!(cont, Ok(HandledItem::Continue)));
        let cont = buffer.handle_msg(Continue(Bytes::from_static(b"continue")));
        assert!(matches!(cont, Ok(HandledItem::Continue)));
        let last = buffer.handle_msg(Last(Bytes::from_static(b"last")));
        assert!(matches!(last, Ok(HandledItem::Text(text)) if text.eq("firstcontinuelast")));
    }

}
