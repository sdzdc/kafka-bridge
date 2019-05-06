use crate::socket::Socket;
use json::JsonValue;
use percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};
//use percent_encoding::percent_decode;

pub struct SubscribeClient {
    socket: Socket,
    messages: Vec<Message>,
    timetoken: String,
    channel: String,
    subscribe_key: String,
    _secret_key: String,
    agent: String,
}

pub struct PublishClient {
    socket: Socket,
    publish_key: String,
    subscribe_key: String,
    _secret_key: String,
    agent: String,
}

pub struct Message {
    pub channel: String,
    pub data: String,
    pub metadata: String,
    pub id: String,
}

#[derive(Debug)]
pub enum Error {
    Initialize,
    Publish,
    PublishWrite,
    PublishResponse,
    Subscribe,
    SubscribeWrite,
    SubscribeRead,
    MissingChannel,
    HTTPResponse,
}

fn http_response(socket: &mut Socket) -> Result<JsonValue, Error> {
    let mut body_length: usize = 0;
    loop {
        let data = match socket.readln() {
            Ok(data) => data,
            Err(_error) => return Err(Error::HTTPResponse),
        };

        // Capture Content Length of Payload
        if body_length == 0 && data.contains("Content-Length") {
            let result = match data.split_whitespace().nth(1) {
                Some(length) => length.parse(),
                None => return Err(Error::HTTPResponse),
            };
            body_length = match result {
                Ok(length) => length,
                Err(_error) => return Err(Error::HTTPResponse),
            };
        }

        // End of Headers
        if data.len() == 2 {
            let paylaod = match socket.read(body_length) {
                Ok(data) => data,
                Err(_error) => return Err(Error::HTTPResponse),
            };
            match json::parse(&paylaod) {
                Ok(response) => return Ok(response),
                Err(_error) => return Err(Error::HTTPResponse),
            };
        }
    }
}

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
/// # PubNub Client
///
/// This client lib offers publish/subscribe support to PubNub.
///
/// ```no_run
/// use wanbus::pubnub::Client;
///
/// let host = "psdsn.pubnub.com:80";
/// let channel = "demo";
/// let publish_key = "demo";
/// let subscribe_key = "demo";
/// let _secret_key = "secret";
/// let mut pubnub = Client::new(
///     host,
///     channel,
///     publish_key,
///     subscribe_key,
///     _secret_key,
///  ).expect("NATS Subscribe Client");
///
/// let result = pubnub.next_message();
/// assert!(result.is_ok());
/// let message = result.expect("Received Message");
/// println!("{} -> {}", message.channel, message.data);
/// ```
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
impl SubscribeClient {
    pub fn new(
        host: &str,
        channel: &str,
        subscribe_key: &str,
        _secret_key: &str,
    ) -> Result<Self, Error> {
        let agent = "PubNub-Subscribe-Client";
        let socket = Socket::new(host, agent, 30);

        let mut pubnub = Self {
            socket,
            messages: Vec::new(),
            channel: channel.into(),
            timetoken: "0".into(),
            subscribe_key: subscribe_key.into(),
            _secret_key: _secret_key.into(),
            agent: agent.into(),
        };

        match pubnub.subscribe() {
            Ok(()) => Ok(pubnub),
            Err(_error) => Err(Error::Subscribe),
        }
    }

    pub fn next_message(&mut self) -> Result<Message, Error> {
        // Return next saved mesasge
        if let Some(message) = self.messages.pop() {
            return Ok(message);
        }

        // Capture
        let response: JsonValue = match http_response(&mut self.socket) {
            Ok(data) => data,
            Err(_error) => {
                // Already returning an error, would you like another?
                self.subscribe().is_err();

                // Return first error
                return Err(Error::SubscribeRead);
            }
        };

        // Save Last Received Netwrok Queue ID
        self.timetoken = response["t"]["t"].to_string();

        // Capture Messages in Vec Buffer
        for message in response["m"].members() {
            self.messages.push(Message {
                channel: message["c"].to_string(),
                data: message["d"].to_string(),
                metadata: "TODO metadata".to_string(),
                id: message["p"]["t"].to_string(),
            });
        }

        // Ask for more messages from network
        match self.subscribe() {
            Ok(()) => self.next_message(),
            Err(_) => Err(Error::SubscribeRead),
        }
    }

    fn subscribe(&mut self) -> Result<(), Error> {
        // Don't subscribe if without a channel
        if self.channel.is_empty() {
            return Err(Error::MissingChannel);
        }
        let uri = format!(
            "/v2/subscribe/{subscribe_key}/{channel}/0/{timetoken}?pnsdk={agent}",
            subscribe_key = self.subscribe_key,
            channel = self.channel,
            timetoken = self.timetoken,
            agent = self.agent,
        );
        let request =
            format!("GET {} HTTP/1.1\r\nHost: pubnub\r\n\r\n", uri,);
        match self.socket.write(request) {
            Ok(_size) => Ok(()),
            Err(_error) => Err(Error::SubscribeWrite),
        }
    }
}

impl PublishClient {
    pub fn new(
        host: &str,
        publish_key: &str,
        subscribe_key: &str,
        _secret_key: &str,
    ) -> Result<Self, Error> {
        let agent = "PubNub-Publish-Client";
        let socket = Socket::new(host, agent, 5);

        Ok(Self {
            socket,
            publish_key: publish_key.into(),
            subscribe_key: subscribe_key.into(),
            _secret_key: _secret_key.into(),
            agent: agent.into(),
        })
    }

    pub fn publish(
        &mut self,
        channel: &str,
        message: &str,
    ) -> Result<String, Error> {
        let encoded_message =
            utf8_percent_encode(message, DEFAULT_ENCODE_SET).to_string();
        let uri = format!(
            "/publish/{}/{}/0/{}/0/{}?pnsdk={}",
            self.publish_key,
            self.subscribe_key,
            channel,
            encoded_message,
            self.agent
        );

        let request =
            format!("GET {} HTTP/1.1\r\nHost: pubnub\r\n\r\n", uri,);
        let _size = match self.socket.write(request) {
            Ok(size) => size,
            Err(_error) => return Err(Error::PublishWrite),
        };

        // Capture and return TimeToken
        let response: JsonValue = match http_response(&mut self.socket) {
            Ok(data) => data,
            Err(_error) => return Err(Error::PublishResponse),
        };
        Ok(response[2].to_string())
    }
}
