Implemented 
3. merges and sorts the order books to create a combined order book

Partially implemented only for binance exchange but not tested with the real exchanges and not really covered by tests
1. connects to two exchanges' websocket feeds at the same time
2. pulls order books, using these streaming connections, for a given traded pair of currencies (configurable), from each exchange

Not implemented at all
4. from the combined book, publishes the spread, top ten bids, and top ten asks, as a stream, through a gRPC server

I was planning to finish binance connection and then implement similarly the other connection. Then using mpsc channel merge data from both exchanges and push it into the summary state. On each event send a summary to grpc rx end.
I was planning to implement an automatic subscription on symbols for both exchanges in the first run of the handler of subscription on the stream from grpc. Similarly when last stream has unsubscribed connections to the exchanges also dropped.

Finally, the server application on start prints its own grpc endpoint on the screen and optionally allows configuring address and port from the command line parameter. Trivial cli grpc client that accepts address/port/symbol from the command line and connects to the server. I intended to demonstrate running 3 clients that subscribe to 2 symbols. 2 clients for the first symbol and the last client for the second.
