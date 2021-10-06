use mtt_macros::packet;

#[packet]
#[derive(Debug)]
pub enum ServerBound {
    #[id = 0x00]
    Hello {
        test: i32,
    },
}
