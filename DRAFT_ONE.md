# Draft ONE

## Command and Control

### Elements

- Hidden, fronted by some innocent service or obscure layers
- Accepts payloads from nodes. E.g. new_node, ip_update, heartbeat
- Queues tasks for nodes, when there is a gap (such as a node checking in), provide it with a workload or refresh its knowledge (p2p mesh info or whatever else)

### The Front

- Simple points of redirection. Such as fake blog websites, stores etc. Simple, not the best option, but can be used alongside more complex options.

- Dead drops. Nodes need only know where to dump a payload, and the C2 server will come to collect. Opens up some interesting options. But the target location still needs to be unsuspicious. It's not far fetched to expect a netadmin to notice frequent dumps to pastebin-type websites and be suspicious about it. Maybe encode data to video frames and upload to a private YouTube channel? Many ideas. But there is also the possibility that the target/enemy environment is highly restricted, blocking access to Google or Amazon services.

- Peer-to-peer mesh, with stationed/fixed beacons within the operator's control. A node might prepare a payload to be hopped through the mesh until it lands on a node that knows of a beacon, then forwards it to that beacon. The beacon is kind of like a depot. Where it will have direct communication with the C2, outside the control of the target/enemy. Not all nodes know where the beacons are. Knowledge of this might be distributed selectively to the more trustworthy or reliable nodes. Or at random. This will, however, require a mechanism to prevent build-up of duplicate payloads. Node A broadcasts to node B and C, C then to D, B to E, E to D and F, D to E and F. Either a node needs to know if it already holds a payload, or the payloads need some way to expire. Perhaps like the traceroute command's usage of the TTL field. Allowing perhaps only _x_-amount of hops before dying. Or maybe a timestamp that expires after a certain amount of time. Or both.
There's also the risk of a payload never reaching a beacon if a payload can expire. A crude solution would simply be to give large TTL values or expiry times to payloads and hope for the best. But that is a bit of a "shot in the dark" approach. Perhaps a lightweight reputation mechanism can be built in each node. The relaying mechanism could perhaps be built to accumulate travel info inside on payload. As an example, node A sends a payload to node B and C, the payload propagates through the net and tracks which nodes it hops on until it reaches a beacon. The beacon then emits its own payload intended for the authoring node, node A, to notify it that the payload was received. From this, node A could increase the reputation score of node B or C depending on through which one the beacon received the payload. This could be something to dive into, for now just a thought...

Whatever the front is, it would be prudent to open up as many communication mediums as possible. Handling HTTP(S), TCP, UDP or some other or custom protocol. The nodes will need to be built with a diverse toolkit to ensure they can reach the front by any means necessary.

## Nodes

### Elements

- Knows what the C2's front is, needs various ways to reach the front.
- Keeps low profile in terms of network traffic, and cpu/ram/disk usage.
- Has library of operations it can perform if instructed by the C2.
- Attempts heartbeat packet frequently.W
- Will try to always keep a UDP port open, but stops every hour or so and starts again to get a fresh port and tries to share that port with C2 and/or peers to avoid suspicion.
- Communications will either be with peers or C2. C2 comms likely to happen via hops to other peers as proxies.

### C3 Module

Command and Control Communications module. Exposes an interface for sending any arbitrary payload to the C2. It is the C3 module's responsibility to know by what means it can reach the C2 and try its best to send it the payload. It is also responsible for receiving payloads from the C2, and passing them to the appropriate module for execution. It will also need to know how to handle the payloads it receives, such as if it is a task or an update to the node's knowledge of the network or whatever else.
