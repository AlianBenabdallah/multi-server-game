use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, SocketAddr, Shutdown};
use std::str::{from_utf8};
use rand::Rng;


pub struct Agent{
    pub id : u16,                       // Identifier (port number)
    value : [u8; 2],                    // Value to send stored in a byte array
    listener : TcpListener,             // TCP socket
}

impl Agent{
    pub fn new (real_value : u16, max_value : u16, liar : bool) -> Self {
        /*
            Creates a new agent. 
                - id is any available port on which the agent listens.
                - value is real_value if liar is false otherwhise a random value between 1.. 
                  and max_value which is not real_value

            Args : 
                - real_value : target value
                - max_value : expected value
                - liar : true if the agent must lie
            Returns :
                - Agent
        */
        let mut val = real_value;

        // If the agent lies, value is a random u16 which is not real_value
        if liar {
            let mut rng = rand::thread_rng();
            val = rng.gen_range(1..max_value) as u16; 
            if val >= real_value{  
                val = val + 1;
            }
        }
        
        let addrs = SocketAddr::from(([127, 0, 0, 1], 0));  // Ports 0 will find any available port

        let listener = TcpListener::bind(addrs).unwrap();

        Self {
            id : listener.local_addr().unwrap().port(),
            value : val.to_be_bytes(),           
            listener : listener,
        }
    } 


    pub fn handle_connection(&self, mut stream : TcpStream) -> bool{
        /*
            Reads the value received from the client.
            If the value is "talk" it sends self.value and returns false
            If the value is "stop" it returns true without answering.
            
            Args : 
                - stream : TCP stream with the client
            Returns : 
                - bool : true if stop is received
        */

        let mut stop = false;

        let mut buffer = [0 as u8; 1024];
        match stream.read(& mut buffer){

            Ok(size) => {
                let msg = from_utf8(&buffer[..size]).expect("");
                match msg {
                    "talk" => {
                        stream.write(&self.value).unwrap();
                    },
                    "stop" => stop = true,
                    _ => {
                        println!("Agent {}: Received incorrect message : {}", self.id, msg);
                    },
                }
            },

            Err(e) => {
                stream.shutdown(Shutdown::Both).unwrap();
                println!("Error handle connection {}", e);
            }
        }
        stop
    }

    pub fn run(&mut self) -> (){
        /*
            Thread loop. It listens to incoming connnections on the listener and calls handle_connection.
            If handle_connection returns true, it closes the socket and the thread can be joined.
        */

        println!("Agent {} listening", self.id);
        for stream in self.listener.incoming(){
            let stream = stream;
            let mut stop = false;
            
            match stream {
                Ok(stream) => {
                    // Connection succeed
                    stop = self.handle_connection(stream);
                }
                Err(e) => {
                    // Connection failed
                    println!("Error : {}", e);
                }
            }

            if stop{
                break;
            }
        }

    }
}


/*---------------------------- TESTS ----------------------------*/

#[cfg(test)]
mod tests {
    use crate::Agent;
    use std::thread;
    use std::net::{SocketAddr, TcpStream};
    use std::io::{Read, Write};
    use std::collections::HashSet;

    #[test]
    fn test_agent() {
        let value : u16 = 5;
        let max_value : u16 = 5;
        let mut agent_liar = Agent::new(value, max_value, true);
        let mut agent_truthful = Agent::new(value, max_value, false);

        let port_liar = agent_liar.id;
        let port_truthful = agent_truthful.id;
        
        let thread_liar = thread::spawn(move || {agent_liar.run();});
        let thread_truthful = thread::spawn(move || {agent_truthful.run();});

        let addrs_liar = SocketAddr::from(([127, 0, 0, 1], port_liar));
        let addrs_truthful = SocketAddr::from(([127, 0, 0, 1], port_truthful));

        let mut liar_set : HashSet<u16> = HashSet::new();
        let mut truthful_set : HashSet<u16> = HashSet::new();
        
        for _ in 0..10{
            match TcpStream::connect(addrs_liar) {
                Ok(mut stream) => {
                    let b = "talk".as_bytes();
                    stream.write(b).unwrap();
                    
                    let mut buffer = [0 as u8; 1024];

                    match stream.read(& mut buffer){
                        Ok(size) => {
                            assert_eq!(size, 2);
                            let val : u16= (buffer[1] as u16 ) | (buffer[0] as u16) << 8;
                            assert_eq!(stream.peer_addr().unwrap().port(), port_liar);
                            assert!(!(val == value));
                            liar_set.insert(val);
                            
                        },
        
                        Err(e) => {
                            println!("Error while reading: {}", e);
                            assert!(false);
                        }
                    
                    }
                },
                Err(e) => {
                    println!("Error while connecting : {}", e);
                    assert!(false);
                }
            }
        

            match TcpStream::connect(addrs_truthful) {
                Ok(mut stream) => {
                    let b = "talk".as_bytes();
                    stream.write(b).unwrap();
                    
                    let mut buffer = [0 as u8; 1024];

                    match stream.read(& mut buffer){
                        Ok(size) => {
                            assert_eq!(size, 2);
                            let val : u16= (buffer[1] as u16 ) | (buffer[0] as u16) << 8;
                            assert_eq!(stream.peer_addr().unwrap().port(), port_truthful);
                            assert_eq!(val, value);
                            truthful_set.insert(val);
                            
                        },
        
                        Err(e) => {
                            println!("Error while reading : {}", e);
                            assert!(false);
                        }
                    
                    }
                },
                Err(e) => {
                    println!("Error while connecting : {}", e);
                    assert!(false);
                }
            }
        }

        match TcpStream::connect(addrs_liar) {
            Ok(mut stream) => {
                let b = "stop".as_bytes();
                stream.write(b).unwrap();
            },
            Err(e) => {
                println!("Error while connecting : {}", e);
                assert!(false);
            }
        }

        match TcpStream::connect(addrs_truthful) {
            Ok(mut stream) => {
                let b = "stop".as_bytes();
                stream.write(b).unwrap();
            },
            Err(e) => {
                println!("Error while connecting : {}", e);
                assert!(false);
            }
        }



        assert_eq!(truthful_set.len(), 1);
        assert_eq!(liar_set.len(), 1);

        thread_liar.join().expect("The thread being joined has panicked");
        thread_truthful.join().expect("The thread being joined has panicked");

    }
}