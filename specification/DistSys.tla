---- MODULE DistSys ----
EXTENDS Exokernel, Sequences, Naturals

(* -- Constants -- *)
CONSTANT NodeId

(* -- Variables -- *)
VARIABLE nodes, channel


(* -- Types and Initial State -- *)

(* Each Node has an Exokernel managing its tasks and resources *)
Node == [ id: NodeId, tasks: TaskId]

(* Channel simulates a message queue between nodes for communication *)
Message == [
 from: NodeId,
 to: NodeId,
 taskId: TaskId,
 resourceId: ResourceId,
 type: {"request", "grant", "release"}
]

(* Initial state *)
DistSysInit ==
    /\ exokernelState = <<>>
    /\ nodes = [n \in NodeId |-> [id |-> n, taskId |-> [t \in TaskId |-> "waiting"], resourceId |-> [r \in ResourceId |-> TRUE] ] ]
    \* /\ nodes = [n \in NodeId |-> [id \in NodeId |-> n, tasks |-> [task \in TaskId ]] ]
    /\ channel = <<>>

(* -- Communication Actions -- *)

(* Node sends a request for a resource to another node *)
NodeRequestResource(from, to, taskId, resourceId) ==
    /\ nodes[from].tasks[taskId] = "waiting"
    /\ channel' = Append(channel, [from |-> from, to |-> to, taskId |-> taskId, resourceId |-> resourceId, type |-> "request"])
    /\ UNCHANGED nodes

(* Node grants a resource to another node's task *)
NodeGrantResource (from, to, taskId, resourceId) ==
    /\ \E msg \in channel :
         /\ msg.type = "request"
         /\ msg.to = from
         /\ msg.from = to
         /\ msg.taskId = taskId
         /\ msg.resourceId = resourceId
    /\ nodes[from].resources[resourceId] = TRUE
    /\ channel' = channel \ { [from |-> from, to |-> to, taskId |-> taskId, resourceId |-> resourceId, type |-> "request"] }
    /\ nodes' = [nodes EXCEPT 
                 ![to].tasks[taskId].status = "using",
                 ![from].resources[resourceId] = FALSE,
                 ![to].permissions[<<taskId, resourceId>>] = TRUE]

(* Node releases a resource back to its owner *)
NodeReleaseResource(from, to, taskId, resourceId) ==
    /\ nodes[to].tasks[taskId].status = "using"
    /\ channel' = Append(channel, [from |-> from, to |-> to, taskId |-> taskId, resourceId |-> resourceId, type |-> "release"])
    /\ nodes' = [nodes EXCEPT ![to].tasks[taskId].status = "idle",
                          ![to].resources[resourceId] = TRUE]

(* -- Next-state relation -- *)
DistSysNext ==
    \/ \E from, to \in NodeId, taskId \in TaskId, resourceId \in ResourceId : NodeRequestResource(from, to, taskId, resourceId)
    \/ \E from, to \in NodeId, taskId \in TaskId, resourceId \in ResourceId : NodeGrantResource(from, to, taskId, resourceId)
    \/ \E from, to \in NodeId, taskId \in TaskId, resourceId \in ResourceId : NodeReleaseResource(from, to, taskId, resourceId)

(* -- Invariants -- *)

(* Isolation across nodes: No two tasks in different nodes can use the same resource simultaneously *)
\* NodeIsolation ==
\*     \A n1, n2 \in NodeId, r \in ResourceId :
\*         (n1 # n2) => Cardinality({t \in TaskId : nodes[n1].tasks[t].status = "using" /\ nodes[n1].permissions[<<t, r>>]}) = 0

(* -- Specification -- *)
DistSysSpec == DistSysInit /\ [][DistSysNext]_<<nodes, channel>>

====
