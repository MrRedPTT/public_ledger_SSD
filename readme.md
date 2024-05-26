# About the Program

This is a project for Systems and Data Security class in the Masters of Cybersecurity

## How To Run:
### Set up
Compiling the Program:
```sh
    cargo build
```

If using using Windows, change the `OS_CONF` enc in the [.cargo/config.toml](./.cargo/config.toml) to "windows"

In order to avoid being asked for the `bootstrap` node location, add the `bootstrap.txt` file inside [src/](./src/) with the following content:
```
127.0.0.1
```

### Runing
In Terminal 1:
```sh
    cargo bootstrap
```

In Terminal 2:
```sh
    cargo client
```

In Terminal 3:
```sh
    cargo server
```

Each of the previous commands generates a node in the P2P Network.

### Notes on running

The `bootstrap` node needs to be the first node to be inside the network, 
after which, any number of `client` and/or `server` nodes and be created.

New nodes will only join the network if `bootstrap` node in online, but,
after joining the network, the `bootstrap` node does not need to be online
as the nodes already inside it, will comunicate between themselves.

The `client` nodes are nodes in which the Auction is enable and the user is able to interact with it,
on the other hand, `server` nodes are nodes that display usefull debug messages, such as,
which Marco it received.

It is best to have more than 1 `client` as clients that create Auctions will not be able to bid on them,
and the auction will not appear in `Show Auctions`, it will only appear in the second `client` node.

# About the project
## Secure Ledger

The secure ledger should be moduler and it must support PoW and Delegated Proof of Stake
- using proof-of-work[[1]](https://assets.pubpub.org/d8wct41f/31611263538139.pdf)
- proof-of-stake ([[3]](https://dl.acm.org/doi/pdf/10.1145/571637.571640), [[9]](https://ieeexplore.ieee.org/stamp/stamp.jsp?tp=&arnumber=8645706))

## P2P

Layer that gossips the necessary data to support the blockchain. Must include:
- must implemente S/Kademlkia [[5]](https://link.springer.com/chapter/10.1007/3-540-45748-8_5)
- Resistance to Sybil and Eclipse attacks
- Implement trust mechanisms (PoS)

## Auction

Auction System capable of supporting sellers and buyers using a single attribute auction following English auctions
- Transactions should be saved in blockchain and be properly gossiped
- publisher/subscriber should be built on top of Kadmlia to support auctions [[8]](http://bittorrent.org/beps/bep_0050.html)

## Fault injection
Fault Injection mechanism that allows to shutdown 1 or more nodes.
To be used during presentation to show resiliency of the system.

# Useful links: 

## Rust Documentation

[Rust documentation](https://www.rust-lang.org/learn)

## References 

1. [Bitcoin](https://assets.pubpub.org/d8wct41f/31611263538139.pdf)
by Satoshi Nakamoto

2. [Ethereum](https://cryptodeep.ru/doc/paper.pdf) 
bt Gavin Wood

3. [Practical byzantine fault tolerance and proactive recovery, ACM Transactions on Computer Systems (TOCS)](https://dl.acm.org/doi/pdf/10.1145/571637.571640) 
by Miguel Castro & Barbara Liskov

4. [State machine replication for the masseswith bft-smart](https://ieeexplore.ieee.org/stamp/stamp.jsp?tp=&arnumber=6903593) 
by João Sousa, Eduardo Alchieri, and Alysson Bessani.

5. [Kademlia: A peer-to-peer information system based on the xor metric](https://link.springer.com/chapter/10.1007/3-540-45748-8_5) 
by Petar Maymounkov and David Mazieres. 

6. [A practicable approach towards secure key- based routing. In Parallel and Distributed Systems](https://ieeexplore.ieee.org/stamp/stamp.jsp?tp=&arnumber=4447808) 
by Ingmar Baumgart and Sebastian Mies.

7. [Reputation based approach for improved fairness and robustness in p2p protocols](https://link.springer.com/article/10.1007/s12083-018-0701-x) 
by Francis N. Nwebonyi, Rolando Martins, and Manuel E. Correia.

8. [Bittorrent publish/subscribe protocol](http://bittorrent.org/beps/bep_0050.html) 

9. [Repucoin: Your reputation is your power](https://ieeexplore.ieee.org/stamp/stamp.jsp?tp=&arnumber=8645706) 
by Yu, Jiangshan, David Kozhaya, Jeremie Decouchant, and Paulo Esteves-Verissimo

## Software

[Jet Brains - rust](https://www.jetbrains.com/rust/nextversion/)

[Jet Brains - students](https://www.jetbrains.com/shop/eform/students)


