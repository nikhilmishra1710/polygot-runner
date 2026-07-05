use std::process::{Child, ChildStderr, ChildStdin, ChildStdout, ExitStatus};

pub struct RunningProcess {
    child: Child,
}

impl RunningProcess {
    pub fn new(child: Child) -> Self {
        Self { child }
    }

    pub fn id(&self) -> u32 {
        self.child.id()
    }

    pub fn try_wait(&mut self) -> std::io::Result<Option<ExitStatus>> {
        self.child.try_wait()
    }

    pub fn wait(&mut self) -> std::io::Result<ExitStatus> {
        self.child.wait()
    }

    pub fn take_stdout(&mut self) -> Option<ChildStdout> {
        self.child.stdout.take()
    }

    pub fn take_stderr(&mut self) -> Option<ChildStderr> {
        self.child.stderr.take()
    }

    pub fn take_stdin(&mut self) -> Option<ChildStdin> {
        self.child.stdin.take()
    }

    pub fn write_stdin(&mut self, input: &[u8]) -> std::io::Result<()> {
        use std::io::Write;

        if let Some(mut stdin) = self.child.stdin.take() {
            stdin.write_all(input)?;
        }

        Ok(())
    }
}
