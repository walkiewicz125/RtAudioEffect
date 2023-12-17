use super::stream::Stream;

pub trait Efect {
    fn connect(stream: &dyn Stream);
}
