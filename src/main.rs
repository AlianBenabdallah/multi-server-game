mod agent;
mod config;

use agent::Agent;
use config::{Config, write_config_file, read_config_file, delete_config_file};

use std::collections::{HashMap, HashSet};
use std::thread;
use std::io::{self, Read, Write};
use std::net::{TcpStream, SocketAddr};
use std::str::{from_utf8};



fn init(config : Config) -> (Vec<thread::JoinHandle<()>>, Vec<u16>) { 
    /* 
        Reads config file and launches a total of config.num_agents agents' threads among which
        config.liar_ratio * config.num_agents are liars and the rest are telling the truth.

        Returns a vector of threads to join and a vector of port numbers chosen randomly by each agent.

        Args :
            - config : Config structure 
        Returns :
            -  Vec<thread::JoinHandle<()>> : Threads to join with shutdown()
            -  Vec<u16> : Port numbers.
    */
    let number_of_liars = (config.liar_ratio * (config.num_agents as f32)) as u16;
    
    let mut thread_vec : Vec<thread::JoinHandle<()>> = Vec::new();
    let mut port_vec : Vec<u16> = Vec::new();

    // Launch the liars
    for _ in 0..number_of_liars {
        let mut agent = Agent::new(config.value, config.max_value, true);
        let id = agent.id;
        port_vec.push(id);

        let thread = thread::spawn(move || {agent.run();});
        thread_vec.push(thread);
    }

    // Launch the real agents
    for _ in 0..config.num_agents - number_of_liars {
        let mut agent = Agent::new(config.value, config.max_value, false);
        let id = agent.id;
        port_vec.push(id);

        let thread = thread::spawn(move || {agent.run();});
        thread_vec.push(thread);
    }

    (thread_vec, port_vec)
}

fn game_loop(value : u16, liar_ratio : f32){
    /* 
        Reads the stdin until the end of the game. It waits for a "play" or "stop" command. 
        If it reads play, it plays a round of the game. If the game is won, the program stops. 
        If it reads stop, it stops the game.

        Args : 
            - value : target value
            - liar_ratio : liar ratio
    */
    
    let mut already_tried : HashSet<u16> = HashSet::new();  // Set of values proposed in the previous rounds
    let mut number_of_rounds = 1;
    println!("ready");                                  
    loop {

        let mut input = String::new();
        io::stdout().flush().expect("Couldn't flush stdout");
        io::stdin().read_line(&mut input).expect("Error reading input.");
        match input.trim(){
            "play" => {
                if play(&mut already_tried, liar_ratio, value){
                    println!("You have found the correct value after {} round(s) !", number_of_rounds);
                    break;
                }
            },
            "stop" => {break;},
            _ => println!("You should enter 'play' or 'stop', you entered {}", input),
        }
        input.clear();
        number_of_rounds += 1;
    }
    stop();
}

fn stop(){ 
    /* 
        Reads config file, connects to every agent and sends them "stop"
        Necessary to stop the threads.
    */

    let ports = read_config_file();
    
    for port in ports {
        let addrs = SocketAddr::from(([127, 0, 0, 1], port));
        match TcpStream::connect(addrs) {
            Ok(mut stream) => {
                let b = "stop".as_bytes();
                stream.write(b).unwrap();
            }
            Err(e) => {
                println!("Failed to connect to {} : {}", port, e);
            }
        }
    }
}

fn play(already_tried : &mut HashSet<u16>, liar_ratio : f32, value : u16) -> bool { 
    /*
        Plays a round of the game. It sends "talk" to every agent. The agents will answer with their value.
        The values are counted in the map "counts". The client compares the frequency of each value to
        the ratio of agents telling the truth (1 - liar_ratio) and selects the closest one.

        It sends messages to every agent then receives their answer to avoid waiting for answers from slow agents.

        Args : 
            - already_tried : set of values played in previous round
            - liar_ratio : liar ratio
            - value : target value 
        Returns :
            - bool : true if game is won else false
    */

    let ports = read_config_file();
    let size = ports.len() as f32;

    let mut counts : HashMap <u16, f32>= HashMap::new();
    {
        let mut tcp_connections = Vec::new();

        // Sending messages
        for port in ports {
            let addrs = SocketAddr::from(([127, 0, 0, 1], port));
            match TcpStream::connect(addrs) {
                Ok(mut stream) => {
                    let b = "talk".as_bytes();
                    stream.write(b).unwrap();
                    tcp_connections.push(stream);
                }
                Err(e) => {
                    println!("Failed to connect to {} : {}", port, e);
                }
            }
        }
        // Receiving answers
        for mut stream in tcp_connections {
            let mut buffer = [0 as u8; 1024];
            match stream.read(& mut buffer){
                Ok(size) => {
                    let msg = from_utf8(&buffer[..size]).expect("");
                    if size == 2 {
                        // [u8,u8]
                        
                        // [16-8, 0-8]  
                        let val : u16 = (buffer[1] as u16) | (buffer[0] as u16) << 8;
                        println!("Client: received {} from {}", val, stream.peer_addr().unwrap().port());
                        let count = counts.entry(val).or_insert(0.0);
                        *count += 1.;
                    } else {
                        println!("Client: received incorrect data {} from {}", msg, stream.peer_addr().unwrap().port());
                    }
                },

                Err(e) => {
                    println!("Client failed to read {}", e);
                }
            }
        }
    }
    // (value => frequency)
    let mut new_key : u16 = 0;
    let mut min_diff = 2.;

    // (1 => 7,
    //  2 => 3)
    for (key, value) in counts {
        if !already_tried.contains(&key){
            // |0.3 - 0.7| / |0.3 - 0.3|
            let diff = f32::abs((1.0 - liar_ratio) - value / size);
            if diff < min_diff{
                new_key = key;
                min_diff = diff;
            }
        }
    }

    println!("You propose value {}", new_key);

    already_tried.insert(new_key);

    new_key == value
}

fn shutdown(threads : Vec<thread::JoinHandle<()>>) -> () {
    /* 
        Joins every thread.

        Args : 
            - threads : threads to be joined
    */
    for thread in threads {
        thread.join().expect("The thread being joined has panicked");
        println!("Joined thread");
    }
    println!("Joined all threads");
}

fn main() {

    println!("Welcome to liarslie. To start a new game, please type");
    println!("start --value <v> --max-value <max> --num-agents <number> --liar-ratio <ratio>");
    
    // Wait for a valid start command and parses it in a Config Structure
    let config : Config = Config::new();

    // extract useful fields
    let ratio = config.liar_ratio;
    let value = config.value;

    // Launches the threads and get the port numbers
    let (threads, ports) = init(config);

    // Write config file
    write_config_file(ports);

    // Game loop. Waits for "play" or "stop"
    game_loop(value, ratio);

    // Join every thread
    shutdown(threads);

    // Deleted agent.config;
    delete_config_file();
    
}
