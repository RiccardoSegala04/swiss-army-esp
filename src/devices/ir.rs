enum InfraredCommand {}

#[derive(Clone)]
pub struct IrSignal<'a> {
    pub timings: &'a [u16],
}
