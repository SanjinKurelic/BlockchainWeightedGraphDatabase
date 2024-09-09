# Blockchain Weighted Graph Database

Blockchain Weighted Graph Database is type of graph database where each node is connected
with weighted edge (e.g. User John like Song "Cool Song" with 75% weight of edge between them).

Edge store only the last value of the weight, but each action which change the value of weight is stored
in blockchain which means every agent in network can verify if this weight is correct and which actions
made current value.

This Graph Database also support simple query language for CRUD operations under graph database and blockchain.

## Example

Let's say we have graph network where we have Users and Playlists with Songs. Each User can give reaction to the Playlist
with emoticon:

| Emoticon         | Weight |
|------------------|--------|
| :metal:          | 90     | 
| :partying_face:  | 70     | 
| :astonished:     | 60     | 
| :hushed:         | 45     | 
| :yum:            | 30     | 
| :smiley:         | 20     | 
| :wink:           | 10     | 
| :expressionless: | 0      | 
| :confused:       | -10    | 
| :sleepy:         | -20    | 
| :sleeping:       | -30    | 
| :unamused:       | -45    | 
| :disappointed:   | -60    | 
| :-1:             | -70    | 
| :no_entry_sign:  | -90    | 

To create such network, first we should create nodes:

```
add node User(name="John")
```

```
add node Playlist(name="Party mix", description="Let's party")
```

```
add node Song(name="Cool song", yearOfRelease=1992, file="song.mp3")
```

Then we can connect nodes that are fixed by the design:

```
add connection from Playlist($id="OpRi5Yhr0s4TbQXU") to Song($id="TYqHmCEulrTXI0hk") with weight 100
```

On the client app we can map all those emoticons to weight numbers, so when user react with :partying_face: this command should
be sent to the Graph Database:

```
add connection from User($id="YTB3kJI9L6kmiF0z") to Playlist($id="OpRi5Yhr0s4TbQXU") with weight 70
```

Using search filed in the client app user can find his/her favourite Playlist with this command:

```
fetch User($id="YTB3kJI9L6kmiF0z") join Playlist($weight>=50) join Song($weight=100)
```

Which will return this result:

```json
{
  "result_list": [
    {
      "name": "John",
      "playlist_list": [
        {
          "name": "Party Mix",
          "description": "Let's party",
          "song_list": [
            {
              "name": "Cool song",
              "yearOfRelease": 1992,
              "file": "song.mp3"
            }
          ]
        }
      ]
    }
  ]
}
```

If user change the weight of the connection:

```
update connection from User($id="YTB3kJI9L6kmiF0z") to Playlist($id="OpRi5Yhr0s4TbQXU") with weight 30
```

Then result will be empty for same `fetch` query:

```json
{
  "result_list": []
}
```

To retrieve weight history we can query blockchain using this command:

```
fetch connection chain
```

Which will output following result:

```json
{
  "result_list": [
    {
      "id": 0,
      "hash": "0000f816a87f806bb0073dcf026a64fb40c946b5abee2573702828694d5b4c43",
      "previous_hash": "root",
      "timestamp": 1725886322,
      "data": {
        "from": "YTB3kJI9L6kmiF0z",
        "to": "OpRi5Yhr0s4TbQXU",
        "weight": 70
      },
      "nonce": 2836
    },
    {
      "id": 1,
      "hash": "00008cf68da9f978aa080b7aad93fb4285e3c0dbd85fc21bc7e83e623f9fa922",
      "previous_hash": "0000f816a87f806bb0073dcf026a64fb40c946b5abee2573702828694d5b4c43",
      "timestamp": 1725886362,
      "data": {
        "from": "YTB3kJI9L6kmiF0z",
        "to": "OpRi5Yhr0s4TbQXU",
        "weight": 30
      },
      "nonce": 62235
    }
  ]
}
```