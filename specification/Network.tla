---- MODULE Network ----

EXTENDS Naturals, Sequences

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
 /\ tasks \in [NodeID -> [TaskID -> TaskState]]                       (*****************)
 /\ resources \in [NodeID -> [ResourceID -> Availability]]            (* shtsathshtnhe *)
 /\ allocations \in [NodeID -> [TaskID -> [ResourceID -> BOOLEAN ] ]] (*****************)

----
(***)
(* Channel states *)
(***)

Read(n) ==
 /\ Len(ch[n]) > 0
 /\ ch' = [ ch EXCEPT ![n] = Tail(@) ]
 /\ UNCHANGED <<tasks, resources, allocations>>

Write(n, d) ==
 /\ IF d = "revoke" THEN \E r \in ResourceID: resources[n][r] = "busy" ELSE TRUE
 /\ Len(ch[n]) < BufLength
 /\ ch' = [ ch EXCEPT ![n] = Append(@, d) ]
 /\ UNCHANGED <<tasks, resources, allocations>>

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
 /\ UNCHANGED <<ch>>

RevokeResource(n, t, r) ==
 /\ tasks[n][t] = "running"
 /\ resources[n][r] = "busy"
 /\ allocations[n][t][r] = TRUE
 /\ tasks' = [ tasks EXCEPT ![n][t] = "waiting" ]
 /\ resources' = [ resources EXCEPT ![n][r] = "free" ]
 /\ allocations' = [ allocations EXCEPT ![n][t][r] = FALSE ]
 /\ UNCHANGED <<ch>>

----
(***)
(* Message Handling *)
(***)

HandleMessage(n) ==
  /\ Read(n)  \* Ensure that the channel is non-empty
  /\ LET m == Head(ch[n]) IN
       CASE 
         m = "request" ->
           \E t \in TaskID, r \in ResourceID:
             /\ tasks[n][t] = "waiting"
             /\ resources[n][r] = "free"
             /\ allocations[n][t][r] = FALSE
             /\ tasks' = [tasks EXCEPT ![n][t] = "running"]
             /\ resources' = [resources EXCEPT ![n][r] = "busy"]
             /\ allocations' = [allocations EXCEPT ![n][t][r] = TRUE]
             /\ ch' = [ch EXCEPT ![n] = Tail(@)] \* Remove the processed message

         [] m = "revoke" ->
           \E t \in TaskID, r \in ResourceID:
             /\ tasks[n][t] = "running"
             /\ resources[n][r] = "busy"
             /\ allocations[n][t][r] = TRUE
             /\ tasks' = [tasks EXCEPT ![n][t] = "waiting"]
             /\ resources' = [resources EXCEPT ![n][r] = "free"]
             /\ allocations' = [allocations EXCEPT ![n][t][r] = FALSE]
             /\ ch' = [ch EXCEPT ![n] = Tail(@)] \* Remove the processed message

         [] OTHER ->
           /\ ch' = [ch EXCEPT ![n] = Tail(@)] \* Remove the message if it doesn't match expected types
           /\ UNCHANGED <<tasks, resources, allocations>>  \* No changes to other state variables

----
(***)
(* Node step *)
(***)

\* NodeStep(n) ==

----

Init ==
 /\ ch = [ n \in NodeID |-> << >> ]
 /\ tasks = [ n \in NodeID |-> [ t \in TaskID |-> "waiting" ] ]
 /\ resources = [ n \in NodeID |-> [ r \in ResourceID |-> "free" ] ]
 /\ allocations = [ n \in NodeID |-> [ t \in TaskID |-> [ r \in ResourceID |-> FALSE ] ] ]


Next ==
 \E n \in NodeID:
  \/ (\E t \in TaskID: \E r \in ResourceID: RequestResource(n, t, r) \/ RevokeResource(n, t, r))
  \/ HandleMessage(n)
  \/ \E d \in Message: Write(n, d)

\* UNCHANGED<<in, out, tasks, resources, allocations>>

----
(***)
(* Safety and Liveness *)
(***)
Reads ==
 \A n \in NodeID:
  LET L == Len(ch[n]) IN
   LET LNext == Len(ch[n]') IN
    (L > 0) => ((L < LNext) \/ (L > LNext))

ReadsIncoming == [][Reads]_<<ch>>

----

(***)
(* Safety properties *)
(***)

(* Ensures that resources are exclusively allocated to one task at a time within a given node. *)
ExclusiveAllocationSafety ==
\A n \in NodeID, r \in ResourceID:
  \E t \in TaskID: allocations[n][t][r] = TRUE => \A tn \in TaskID \ {t}: allocations[n][tn][r] = FALSE

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

FairResourceAllocation ==
  WF_vars(\E n \in NodeID, t \in TaskID, r \in ResourceID:
            tasks[n][t] = "waiting" /\ resources[n][r] = "free")

FairNodeSteps ==
  SF_vars(\E n \in NodeID:
            \E t \in TaskID, r \in ResourceID, d \in Message: 
              RequestResource(n, t, r) \/ RevokeResource(n, t, r) \/ HandleMessage(n) \/ Write(n, d))
            
FairBufferSpace ==
  SF_vars(\E n \in NodeID, d \in Message:
            Len(ch[n]) < BufLength)

----

Spec ==
 /\ Init /\ [][Next]_<<ch, tasks, resources, allocations>>
 /\ FairNodeSteps
 /\ FairResourceAllocation
 /\ FairBufferSpace

====
