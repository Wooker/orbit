---- MODULE Node ----

EXTENDS Naturals, Sequences

(* Parameters and constants specific to the node *)
CONSTANT Message, BufLength, TaskCount, ResourceCount
VARIABLES in, out, tasks, resources, allocations

InCh == INSTANCE Channel WITH Data <- Message, Length <- BufLength, buffer <- in
OutCh == INSTANCE Channel WITH Data <- Message, Length <- BufLength, buffer <- out
Kernel == INSTANCE Exokernel

NodeInit ==
  /\ InCh!ChInit
  /\ OutCh!ChInit
  /\ Kernel!ExokernelInit

----

HandleMessage ==
  LET m == Head(in)
  IN
  CASE m = "request" -> Kernel!OccupyResource
    [] m = "revoke" -> UNCHANGED <<out, tasks, resources, allocations>>
    [] OTHER -> UNCHANGED <<out, tasks, resources, allocations>>

----

NodeRead ==
  /\ InCh!Incoming
  /\ HandleMessage
  /\ in' = Tail(in)
  /\ UNCHANGED <<out, tasks, allocations>>

NodeWrite(d) ==
  /\ OutCh!Write(d)
  /\ UNCHANGED <<in, tasks, resources, allocations>>

NodeNext ==
  \/ NodeRead
  \/ \E d \in Message : NodeWrite(d)

----

NodeSpec == NodeInit /\ [][NodeNext]_<<in,out,tasks,resources,allocations>>

====

