use command::{
    Context,
    ExitStatus,
    Result,
};

pub fn exit(Context { args, .. }: Context) -> Result {
    let status = match args.into_iter().next() {
        Some(status) => status.parse().unwrap_or(255),
        None => 0,
    };
    Ok(ExitStatus::ExitWith(status))
}
