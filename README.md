# Ali Benabdallah 

This project is my first contact with Rust.

## Name
liarlie

## Description
Liars Lie is a game where a client queries a set of agents about an integer value. A configurable subset of agents tell the truth and will reveal the true value when asked. On the other hand, liars lie in Liars Lie and each liar will respond with an arbitrary (random) value, but always the same arbitrary value, when asked. The challenge is to determine the “true” value v by asking the agents their individual values. We call this v the network value below.

The main launcher will launch the agents on different threads and query them via TCP. It selects a value which has not been played before and uses knowledge about the liar ratio to identify the network value. It selects the value that appears with a frequency close to $1-liar\_ratio$.

## Dependencies

    clap = "2.33.3"
    rand = "0.8.5"
    shellwords = "1.0.0"

## Installation

`cargo build --release` will produce an executable in `./target/release/`.

## Tests
`cargo test` will run all the tests :
- In `config.rs` : Checks that the config file is correctly written, read and deleted. Verifies that no identifier is written twice.
- In `agent.rs` : Launches two agents. Verify that they handle messages correctly and always answer with the same value.

## Usage

Use `./target/release/liarslie.exe` to launch the program.

Then it expects the following command : 

    COMMAND :
        start
    ARGS : 
        --liar-ratio <liar-ratio>
        --max-value <max-value>
        --num-agents <num-agents>
        --value <value>
    USAGE:
        start --value <v> --max-value <max> --num-agents <number> --liar-ratio <ratio>
    EXAMPLE:
        start --value 1 --max-value 3 --num-agents 10 --liar-ratio 0.5

Once `ready` is displayed :

- `play` to play a round of the game.
- `stop` to stop the program.

## Design choice
The different actors communicate via TCP because of the reliability of TCP. 

TCP also allows the game to be played in a distributed settings. The agents' sockets could be wrapped to use TLS with a certificate signed by the client to allow authentication and the encryption of every message.

Each agent is on a separate thread as they take less time to switch context.

## Possible extension

Add encryption.

Add a manual mode.

Read the input in a loop when we start the executable. Needs to use a more recent version of clap or wrap the function in a `Result<>` structure. 

For the expert mode, liars could communicate via multicast even though it is not reliable, the game is ran locally.

Also, if we assume a reliable network with no loss or delay. If we also assume that the liars know the number liars and agents:

- If the liars are a majority :
    - The first liar queried by the client must broadcast his value.
    - The following liars will have to send the same value. This way, at any given time, the liars know how many times the client received this value.
    - Doing the same until this value is sent $floor((1 - liar\_ratio) * num_agents)$ times.
    - The rest of the agents will have to synchronize on a different value if possible in order to keep this value's frequency close to $1-liar\_ratio$.

- If the liars are not a majority.
    - A portion of liar will have to tell the truth while another will have to synchronize on another value as done previously in order to make the network value's frequency far from $1-liar\_ratio$.

## Authors and acknowledgment
Benabdallah Ali :  ali.benabdallah@epfl.ch

## Project status
Completed
