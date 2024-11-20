---- MODULE Net ----

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

ChannelTypeInvariant ==
 /\ \A n \in NodeID: Len(ch[n]) <= BufLength

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
 /\ Read(n)
 /\ LET m == Head(ch[n]) IN
  CASE m = "request_local" -> UNCHANGED<<ch, tasks, resources, allocations>>
    [] m = "revoke_local" -> UNCHANGED<<ch, tasks, resources, allocations>>
    [] m = "revoke_shared" -> UNCHANGED<<ch, tasks, resources, allocations>>
    [] m = "revoke_shared" -> UNCHANGED<<ch, tasks, resources, allocations>>
    [] OTHER -> UNCHANGED<<ch, tasks, resources, allocations>>

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
  \/ HandleMessage(n) \/ \E d \in Message: Write(n, d)

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
\* Fairness ==
\*  /\ 

Liveness ==
 /\ \A d \in Message: d \in {"request"} ~>
  \E r \in ResourceID: \E n \in NodeID: resources[n][r] = "busy"

====
