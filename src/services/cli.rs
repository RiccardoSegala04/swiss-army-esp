use defmt::info;

use embassy_net::{
    IpListenEndpoint, Ipv4Cidr, Runner, Stack, StackResources, StaticConfigV4,
    tcp::{TcpSocket, TcpWriter},
};

use heapless::String;

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::channel::{self, DynamicReceiver, DynamicSender};

use embassy_futures::select::{Either, select};

use embedded_io_async::Write;

use crate::devices::ir::{InfraredCommand, InfraredEvent};
use crate::services::router::{RouterCommand, RouterEvent};

use embedded_cli::{
    Command, CommandGroup,
    cli::{Cli, CliBuilder},
    writer::EmptyWriter,
};

use core::convert::Infallible;

pub static EVENT_CHANNEL: Channel<CriticalSectionRawMutex, RouterEvent, 8> = Channel::new();

const LOGO: &'static str = r#"
                                      .:^
               ^                     /   :
  '`.        /;/                    /    /
  \  \      /;/                    /    /
   \\ \    /;/                    /  ///
    \\ \  /;/                    /  ///
     \  \/_/____________________/    /
      `/                         \  /
      {  - Swiss Army Esp  CLI -  }'
       \_________________________/
  
"#;

#[derive(Command, Clone)]
enum Ir {
    Record,
    List,
    Transmit { idx: u8 },
}

#[derive(Command, Clone)]
enum Base {
    /// Say hello to World or someone else
    Hello {
        /// To whom to say hello (World by default)
        n: u8,
    },

    Ir {
        #[command(subcommand)]
        ir: Ir,
    },

    /// Stop CLI and exit
    Exit,
}

pub struct CliService<'a> {
    commands_sender: DynamicSender<'static, RouterCommand>,
    events_receiver: DynamicReceiver<'static, RouterEvent>,
    stack: Stack<'a>,
    cli: Cli<EmptyWriter, Infallible, [u8; 40], [u8; 100]>,
}

impl<'a> CliService<'a> {
    pub fn new(commands_sender: DynamicSender<'static, RouterCommand>, stack: Stack<'a>) -> Self {
        Self {
            commands_sender,
            events_receiver: EVENT_CHANNEL.dyn_receiver(),
            stack,
            cli: CliBuilder::default().build().unwrap(),
        }
    }

    fn handle_user_input(&mut self, buffer: &[u8]) -> Option<Base> {
        let mut command: Option<Base> = None;
        for byte in buffer {
            let _ = self.cli.process_byte::<Base, _>(
                *byte,
                &mut Base::processor(|cli, comm| {
                    command = Some(comm.clone());
                    Ok(())
                }),
            );
        }

        command
    }

    async fn handle_ir_command(&self, socket: &mut TcpSocket<'_>, command: Ir) {
        match command {
            Ir::Record => {
                self.commands_sender
                    .send(RouterCommand::InfraredCommand(InfraredCommand::Listen))
                    .await;

                socket.write_all(b"Listening...\n").await;

                loop {
                    let ev = self.events_receiver.receive().await;

                    if let RouterEvent::InfraredEvent(ir) = ev {
                        match ir {
                            InfraredEvent::SignalTooLong => {
                                socket.write_all(b"Signal was too long\n").await;
                                break;
                            }
                            InfraredEvent::NoSignal => {
                                socket.write_all(b"No signal detected\n").await;
                                break;
                            }
                            InfraredEvent::Signal(_) => {
                                socket.write_all(b"Signal recorded\n").await;
                                break;
                            }
                            _ => {}
                        }
                    }
                }
            }

            Ir::List => {
                let len = crate::devices::ir::SIGNAL_HISTORY.get().lock().await.len();
                for i in 0..len {
                    socket.write_all(b"X ").await;
                }
                socket.write_all(b"\n").await;
            }

            Ir::Transmit { idx } => {
                let sig = crate::devices::ir::SIGNAL_HISTORY
                    .get()
                    .lock()
                    .await
                    .get(idx as usize)
                    .cloned();

                if let Some(sig) = sig {
                    self.commands_sender
                        .send(RouterCommand::InfraredCommand(InfraredCommand::Play(sig)))
                        .await;

                    loop {
                        let ev = self.events_receiver.receive().await;

                        if let RouterEvent::InfraredEvent(InfraredEvent::SignalPlayed) = ev {
                            socket.write_all(b"Signal transmitted\n").await;
                            break;
                        }
                    }
                } else {
                    socket.write_all(b"Invalid index\n").await;
                }
            }
        }
    }

    async fn handle_command(&self, socket: &mut TcpSocket<'_>, command: Option<Base>) {
        if let Some(c) = command {
            match c {
                Base::Hello { n } => {
                    for i in 0..n {
                        let _ = socket.write_all(b"Hello\n").await;
                    }
                }

                Base::Ir { ir } => self.handle_ir_command(socket, ir).await,

                Base::Exit => {
                    let _ = socket.write_all(b"Exit\n").await;
                    socket.close();
                }
            }
        }
    }

    pub async fn run(&mut self) -> ! {
        let mut rx_buffer = [0; 1536];
        let mut tx_buffer = [0; 1536];

        loop {
            let mut socket = TcpSocket::new(self.stack, &mut rx_buffer, &mut tx_buffer);

            let r = socket
                .accept(IpListenEndpoint {
                    addr: None,
                    port: 8080,
                })
                .await;

            if let Err(e) = r {
                info!("connect error: {:?}", e);
                continue;
            }

            socket.write_all(LOGO.as_bytes()).await;

            let mut buffer: [u8; 128] = [0; 128];

            socket.write_all(b"$ ").await;

            loop {
                let ev = self.events_receiver.receive();
                let input = socket.read(&mut buffer);

                match select(ev, input).await {
                    Either::First(_) => {}
                    Either::Second(input) => {
                        match input {
                            Ok(len) => {
                                if len > 0 {
                                    let command = self.handle_user_input(&mut buffer[0..len]);
                                    self.handle_command(&mut socket, command).await;
                                    socket.write_all(b"$ ").await;
                                } else {
                                    break;
                                }
                            }
                            Err(e) => {
                                info!("read error: {:?}", e);
                                break;
                            }
                        };
                    }
                }
            }
        }
    }

    pub fn event_sender() -> DynamicSender<'static, RouterEvent> {
        EVENT_CHANNEL.dyn_sender()
    }
}
