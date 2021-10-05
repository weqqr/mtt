use mtt_macros::packet;

#[packet]
pub enum ServerBound {
    #[id = 0x00]
    Hello {
        test: i32,
    },
}
