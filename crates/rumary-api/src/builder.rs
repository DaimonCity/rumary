use crate::util;
use tokio::process::Command;

pub struct Builder {
    domain: String,
    cmd: Command,
}

impl Builder {
    async fn new() -> Self {
        let domain = util::get_domain().await.unwrap();
        let mut cmd = Command::new("cargo");
        cmd.arg("build");
        cmd.arg("--release");

        Self { domain, cmd }
    }

    async fn build(&mut self, os: Os) {
        match os {
            Os::Windows => {
                self.windows_build().await;
            }
            Os::Linux => {
                self.linux_build().await;
            }
            Os::MacOS => {
                self.macos_build().await;
            }
        }
    }

    async fn windows_build(&mut self) {
        let cmd = &mut self.cmd;
        cmd.arg("target=x86_64-pc-windows-msvc");
    }

    async fn linux_build(&mut self) {
        let cmd = &mut self.cmd;
        cmd.arg("target=x86_64-unknown-linux-gnu");
    }

    async fn macos_build(&mut self) {
        let cmd = &mut self.cmd;
        cmd.arg("target=x86_64-apple-darwin");
    }
}

pub enum Os {
    Windows,
    Linux,
    MacOS,
}
