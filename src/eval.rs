extern crate alloc;

use core::{error::Error, fmt, str::FromStr};

use alloc::{string::String, vec::Vec};
use defmt::{error, info, Format, Formatter};
use miniarg::{ArgumentIterator, Key, ParseError};
use rtt_target::{DownChannel, UpChannel};
fn dummy_blink() {
    info!("toggle");
}
fn dummy_say_hello() {
    info!("hello");
}
pub struct RttCmdHandler<'a> {
    pub up_chan: UpChannel,
    down_chan: DownChannel,
    rx_buf: [u8; 128],
    cmds: Vec<Cmd<'a>>,
}

impl<'a> RttCmdHandler<'a> {
    pub fn new(up_chan: UpChannel, down_chan: DownChannel) -> Self {
        let rx_buf = [0u8; 128];

        let cmds = [
            Cmd {
                kind: RttCmd::ToggleLed,
                help_txt: "Toggle the LED",
                callback: dummy_blink,
            },
            Cmd {
                kind: RttCmd::SayHello,
                help_txt: "Print 'hello'",
                callback: dummy_say_hello,
            },
        ]
        .to_vec();

        Self {
            up_chan,
            down_chan,
            rx_buf,
            cmds,
        }
    }

    pub fn process_incoming_cmds(&mut self) {
        let bytes_read = self.down_chan.read(&mut self.rx_buf);
        if bytes_read > 0 {
            let bytes = self.rx_buf[..bytes_read].to_vec();

            let cmdline = match String::from_utf8(bytes) {
                Ok(str) => str,
                Err(_) => {
                    error!("error parsing utf8)");
                    return;
                }
            };

            let cmd = match cmdline.split_whitespace().next() {
                Some(cmd) => cmd,
                None => return,
            };

            let args = match RttKeys::parse(&cmdline).collect::<Result<Vec<_>, _>>() {
                Ok(vec) => vec,
                Err(e) => {
                    error!("Argument error: {}", RttParseError(e));
                    return;
                }
            };

            info!("{}", cmd);
            for arg in args.iter() {
                info!("{}", arg);
            }
        }
    }
}

#[derive(Debug, defmt::Format, Clone)]
struct Cmd<'a> {
    kind: RttCmd,
    help_txt: &'a str,
    callback: fn(),
}

impl<'a> Cmd<'a> {
    fn new(help_txt: &'a str, callback: fn(), kind: RttCmd) -> Self {
        Self {
            help_txt,
            callback,
            kind,
        }
    }
}

impl<'a> FromStr for Cmd<'a> {
    type Err = RttCmdErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "led" => Ok(Self {
                kind: RttCmd::ToggleLed,
                help_txt: "toggle the led",
                callback: dummy_blink,
            }),
            "hello" => Ok(Self {
                kind: RttCmd::SayHello,
                help_txt: "say hello",
                callback: dummy_say_hello,
            }),
            _ => Err("Invalid cmd"),
        }
    }
}

#[derive(Debug, defmt::Format, Clone)]
enum RttCmd {
    ToggleLed,
    SayHello,
}

#[derive(Debug, Key, PartialEq, defmt::Format)]
enum RttKeys {
    /// This key Foos
    Foo,

    /// This key Bars
    Bar,
}

#[derive(Debug)]
struct RttParseError<'a>(ParseError<'a>);

impl<'a> Format for RttParseError<'a> {
    fn format(&self, fmt: Formatter) {
        defmt::write!(
            fmt,
            "{}",
            match self.0 {
                ParseError::NotAKey(_) => "Not a key",
                ParseError::UnknownKey(_) => "Unknown key",
                _ => "An unknown error occured parsing the key",
            }
        );
    }
}
