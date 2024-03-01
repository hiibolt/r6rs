use crate::VecDeque;
use crate::Message;
use crate::Context;

pub async fn linked( _ctx: &Context, _msg: &Message, args: VecDeque<String> ) -> Result<String, String> {
    Ok(args.into_iter().collect::<Vec<String>>().join(" "))
}