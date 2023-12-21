# Stages and Flow of pBFT:
1. Request Propagation Stage:
Event: Client sends a request to the primary (leader) node.
Description: This stage begins when a client initiates a request and sends it to the primary node in the pBFT network.

2. Pre-Prepare Stage:
Events:
Primary (Leader) Node: Receives the client request and creates a pre-prepare message containing the proposed request.
Other Nodes: Receive the pre-prepare message from the primary node.
Description: In this stage, the primary node proposes the request by creating a pre-prepare message containing the proposed request and disseminates it to all other nodes in the network.

3. Prepare Stage:
Events:
Nodes: Receive the pre-prepare message and validate the request's correctness. If valid, nodes broadcast a prepare message to all nodes.
Description: Upon receiving the pre-prepare message, other nodes validate the request's correctness and broadcast a prepare message, signaling their acceptance of the proposed request.

4. Commit Stage:
Events:
Nodes: Once a node collects prepare messages from a two-thirds majority of nodes (including itself), it broadcasts a commit message to all nodes.
Description: Nodes collect prepare messages from a sufficient number of nodes and, upon validation, send a commit message, finalizing their commitment to the proposed request.

5. Response Stage:
Events:
Primary (Leader) Node: Collects commit messages from a two-thirds majority of nodes and sends a response to the client.
Client: Receives the response from the primary node.
Description: The primary node collects commit messages from a sufficient number of nodes, validates the consensus, and sends a response to the client, acknowledging the successful processing of the request.

## Flow Summary:

* Request Propagation: Client sends a request to the primary node.
* Pre-Prepare: Primary node proposes the request and sends a pre-prepare message to other nodes.
* Prepare: Nodes validate the request and send prepare messages upon agreement.
* Commit: Nodes broadcast commit messages after collecting enough prepare messages.
* Response: Primary node confirms consensus and sends a response to the client.

Throughout these stages, nodes exchange messages to propose, validate, and commit to the request, ensuring agreement despite the potential presence of faulty or malicious nodes. The algorithm guarantees safety and finality when a request is committed, ensuring consistency in the distributed network.