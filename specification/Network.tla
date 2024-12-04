---- MODULE Network ----

EXTENDS Naturals, Sequences, TLC

CONSTANTS NodeCount, TaskCount, ResourceCount, BufLength, Message
VARIABLES ch, tasks, resources, allocations

NodeID == 1..NodeCount
TaskID == 1..TaskCount
ResourceID == 1..ResourceCount

TaskState == {"waiting", "running"}
Availability == {"free", "busy"}

----
(***)
(* Type invariants *)
(***)

TypeInvariant ==
 /\ ch \in [NodeID -> Seq(Message)] 
 /\ tasks \in [NodeID -> [TaskID -> TaskState]]                       
 /\ resources \in [NodeID -> [ResourceID -> Availability]]            
 /\ allocations \in [NodeID -> [TaskID -> [ResourceID -> BOOLEAN ] ]] 

----
(***)
(* Exokernel states *)
(***)

RequestResource(n, t, r) ==
 /\ tasks[n][t] = "waiting"
 /\ resources[n][r] = "free"
 /\ allocations[n][t][r] = FALSE
 /\ tasks' = [ tasks EXCEPT ![n][t] = "running" ]
 /\ resources' = [ resources EXCEPT ![n][r] = "busy" ]
 /\ allocations' = [ allocations EXCEPT ![n][t][r] = TRUE ]

RevokeResource(n, t, r) ==
 /\ tasks[n][t] = "running"
 /\ resources[n][r] = "busy"
 /\ allocations[n][t][r] = TRUE
 /\ tasks' = [ tasks EXCEPT ![n][t] = "waiting" ]
 /\ resources' = [ resources EXCEPT ![n][r] = "free" ]
 /\ allocations' = [ allocations EXCEPT ![n][t][r] = FALSE ]

----
(***)
(* Channel states *)
(***)

Read(n) ==
  /\ Len(ch[n]) > 0
  /\ LET m == Head(ch[n]) IN
       CASE 
         m = "request" ->
           \E t \in TaskID, r \in ResourceID:
            /\ RequestResource(n, t, r)
            /\ ch' = [ch EXCEPT ![n] = Tail(@)]

         [] m = "revoke" -> 
           \E t \in TaskID, r \in ResourceID:
            /\ RevokeResource(n, t, r)
            /\ ch' = [ch EXCEPT ![n] = Tail(@)]

         [] OTHER -> (* read the message and do nothing *)
           /\ ch' = [ch EXCEPT ![n] = Tail(@)]

Write(n, d) ==
 /\ IF d = "request" THEN \E r \in ResourceID: resources[n][r] = "free" ELSE TRUE
 /\ IF d = "revoke" THEN \E r \in ResourceID: resources[n][r] = "busy" ELSE TRUE
 /\ Len(ch[n]) < BufLength
 /\ ch' = [ ch EXCEPT ![n] = Append(@, d) ]
 /\ UNCHANGED <<tasks, resources, allocations>>

----

Init ==
 /\ ch = [ n \in NodeID |-> << >> ]
 /\ tasks = [ n \in NodeID |-> [ t \in TaskID |-> "waiting" ] ]
 /\ resources = [ n \in NodeID |-> [ r \in ResourceID |-> "free" ] ]
 /\ allocations = [ n \in NodeID |-> [ t \in TaskID |-> [ r \in ResourceID |-> FALSE ] ] ]


Next ==
 \E n \in NodeID:
  \/ Read(n)
  \/ \E d \in Message: Write(n, d)

----
(***)
(* Safety properties *)
(***)

(* Ensures that resources are exclusively allocated to one task at a time within a given node. *)
ExclusiveAllocationSafety ==
\A n \in NodeID, r \in ResourceID:
  \E t \in TaskID:
    allocations[n][t][r] = TRUE => \A tn \in TaskID \ {t}: allocations[n][tn][r] = FALSE

(***)
(* Ensures that the message queue (or buffer) for each node *)
(* does not exceed its predefined maximum capacity.*)
(***)
ChannelBufferSafety ==
\A n \in NodeID: Len(ch[n]) <= BufLength

(* Ensures that a resource marked as "free" is not allocated to any task. *)
ResourceStateConsistencySafety ==
\A n \in NodeID, r \in ResourceID:
  resources[n][r] = "free" => \A t \in TaskID: allocations[n][t][r] = FALSE

(* Ensures that a task in the "running" state has at least one resource allocated to it. *)
TaskStateConsistencySafety ==
\A n \in NodeID, t \in TaskID:
  tasks[n][t] = "running" => \E r \in ResourceID: allocations[n][t][r] = TRUE

----

(***)
(* Fairness *)
(***)

vars == <<ch, tasks, resources, allocations>>

(***)
(* Ensures that tasks waiting for free resources will *)
(* eventually be considered for resource allocation.*)
(***)
FairResourceAllocation ==
 WF_vars(\E n \in NodeID, t \in TaskID, r \in ResourceID:
  tasks[n][t] = "waiting" /\ resources[n][r] = "free")

(***)
(* Guarantees that each node will eventually execute one of its *)
(* possible actions, such as requesting or revoking resources,  *)
(* handling messages, or writing to the buffer.                 *)
(***)
FairNodeSteps ==
 SF_vars(\E n \in NodeID:
  \E t \in TaskID, r \in ResourceID, d \in Message: 
   \/ RequestResource(n, t, r)
   \/ RevokeResource(n, t, r)
   \/ Read(n)
   \/ Write(n, d))
            
(***)
(* Ensures that nodes will eventually have space available *)
(* in their message buffer to send new messages. *)
(***)
FairBufferSpace ==
 SF_vars(\E n \in NodeID, d \in Message:
  Len(ch[n]) < BufLength)

----

(* Ensures that every message written to a channel leads to reading it *)
MessageReadLiveness ==
  \A n \in NodeID: \A d \in Message: (d \in ch[n]) ~> (d \notin ch[n])

----

Spec ==
 /\ Init /\ [][Next]_<<ch, tasks, resources, allocations>>
 /\ FairNodeSteps
 /\ FairResourceAllocation
 /\ FairBufferSpace

====
