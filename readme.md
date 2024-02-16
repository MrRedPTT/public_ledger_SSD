# About 

This is a project for Systems and Data Security class in the Masters of Cybersecurity

The [full assigment](./docs/assigment.pdf)

# To do:

## Secure Ledger

The secure ledger should be moduler and it must support PoW and Delegated Proof of Stake
- using proof-of-work[]
- proof-of-stake[3,9]

## P2P

Layer that gossips the necessary data to support the blockchain. Must include:
- must implemente S/Kademlkia [5]
- Resistance to Sybil and Eclipse attacks
- Implement trust mechanisms (PoS)

## Auction

Auction System capable of supporting sellers and buyers using a single attribute auction following English auctions
- Transactions should be saved in blockchain and be properly gossiped
- publisher/subscriber should be built on top of Kadmlia to support auctions [8]

## Fault injection
Fault Injection mechanism that allows to shutdown 1 or more nodes.
To be used during presentation to show resiliency of the system.

## Report
[check this link](https://www.overleaf.com/3364126665gpjnjtsdznxz#c52689)

# Useful links: 

## Documentation

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


