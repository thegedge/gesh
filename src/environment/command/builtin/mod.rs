//! Support for builtins commands.
//!
macro_rules! builtin {
    ( $name:ident, $self:ident $body:block) => {
        use environment::{
            command::{
                Command,
                Error,
                ExitStatus,
            },
            Environment
        };

        pub struct $name<'e> {
            env: Option<&'e mut Environment>,
            args: Vec<String>,
        }

        impl <'e> $name<'e> {
            pub fn new() -> $name<'e> {
                $name {
                    env: None,
                    args: vec![],
                }
            }
        }

        impl <'e> Command<'e> for $name<'e> {
            fn args(&mut $self, args: Vec<String>) -> &mut Self {
                $self.args = args;
                $self
            }

            fn env<'v: 'e>(&mut $self, env: &'v mut Environment) -> &mut Self {
                $self.env = Some(env);
                $self
            }

            fn execute(&mut $self) -> Result<ExitStatus, Error> {
                $body
            }
        }
    };
}

mod cd;

pub use self::{
    cd::Cd,
};
