---- MODULE Node ----

EXTENDS Exokernel, Naturals, Sequences

(* Parameters and constants specific to the node *)
CONSTANT NumNodes
VARIABLE nodes

(* NodeId represents a unique identifier for each node. *)
NodeId == 1..NumNodes

(**************************************************************)
(* NodeState: record containing a local instance of Exokernel *)
(*            and information on node status.                 *)
(**************************************************************)
NodeState ==
  [ exokernel: ExokernelTypeInvariant,
    status: {"active", "inactive"} 
  ]

(* Type invariant for nodes *)
NodeTypeInvariant ==
  /\ nodes \in [NodeId -> NodeState]
  /\ \A n \in NodeId : ExokernelTypeInvariant

(* Initialize each node with its own Exokernel instance and set status to active *)
NodeInit ==
  nodes = [ n \in NodeId |-> [ exokernel |-> ExokernelInit, status |-> "active" ] ]

(* A node can request a resource locally or communicate with other nodes for remote resources *)
NodeRequestResource(n, t, r) ==
  /\ n \in NodeId
  /\ t \in TaskId
  /\ r \in ResourceId
  /\ nodes[n].exokernel.resources[r] = "free"
  /\ nodes' = [nodes EXCEPT ![n].exokernel.tasks[t] = "running",
                               ![n].exokernel.resources[r] = "busy",
                               ![n].exokernel.allocations[t][r] = TRUE ]

(* A node can release a resource it holds *)
NodeReleaseResource(n, t, r) ==
  /\ n \in NodeId
  /\ t \in TaskId
  /\ r \in ResourceId
  /\ nodes[n].exokernel.allocations[t][r] = TRUE
  /\ nodes' = [nodes EXCEPT ![n].exokernel.tasks[t] = "waiting",
                               ![n].exokernel.resources[r] = "free",
                               ![n].exokernel.allocations[t][r] = FALSE ]

(* Define valid state transitions for nodes *)
NodeNext ==
  \E n \in NodeId, t \in TaskId, r \in ResourceId : 
    (NodeRequestResource(n, t, r) \/ NodeReleaseResource(n, t, r))

(* Overall specification of node behavior *)
NodeSpec == NodeInit /\ [][NodeNext]_nodes

(* Invariant for resource exclusivity across all nodes *)
DistributedResourceExclusivity ==
  \A n1, n2 \in NodeId : n1 /= n2 =>
    \A r \in ResourceId : 
      ~(\E t1 \in TaskId : nodes[n1].exokernel.allocations[t1][r]) /\
      ~(\E t2 \in TaskId : nodes[n2].exokernel.allocations[t2][r])

====

