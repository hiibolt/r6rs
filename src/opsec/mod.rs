use crate::VecDeque;
use crate::Message;
use crate::Context;

pub async fn linked( ctx: &Context, msg: &Message, args: VecDeque<String> ) -> Result<String, String> {
    Ok(args.into_iter().collect::<Vec<String>>().join(" "))
}