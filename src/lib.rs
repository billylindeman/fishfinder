pub mod sdr;
pub mod decode;



pub trait SignalSrc<Output> {
    fn produce(&self) -> ringbuf::Consumer<Output>;
}

pub trait SignalSink<Input> {
    fn consume(&self, src: &'static mut ringbuf::Consumer<Input>);
}

pub trait SignalTransform<Input,Output> {
    fn transform(&self, src: &'static mut ringbuf::Consumer<Input>) -> ringbuf::Consumer<Output>;
}