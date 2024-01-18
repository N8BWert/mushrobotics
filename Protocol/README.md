# Mushrobotics Network Protocol

Below is a description of the protocol I will be implementing for the mushroom robotics project.  I'm also going to be using this document as a place to attempt to organize my thoughts around the protocol so it likely won't be super readable or understandable for the near future.  I hope to fix this once the protocol actually exists, but in the meantime I think this is probably how it will be.

## Description

The idea for the use case of this project should follow the below diagram:

![Use Case Diagram](./documentation/SimpleComplexUseCase.drawio.png)

The important thing to highlight from this document is that I'm expecting the router to pretty much be consistent.  I'm pretty convinced that this shouldn't be a problem because the raspberry pi zero running the router won't be handling a ton of traffic, however it may benefit me to add a second router but for now I will ignore this possible problem and assume there is one base station.

I would like to use the nRF24L01+ radio, which can receive on 6 pipes.  To make things easier, each node can also keep track of 2 extra node addresses to allow nodes to have 8 children making their children representable by 3 bits (or in octal mathematically).  Personally, I simply like this being a nice octal number, however 3 does not divide 1 Byte so I may come to regret this decision down the road.

Ideally, this would mean that a representation of the network wouldn't actually be a network, but instead a tree as below:

![Tree Diagram](./documentation/Tree.drawio.png)

However, this tree structure is not redundancy proof.  In fact, if any node dies, all of its children will die which will certainly be a problem given my SEDs should be battery powered.  Therefore, I (in all of my finite wisdom) have decided to use what I will call a dual tree (shown below):

![Dual Tree](./documentation/DualTree.drawio.png)

In my mind, this offers about as good as one can get in terms of both tree-like structure and a bit of redundancy.  It is also important to note, that I don't think it is particularly necessary for two nodes to have the same set of children.  It probably makes more sense for each of the either nodes to share some number of children (at least 1) to allow a bit better random redundancy but it is much harder to draw that as a picture.

I expect my router to be caching the states and addresses of each of it's children so it makes sense to abstract this storage into a n-tier storage system.  Basically, the idea is that the router can expect 8 element arrays of Nodes that contain a pointer to another block of 8 as shown below:

![N-Tier Storage Solution](./documentation/RouterMap.drawio.png)'

Ideally, this should allow me to specify addresses in terms of parent, child, grandchild, ...  Specifically, I would like to be able to say 1.2.3.4, where 1 is the parent of 2 which is the parent of 3 which is the parent of 4.  However, this can get a bit tricky because my dual-tree network shape means each node will have 2 addresses.  An incredibly simple example of this is shown below:

![Dual Address Issue](./documentation/DoubleAddressExample.drawio.png)

In this example, the right-most node can be labeled 1.1.1, 1.3.1, 2.1.1, and 2.3.1.  This ambiguity is definitely a bit difficult so I will be attempting to come up with a solution in the near future.

## Inspirations

* [OpenThread](https://openthread.io/)

#### Last Updated: January 18, 2024