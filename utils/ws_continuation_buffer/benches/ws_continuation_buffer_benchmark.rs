
use criterion::{criterion_group, criterion_main, Criterion};
use ws_continuation_buffer::*;
use bytes::Bytes;

pub fn criterion_benchmark(c: &mut Criterion) {
    let frame_message_counts = vec![3, 5, 10];
    let mut g = c.benchmark_group("ws_continuation_buffer");

    for count in frame_message_counts {
        let with_count = format!("count={}", count);
        g.bench_function(&with_count, |b| b.iter(|| {
            let mut ws_buffer = WsContinuationBuffer::default();
            let bytes: Vec<u8> = (0..100).into_iter().map(|_i| 1u8).collect();
            let bytes = Bytes::from(bytes);
            for _ in 0..1000 {
                for step in 1..=count {
                    if step == 1 {
                        ws_buffer.handle_msg(ContinuationFrameItem::FirstBinary(bytes.clone())).unwrap();
                    } else if step != count {
                        ws_buffer.handle_msg(ContinuationFrameItem::Continue(bytes.clone())).unwrap();
                    } else if step == count {
                        ws_buffer.handle_msg(ContinuationFrameItem::Last(bytes.clone())).unwrap();
                    }                
                }
            }
        }));
    }
    g.finish();    
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
