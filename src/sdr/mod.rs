pub mod crc;
pub mod dsp;
pub mod mode_s;
pub mod rtl;

//pub struct FileSDR {
//    pub path: String,
//}
//
//impl<'env> SignalSrc<'env, u8> for FileSDR {
//    fn produce(&self, scope: &thread::Scope<'env>) -> ringbuf::Consumer<u8> {
//        debug!("starting FileSDR with {}", self.path);
//
//        // setup iq sample buffer
//        let iq_buffer = RingBuffer::<u8>::new(IQ_SAMPLE_CAPACITY);
//        let (mut iq_producer, iq_consumer) = iq_buffer.split();
//
//        let file = File::open(&self.path).unwrap();
//        let mut reader = BufReader::new(file);
//
//        scope.spawn(move |_| loop {
//            match iq_producer.read_from(&mut reader, Some(500)) {
//                Ok(count) => trace!("read {} samples from dump", count),
//                Err(e) => {
//                    error!("error reading from dump: {:?}", e);
//                    break;
//                }
//            }
//            std::thread::sleep(time::Duration::from_millis(10))
//        });
//
//        iq_consumer
//    }
//}
//
