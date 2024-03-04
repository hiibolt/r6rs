use crate::VecDeque;
use crate::Message;
use crate::Context;
use crate::unimplemented;

pub async fn help( ctx: Context, msg: Message, _args: VecDeque<String> ) {
    unimplemented( ctx, msg, "help" ).await;
}