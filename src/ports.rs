#[derive(Clone)]
pub struct ProtoPort {
    pub proto: String,
    pub port: u16,
}

impl ProtoPort {
    pub fn new(proto: String, port: u16) -> Self {
        return Self { proto, port };
    }
}
