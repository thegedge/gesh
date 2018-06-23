builtin!(
    Exit,
    self {
        let status = match self.args.get(0) {
            Some(status) => status.parse().unwrap_or(255),
            None => 0,
        };
        Ok(ExitStatus::ExitWith(status))

    }
);
