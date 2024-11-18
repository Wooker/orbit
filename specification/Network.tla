---- MODULE Network ----

EXTENDS Naturals, Sequences

(* Constants *)
CONSTANT NodeCount, Message, BufLength, TaskCount, ResourceCount

(* Variables *)
VARIABLES nodes

(* Instantiate the Node module for a single node *)
Node == INSTANCE Node WITH
  Message <- Message,
  BufLength <- BufLength,
  TaskCount <- TaskCount,
  ResourceCount <- ResourceCount,
  in <- nodes.in,
  out <- nodes.out,
  tasks <- nodes.tasks,
  resources <- nodes.resources,
  allocations <- nodes.allocations

(* Initial state of a single node *)
InitialNodeState ==
    [ in          |-> [ buffer |-> <<>> ]
    , out         |-> <<>>
    , tasks       |-> {}
    , resources   |-> {"CPU", "RAM"}
    , allocations |-> {}
    ]

(* Initialize nodes dynamically *)
NetworkInit ==
    /\ nodes = [n \in 1..NodeCount |-> InitialNodeState]

(* Actions for a single node *)
NodeStep(n) ==
    LET node == nodes[n] IN
    \/ Node!NodeRead
    \/ \E d \in Message : Node!NodeWrite(d)

(* Next state relation for the network *)
NetworkNext ==
    /\ \E n \in 1..NodeCount : NodeStep(n)
    \* /\ UNCHANGED <<nodes>>

(* Specification *)
NetworkSpec ==
  /\ NetworkInit
  /\ [][NetworkNext]_nodes

==== 
