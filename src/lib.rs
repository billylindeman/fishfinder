pub mod sdr;
pub mod decode;


use crossbeam::thread;

pub trait SignalSrc<'env, Output> {
    fn produce(&self, scope: &thread::Scope<'env>) -> ringbuf::Consumer<Output>;
}

pub trait SignalSink<'env, Input> {
    fn consume(&self, src: ringbuf::Consumer<Input>);
}

pub trait SignalTransform<'env,Input,Output> {
    fn transform<'b>(&self, scope: &thread::Scope<'env>, src: ringbuf::Consumer<Input>) -> ringbuf::Consumer<Output>;
}