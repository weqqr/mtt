use mtt_macros::packet;

#[packet]
#[derive(Debug)]
pub enum ClientBound {
    #[id = 0x00]
    Hello {
        test: i32,
    },
}
