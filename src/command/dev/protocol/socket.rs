use crate::Result;

use interprocess::local_socket::LocalSocketStream;
use saucer::{anyhow, Context, Error};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    fmt::Debug,
    io::{self, BufRead, BufReader, Write},
};

pub(crate) fn handle_socket_error(
    conn: io::Result<LocalSocketStream>,
) -> Option<LocalSocketStream> {
    match conn {
        Ok(val) => Some(val),
        Err(error) => {
            eprintln!("incoming connection failed: {}", error);
            None
        }
    }
}

pub(crate) fn socket_read<B>(
    stream: &mut BufReader<LocalSocketStream>,
) -> std::result::Result<B, saucer::Error>
where
    B: Serialize + DeserializeOwned + Debug,
{
    let mut incoming_message = String::new();

    match stream.read_line(&mut incoming_message) {
        Ok(_) => {
            if incoming_message.is_empty() {
                Err(anyhow!("incoming message was empty"))
            } else {
                let incoming_message: B =
                    serde_json::from_str(&incoming_message).with_context(|| {
                        format!(
                            "incoming message '{}' was not valid JSON",
                            &incoming_message
                        )
                    })?;
                Ok(incoming_message)
            }
        }
        Err(e) => Err(Error::new(e).context("could not read incoming message")),
    }
}

pub(crate) fn socket_write<A>(message: &A, stream: &mut BufReader<LocalSocketStream>) -> Result<()>
where
    A: Serialize + DeserializeOwned + Debug,
{
    let outgoing_json = serde_json::to_string(message)
        .with_context(|| format!("could not convert outgoing message {:?} to json", &message))?;
    let outgoing_string = format!("{}\n", &outgoing_json);
    stream
        .get_mut()
        .write_all(outgoing_string.as_bytes())
        .with_context(|| {
            format!(
                "could not write outgoing message {:?} to socket",
                &outgoing_json
            )
        })?;
    Ok(())
}
