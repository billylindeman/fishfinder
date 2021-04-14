pub mod sdr;

use crossbeam::thread;

pub trait SignalSrc<'env, Output> {
    fn produce(&self, scope: &thread::Scope<'env>) -> ringbuf::Consumer<Output>;
}

pub trait SignalSink<'env, Input> {
    fn consume(src: ringbuf::Consumer<Input>);
}

pub trait SignalTransform<'env, Input, Output> {
    fn transform<'b>(
        scope: &thread::Scope<'env>,
        src: ringbuf::Consumer<Input>,
    ) -> ringbuf::Consumer<Output>;
}

