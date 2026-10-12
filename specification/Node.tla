---- MODULE Node ----

EXTENDS Naturals, Sequences

(* Parameters and constants specific to the node *)
CONSTANT Message, Reply, BufLength, TaskCount, ResourceCount, NodeCount
VARIABLES msg, in, out, tasks, resources, allocations
vars == <<msg, in, out, tasks, resources, allocations>>

Id == 1..NodeCount

Channels == INSTANCE Channel WITH Data <- Message \union Reply, Length <- BufLength, Buffers <- NodeCount
Kernel == INSTANCE Exokernel WITH KernelCount <- NodeCount

NodeInit ==
  /\ Channels!ChInit
  /\ Kernel!ExokernelInit
  /\ msg = [ m \in Id |-> "" ]

----

NodeReply(from, to) ==
  /\ msg[from] # ""
  /\ Channels!Write(from, to, msg[from])
  /\ msg' = [msg EXCEPT ![from] = ""]
  /\ UNCHANGED <<in, tasks, resources, allocations>>

NodeWrite(from, to, d) ==
  /\ msg[from] = ""
  /\ CASE d = "request"
          -> \E t \in Kernel!TaskId, r \in Kernel!ResourceId:
              allocations[to][t][r] = FALSE
       [] d = "revoke"
          -> \E t \in Kernel!TaskId, r \in Kernel!ResourceId:
              allocations[to][t][r] = TRUE
       [] OTHER
          -> FALSE
  /\ \E t \in Kernel!TaskId, r \in Kernel!ResourceId: Kernel!RequestResource(from, to, t, r)
  /\ Channels!Write(from, to, d)
  /\ UNCHANGED <<msg, resources, allocations>>

HandleMessage(from, to) ==
  /\ Len(in[to][from]) # 0
  /\ LET m == Head(in[to][from])
     IN
     /\ in' = [in EXCEPT ![to][from] = Tail(in[to][from])]
     /\ CASE m = "request"
            -> \E t \in Kernel!TaskId, r \in Kernel!ResourceId:
                 /\ Kernel!RequestResource(from, to, t, r)
                 /\ \/ /\ Kernel!OccupyResource(from, to, t, r)
                       /\ msg' = [msg EXCEPT ![to] = "ok"]
                       /\ UNCHANGED <<out>>
                    \/ /\ ~ Kernel!OccupyResource(from, to, t, r)
                       /\ msg' = [msg EXCEPT ![to] = "err"]
                       /\ UNCHANGED <<out, resources, allocations>>
    
         [] m = "revoke"
            -> \E k \in Id, t \in Kernel!TaskId:
                 \E r \in Kernel!ResourceId:
                   /\ Kernel!ReleaseResource(k, t, r)
                   /\ msg' = [msg EXCEPT ![to] = "ok"]
                   /\ UNCHANGED <<out>>
         [] OTHER
            -> /\ UNCHANGED <<msg, in, out, tasks, resources, allocations>>

NodeRead(from, to) ==
  /\ Channels!Read(from, to)
  /\ UNCHANGED <<msg, tasks, resources, allocations>>

NodeNext ==
  \/ /\ UNCHANGED <<msg, in, out>>
     /\ \/ \E from, to \in Id, t \in Kernel!TaskId, r \in Kernel!ResourceId:
           \/ Kernel!RunTask(from, t)
           \/ Kernel!WaitTask(from, t)
           \/ Kernel!RequestResource(from, to, t, r)
           \/ Kernel!RequestFail(from, to, t, r)
           \/ Kernel!OccupyResource(from, to, t, r)
           \/ Kernel!ReleaseResource(from, t, r)
  \/ \E from, to \in Id, m \in Message :
      \/ NodeRead(from, to)
      \/ HandleMessage(from, to)
      \/ NodeWrite(from, to, m)

----

TaskResourceAllocationConsistency ==
  \E k \in Id, t \in Kernel!TaskId, r \in Kernel!ResourceId:
    \/ /\ allocations[k][t][r] = TRUE
       /\ resources[k][r] = "busy"
    \/ /\ allocations[k][t][r] = FALSE
       /\ resources[k][r] = "free"

----

NodeSpec ==
  /\ NodeInit
  /\ [][NodeNext]_<<vars>>
  /\ WF_vars(NodeNext)
  /\ \E from, to \in Id, t \in Kernel!TaskId, r \in Kernel!ResourceId:
     /\ SF_vars(Kernel!RequestResource(from, to, t, r))
        \/ /\ SF_vars(Kernel!OccupyResource(from, to, t, r))
           /\ SF_vars(Kernel!RequestFail(from, to, t, r))

====

