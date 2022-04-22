use clap::{Arg, App};
use std::fs::File;     

use std::io;
use std::io::{BufRead, Write};
use shellwords;

pub struct Config{
    pub max_value : u16,
    pub value : u16,
    pub num_agents : u16,
    pub liar_ratio : f32,
}

pub fn write_config_file(ports : Vec<u16>){
    /*
        Writes agent.config with the port numbers of all the agents.

        Args : 
            - ports : port number of every agent
    */
    let mut f = File::create("agent.config").expect("Unable to create file");                                                                                                          
    for i in &ports{                                                                                                                                                                  
        let write = write!(f, "{}\n", i); 
        match write {
            Ok(_) => {},
            Err(e) => {
                println!("Error happened while writing config file {}", e); 
                std::process::exit(1)
            },
        };    
    }
}

pub fn read_config_file() -> Vec<u16> {
    /* 
        Reads agent.config and returns the port of every agent 
    */
    let file = File::open("agent.config");
    let mut vec = Vec::new();
    match file {
        Ok(lines) => {
            let lines = io::BufReader::new(lines).lines();
            for line in lines {
                match line {
                    Ok(ip) => {
                        let int = ip.parse::<u16>();
                        match int {
                            Ok(u) => vec.push(u),
                            Err(e) => println!("Error {}", e),
                        }
                    }

                    Err(e) => {println!("Error {}", e)},
                }
            }
        },
        Err(e) => {println!("err : {}", e); std::process::exit(1);},
    }
    vec
}

pub fn delete_config_file(){
    /*
        Deletes agent.config
    */
    println!("deleting agent.config");
    match std::fs::remove_file("agent.config"){
        Ok(_) => println!("File agent.config successfully deleted"),
        Err(e) => println!("An error has occurred {}", e),
    };
}

impl Config {

    pub fn new() -> Self {
        /*
            Reads the std::in and waits for a command of structure :
                start --value <v> --max-value <max> --num-agents <number> --liar-ratio <ratio>
            Parse and checks that all the values are correct otherwise displays the correct usage and
            exits the program with code 1.

            Returns : 
                - Config structure from parsed input

            Example : // Exemple : start --value 1 --max-value 3 --num-agents 10 --liar-ratio 0.5
        */

        
        let mut input = String::new();
        let mut problem_found = false;
        
        let value : u16;
        let max_value : u16;
        let num_agents : u16;
        let liar_ratio : f32;

        /* ---- Read the stdin ---- */

        io::stdout().flush().expect("Couldn't flush stdout");
        io::stdin().read_line(&mut input).expect("Error reading input.");
        let words = shellwords::split(&input).unwrap();
        
        /* ---- Parses the command with clap ---- */
        let app = 
            App::new("liarlies")
            .usage("start --value <v> --max-value <max> --num-agents <number> --liar-ratio <ratio>")
            .version("1.0.0")
            .author("Benabdallah Ali")
            .arg(Arg::with_name("value")
                    .long("value")
                    .takes_value(true)
                    .help("True value, integer in [1 ; 65535]")
                    .required(true))
            .arg(Arg::with_name("max-value")
                    .long("max-value")
                    .takes_value(true)
                    .help("Maximum value, integer in [2 ; 65535]")
                    .required(true))
            .arg(Arg::with_name("num-agents")
                    .long("num-agents")
                    .takes_value(true)
                    .help("Number of agents, integer in [2 ; 1000].")
                    .required(true))
            .arg(Arg::with_name("liar-ratio")
                    .long("liar-ratio")
                    .takes_value(true)
                    .help("Ratio of liars, float in [0 ; 1[. There must be at least one liar")
                    .required(true));
        let matches = app.get_matches_from(words);

        /* ---- Get the value and sanity check ---- */
        
        value = match matches.value_of("value") {
            None => {problem_found = true; 0},
            Some(s) => {
                match s.parse::<u16>(){
                    Ok(n) => {
                        if n == 0{
                            println!("value should be in [1; 65535]");
                            problem_found = true;
                        }
                        n
                    },
                    Err(_) => {
                        println!("value should be a 16b integer");
                        problem_found = true; 
                        0
                    }
                }
            }
        };

        max_value = match matches.value_of("max-value") {
            None => {problem_found = true; 0},
            Some(s) => {
                match s.parse::<u16>(){
                    Ok(n) => {
                        if n < 2{
                            println!("max-value should be in [2;65535]");
                            problem_found = true;
                        }
                        
                        n
                    },
                    Err(_) => {
                        println!("max-value should be a 16b integer");
                        problem_found = true; 
                        0
                    }
                }
            }
        };

        num_agents = match matches.value_of("num-agents") {
            None => {problem_found = true; 0},
            Some(s) => {
                match s.parse::<u16>(){
                    Ok(n) => {
                        if n < 2{
                            println!("num-agents should be in [2;1000]");
                            problem_found = true;
                        }
                        n
                    },
                    Err(_) => {
                        println!("value should be a 16b integer");
                        problem_found = true; 
                        0
                    }
                }
            }
        };

        liar_ratio = match matches.value_of("liar-ratio") {
            None => {problem_found = true; 0.0f32},
            Some(s) => {
                match s.parse::<f32>(){
                    Ok(n) => {
                        let number_of_liars = (n * (num_agents as f32)) as u16;
                        if n < 0. || n >= 1.-1e-9 || number_of_liars < 1 || number_of_liars == num_agents {
                            println!("liar_ratio should be in [0,1[ with at least one liar and one honest agent. value : {}, number_of_liars {}", n, number_of_liars);
                            problem_found = true;
                        }
                        n
                    },
                    Err(_) => {
                        println!("liar_ratio should be a float");
                        problem_found = true; 
                        0.0f32
                    }
                }
            }
        };

        println!("max_value {}", max_value);
        println!("value {}", value);
        println!("num_agents {}", num_agents);
        println!("liar_ratio : {}", liar_ratio);

        // If it finds a problem, it exits the loop.
        // It can be done properly with try_get_matches() from clap 3.* but I started with 2.* 

        if problem_found {
            std::process::exit(1);
        }
            
        Self {     
            max_value : max_value,
            value : value,
            num_agents : num_agents,
            liar_ratio : liar_ratio,
        }

    }

}

/*---------------------------- TESTS ----------------------------*/



mod tests {
    use crate::config::{write_config_file, read_config_file, delete_config_file};
    use std::collections::HashSet;
    use std::path::Path;

    #[test]
    fn test_config_file() {

        let ports : Vec<u16> = (1..65535).collect();
        let ports_set : HashSet<u16> = HashSet::from_iter(ports.iter().cloned());

        write_config_file(ports);
        assert!(Path::new("./agent.config").exists());


        let port_read = read_config_file();
        let port_read_set: HashSet<u16> = HashSet::from_iter(port_read.iter().cloned());

        assert_eq!(port_read_set.difference(&ports_set).count(), 0);
        assert_eq!(ports_set.difference(&port_read_set).count(), 0);

        delete_config_file();

        assert!(!Path::new("./agent.config").exists());

    }
}

