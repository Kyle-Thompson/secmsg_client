use std::net::{UdpSocket, Ipv4Addr, SocketAddr};
use std::thread::{self, JoinHandle};
use std::collections::{VecDeque};
use std::sync::{Arc, Mutex, Condvar};
use std::sync::mpsc::{channel, Sender};

const SERVER_ADDR: &'static str = "159.203.57.173:5000";

pub enum MessageType { // TODO: fill in later
    Ack,
    Authenticate {
        username: String,
        password: String
    },
}

pub struct Message {
    msg_type: MessageType,
    destination: Vec<String>,
    // signature
    // id
}

struct MessageContainer {
    msg: Message,
    callback: Option<Sender<Message>>,
}

#[derive(Clone)]
pub struct Net {
    work: Arc<(Mutex<VecDeque<MessageContainer>>, Condvar)>,
}

impl Net {

    pub fn new() -> Net {

        let mut net = Net {
            work: Arc::new( (Mutex::new(VecDeque::new()), Condvar::new()) )
        };
        
        // senders
        for i in 0..16 {
            let send_net = net.clone();
            thread::spawn(move|| {
                let mut socket = UdpSocket::bind("127.0.0.1:0").expect("Couldn't bind socket!");
                let mut element: Option<MessageContainer> = None;

                loop {
                    // grab message from queue
                    let &(ref queue, ref cvar) = &*send_net.work;
                    let (msg, callback) = match {
                        let mut queue = queue.lock().unwrap();
                        while !queue.is_empty() { queue = cvar.wait(queue).unwrap(); }
                        queue.pop_front()
                    } {
                        Some(MessageContainer{msg: m, callback: c}) => (m, c),
                        None => continue,
                    };

                    // process message to send

                    // send message off
                    //socket.send_to();
                    
                    // wait for ack (resend if not received)
                    // repeat
                }
            });
        }
       
        // receiver
        let recv_net = net.clone();
        thread::spawn(move|| {
            let mut socket = UdpSocket::bind("127.0.0.1:5000")
                .expect("Couldn't bind socket!");

            let mut buffer = [0; 4096];
            loop {
                let (amt, src) = socket.recv_from(&mut buffer)
                    .expect("Didn't receive data");

                let recv_net = recv_net.clone();
                thread::spawn(move|| {
                    recv_net.receive_handler(src, &buffer[..amt]);
                });
            }
        });

        net
    }

    fn receive_handler(&self, src: SocketAddr, buf: &[u8]) {

    }

    pub fn authenticate_user(&self, username: String, password: String) {
        let (sender, receiver) = channel::<Message>();
        let &(ref queue, ref cvar) = &*self.work;
        
        {
            let mut queue = queue.lock().unwrap();
            queue.push_back(MessageContainer{
                msg: Message {
                    msg_type: MessageType::Authenticate{
                        username: username, password: password
                    },
                    destination: vec![SERVER_ADDR.to_string()],
                },
                callback: Some(sender),
            });
        }
        cvar.notify_one();

        let received = receiver.recv().unwrap();

        // now do stuff with what was received.
    }

    pub fn send(&self, message: Message) {
        let &(ref queue, ref cvar) = &*self.work;
        {
            let mut queue = queue.lock().unwrap();
            queue.push_back(MessageContainer{msg: message, callback: None});
        }
        cvar.notify_one();
    }

}