# Blockchain Weighted Graph Database

Blockchain Weighted Graph Database is type of graph database where each node is connected
with weighted edge (e.g. User John like Song "Cool Song" with 75% weight of edge between them).

Edge store only the last value of the weight, but each action which change the value of weight is stored
in blockchain which means every agent in network can verify if this weight is correct and which actions
made current value.

This Graph Database also support simple query language for CRUD operations under graph database and blockchain.

## Graph Database

To explain how this database works, let's say we have graph network where we have Users and Playlists with Songs.
Each User can give reaction to the Playlist with emoticon:

| Positive Emoticon | Weight | Neutral Emoticon | Weight | Negative Emoticon | Weight |
|:-----------------:|:------:|:----------------:|:------:|:-----------------:|:------:|
|      :metal:      |   90   | :expressionless: |   0    |    :confused:     |  -10   |  
|  :partying_face:  |   70   |                  |        |     :sleepy:      |  -20   | 
|   :astonished:    |   60   |                  |        |    :sleeping:     |  -30   | 
|     :hushed:      |   45   |                  |        |    :unamused:     |  -45   | 
|       :yum:       |   30   |                  |        |  :disappointed:   |  -60   | 
|     :smiley:      |   20   |                  |        |       :-1:        |  -70   | 
|      :wink:       |   10   |                  |        |  :no_entry_sign:  |  -90   |

Each User interaction with Playlist will change the weight of the edge between them and will also affect fetching the results from the database, e.g.
pseudo command: Give me all Playlists that Users from continent Europe disliked with at least :sleeping: emoticons.

### Defining nodes

To define nodes in such network we need to define each node structure:

```
define node Playlist(name,description)
```

Output: `[{"name":"*"}]`.

Structure of each node is defined with name and list of attributes in brackets. This will also prevent clients to insert nodes with attributes which
are not listed by definition. Even though currently not fully supported, client can define attribute index by using `*` sign.

```
define node Song(*name,yearOfRelease,file)
```

Output: `[{"name":"*","yearOfRelease":"*","file":"*"}]`.

As, we can see from Song definition, name is stored in B-Tree index for faster search. Node definition currently does not support defining which nodes
are required.

> Note: Instead of using B-Tree indexes for Graph Database, Merkle Tree could be used, and root hash could be stored in blockchain.

```
define node User(name,premium) with agent (premium=true)
```

Output: `[{"name":"*","premium":"*"}]`.

User node definition also have agent definition which means this node will be used to create new block in blockchain (usually this is only related to
nodes which holds data from real users but can also be used from nodes which define automated nodes, e.g. AI clients). In current example, only
user nodes with premium set to true can create new block in blockchain.

### Inserting nodes

After defining each node, we can insert node data:

```
add node User(name="John")
```

Output: `[{"$name":"User","$id":"YTB3kJI9L6kmiF0z","$edges":"0","name":"John"}]`.

After inserting node, we can see that each node have additional attributes which are used for internal purposes. Each node have unique id which is
used
to retrieve node from database and to connect nodes with edges. Each node also have `$edges` attribute which is used to store number of edges
connected
to this node. This value will also define difficulty of mining new block in blockchain.

```
add node Playlist(name="Party mix", description="Let's party")
```

Output: `[{"$name":"Playlist","$id":"OpRi5Yhr0s4TbQXU","$edges":"0","name":"Party mix","description":"Let's party"}]`.

```
add node Song(name="Cool song", yearOfRelease=1992, file="song.mp3")
```

Output: `[{"$name":"Song","$id":"TYqHmCEulrTXI0hk","$edges":"0","name":"Cool song","yearOfRelease":"1992","file":"song.mp3"}]`.

### Connecting nodes

Some nodes are connected with fixed weigh by design - usually we define it with weight=100, e.g. each Song can be either connected to Playlist, or not
part of it:

```
add connection from Playlist($id="OpRi5Yhr0s4TbQXU") to Song($id="TYqHmCEulrTXI0hk") with weight 100
```

Output: `[{"$weight":"100","$from":"Playlist","$to":"Song"}]`.

For dynamic weight  (e.g. User reaction to Playlist), client must define dynamic query which is sent to database server. Also, in current example
client app should map all those emoticons to weight numbers, so when user react with :partying_face: this command will be sent to the Graph Database:

```
add connection from User($id="YTB3kJI9L6kmiF0z") to Playlist($id="OpRi5Yhr0s4TbQXU") with weight 70
```

Output: `[{"$weight":"70","$from":"User","$to":"Playlist"}]`.

Each connection between nodes is also stored in blockchain and published over peer-to-peer network.

### Fetching nodes

Using search filed in the client app user can find his/her favourite Playlist by using this command:

```
fetch User($id="YTB3kJI9L6kmiF0z") join Playlist($weight>50) join Song($weight=100)
```

Output:

```json
[
  {
    "$name": "User",
    "$id": "YTB3kJI9L6kmiF0z",
    "$edges": "1",
    "name": "John",
    "Playlist.$name": "Playlist",
    "Playlist.$id": "OpRi5Yhr0s4TbQXU",
    "Playlist.$edges": "1",
    "Playlist.name": "Party mix",
    "Playlist.Song.name": "Cool song",
    "Playlist.Song.yearOfRelease": "1992",
    "Playlist.Song.file": "song.mp3"
  }
]
```

This command will return all Playlists that User John liked with at least 50% weight and all Songs that are part of that Playlist.
Database server will return them as flattened HashMap result converted to JSON, but code can easily be updated to support nested JSON objects.

If user update the weight of the connection:

```
update connection from User($id="YTB3kJI9L6kmiF0z") to Playlist($id="OpRi5Yhr0s4TbQXU") with weight 30
```

Then result will be empty for same `fetch` query: `[{}]`.

## Blockchain

Each user connection between nodes is stored in blockchain. Each block in blockchain contains sequence id, hash and previous block hash, as well as
timestamp. Also, each block contains signature which is public key of the user who created the block (validator) and validator which is reference to
user agent stored in graph database. Only premium users can create new block in blockchain.

There is also difficulty attribute which is used to prevent spamming the network with new blocks. Difficulty is calculated by counting number of edges
connected to user agent node, and only users with 1 edge more can approve new block in the chain (Proof Of Interaction - variant of Proof Of Stake).

Data in each block is stored in JSON format and contains data_type attribute which is used to define type of data stored in block. Each block can
contain RootNode data which is used to store initial block in the chain, ValidatorData which is used to store user agent data, and EdgeData
which is used to store connection between nodes.

To fetch current chain, client can use following command:

```
fetch connection chain
```

Which will output following result:

```json
[
  {
    "signature": "",
    "difficulty": "0",
    "validator": "",
    "id": "0",
    "data": "{\"data_type\":\"RootNode\",\"edge_data\":null,\"validator_data\":null}",
    "timestamp": "1726781317",
    "previous_hash": "",
    "hash": "0000494d137e1631bba301d5acab6e7bb7aa74ce1185d456565ef51d737677b2"
  },
  {
    "signature": "dc8accf49a7bd6974cdf3eb6e6f392454bae8d1af6c43f3a87514e14f56ee4c4adf4ed9ca95a39098c4251d716058c04ccad79105ff48d35f91915fdda215c0d",
    "difficulty": "0",
    "validator": "3087748bc2ea5e6da1ed351ef7a8d763b3b61132ecb75ebf43cb08adbcc8dd29",
    "id": "1",
    "data": "{\"data_type\":\"ValidatorData\",\"edge_data\":null,\"validator_data\":{\"public_key\":\"3087748bc2ea5e6da1ed351ef7a8d763b3b61132ecb75ebf43cb08adbcc8dd29\",\"account_id\":\"kHXsjzIFMCg9Wuj4\"}}",
    "timestamp": "1726781317",
    "previous_hash": "0000494d137e1631bba301d5acab6e7bb7aa74ce1185d456565ef51d737677b2",
    "hash": "920871682f617ba0be3c208248c7d6bfc160b7ee7838af1d8426386828b11943"
  }
]
```

## Running the project

Project can be run by using following command:

```shell
cargo run
```

Or using connected environment (max 4 additional clients - for more edit bootstrap.rs file):

```shell
cargo run -- -username1="..." -key1="..." -username2="..." -key2="..." -username3="..." -key3="..."
```

There are also many test cases in project which can be run by using following command:

```shell
cargo test
```

## Technical details

This project is written in Rust and uses following libraries:

-  argmap - for parsing command line arguments
- derive_more - for deriving more traits
- ed25519-dalek - for Ed25519 digital signatures
- hex - for encoding and decoding hex strings
- libp2p - for peer-to-peer networking
- nanoid - for generating unique ids (Graph Database)
- peg - for parsing database query language
- rand - for generating random numbers
- rustc-hash - for hashing keys [1]
- serde - for serializing and deserializing data
- sha2 - for SHA-256 hashing (Blockchain)
- tokio - for async networking and file I/O


> [1] Instead of using default Rust hash function, we are using FxHasher which is faster for indexes less than 32 bytes
of data. If someone wants to use this project for bigger indexes, it is recommended to use Fowler-Noll-Vo hash function.