# Pi Calc Smart Contract Runtime

## History of Mobile Process Calculi
In 1992 Robin Milner et al introduced the pi calculus, a member of the mobile process calculus family, as a new and novel model of computation [[paper](https://dl.acm.org/ft_gateway.cfm?id=151240&ftid=289787&dwn=1&CFID=153770346&CFTOKEN=6894c386a5ee5d2c-40734F3C-C2A2-949F-6AAE98DF0EA2B5A5)]. The pi calculus has several properties that make it an attractive option for modern programming. It is Turing complete,  allows for straight-forward compositional programming, and is fundamentally concurrent. With the appropriate structure on it's channels, it allows for programming with in object capability paradigm (see architecture section below).

## History of Blockchain
In 2009 Bitcoin introduced the proof of work blockchain allowing mutually distrusting parties to coordinate permissionlessly through a currency. In 2013 Ethereum generalized that idea to coordinate on arbitrary computations through a system of smart contracts. Both systems have been successful but have struggled to scale.

Progress has been made toward scaling Bitcoin-like UTXO systems through concurrent consensus algorithms [[spectre](https://www.cs.huji.ac.il/~yoni_sompo/pubs/17/SPECTRE.pdf), [cbc casper](https://github.com/ethereum/research/blob/master/papers/CasperTFG/CasperTFG.pdf), [hashgraph](https://www.swirlds.com/downloads/SWIRLDS-TR-2016-01.pdf), [casanova](https://arxiv.org/pdf/1812.02232.pdf)]. However even these algorithms will not scale Ethereum as it's computational model is inherently sequential and totally-ordered.

Recently, several blockchain projects are beginning to explore smart contracting solutions based on mobile process
calculi [[RChain](https://architecture-docs.readthedocs.io/index.html),
[Ambients](https://ipfs.io/ipfs/QmPhPJE55GvqSz7Pwvkc8n9dbKmqGw6tUGTE1MgfNQvzsf)] rather than traditional sequential
models like Ethereum. Such projects will be able to take advantage of concurrent consensus.

## Goals and Scope

The primary goal of this project is to provide the Substrate ecosystem with a smart contracting module that supports the
pi calculus much like the existing [Contracts Module](https://crates.parity.io/srml_contracts/index.html) supports
Ethereum-style sequential contracts.

### In Scope

* A smart contracting Runtime Module
* Support for concurrent execution at the runtime level
* An object capability system (This can likely be separated into its own independently-useful module, but more thought
needed)

### **Not** in Scope
* A concurrent consensus algorithm for Substrate
* Interoperability with any other mobile process calc-based blockchains
* Interoperability with the existing Contracts module

## Architecture

### The Grammar
The pi calculus specifies a grammar including Processes and Channels.

#### Processes
The process constructors are taken exactly from the Minimal Pi calculus.
* Send
* Receive
* Parallel Compose
* Replicate
* Stop
* Restrict (new binding)

#### Channels
The channels, however, will have a small amount of additional structure by being split into two forms
* Public - Anyone can access a public channel simply by including it in their process.
* Unforgeable - Unforgeable names are created by the Restriction process and are available only to those who have created them, or been given access to one in the pi-vm. They provide the basis for object capability programming.


### Storage and Deployment
The primary storage item is a map from process ids to processes. Ids are calculated deterministically (but not sequentially in order to support a partially-ordered consensus algorithm should one ever exist).

The module provides a dispatchable call,
```rust
fn deploy(origin, term: Process) -> Result
```
which adds a process into the storage map giving a new process id.

### Reductions

As with other mobile process calculi, computation happens by means of reductions. Blockchains like RChain make the
mistake of having validators do the expensive and non-deterministic work of _finding_ valid reductions before executing
them. The correct design is to have users find the reductions, and submit them to vsalidators who merely confirm that
the reductions are valid.

The pi calculus specifies two kinds of reductions.

#### Communication Events

The primary reduction is the communications event where a send and a receive on the same channel are both consumed and
their continuations are deployed into the tuplespace. The module provides a dispatchable call, ```rust fn comm(origin,
send: Process, recv: Process) -> Result ``` where the user specifies which processes should be consumed, and the
validators confirm that they are compatible.

#### Replication Expansion
The secondary reduction is expanding a replicated process `!P -> P | !P`. The module provides a dispatchable call,
```rust
fn expand(origin, source: Process) -> Result
```
where the user specifies a process to replicate and the validator ensures that it is compatible.

#### Batch Reductions In practice it will be uncommon to perform single reductions. Rather they will come in batches of
#a few to a few dozen. Later the interface may change to accept a vector of reductions to optimize away excessive
#storage updates. This will happen after the simpler interface is found to work fairly reliably.

### Object Capabilities In Ethereum and the existing Contracts module, access control lists ala `msg.sender` or `origin`
#are ubiquitous. The benefits of object capabilities are well understood
#[[paper](http://srl.cs.jhu.edu/pubs/SRL2003-02.pdf)], and can easily be added to the pi calculus.

The unforgeable channels described above provide the only scaffolding necessary to include an object capability system
in the pi-vm. By creating a receive process that listens on an unforgeable channel, a user ensures that only users who
have access to that unforgeable channel can "call the contract" (deploy a send that will reduce with the receive).

### Storage Rent Storage rent is a low-priority feature to be developed when the above architecture is largely working.
#The implementation can hopefully be informed by that of the existing Contracts module. The idea is that each term that
#is deployed (directly or as the result of a reduction)
