---- MODULE Exokernel ----

EXTENDS Naturals, Sequences

(* Parameters and constants *)
CONSTANT TaskCount, ResourceCount
VARIABLE tasks, resources, allocations

(* TaskId are modeled as a sequence of task IDs, where each task has an index. *)
TaskId == 1..TaskCount

(* ResourceId are modeled as a sequence of resource IDs, where each resource has an index. *)
ResourceId == 1..ResourceCount

TaskState == {"waiting", "running"}
Availability == {"free", "busy"}

(**************************************************************)
(* Type invariant of the specification                        *)
(**************************************************************)
ExokernelTypeInvariant ==
  /\ tasks \in [TaskId -> TaskState]
  /\ resources \in [ResourceId -> Availability]
  /\ allocations \in [TaskId -> [ResourceId -> BOOLEAN ] ]

----

(* Define Exokernel state as a record containing tasks, resources, and allocations *)
ExokernelInit ==
  /\ tasks = [ t \in TaskId |-> "waiting" ]
  /\ resources = [ r \in ResourceId |-> "free" ]
  /\ allocations = [ t \in TaskId |-> [ r \in ResourceId |-> FALSE ] ]

(* A task can request access to a resource if it's not already in use. *)
RequestResource(t, r) ==
  /\ resources[r] = "free"
  /\ allocations[t][r] = FALSE
  /\ tasks[t] = "waiting"
  /\ tasks' = [tasks EXCEPT ![t] = "running"]
  /\ resources' = [resources EXCEPT ![r] = "busy"]
  /\ allocations' = [allocations EXCEPT ![t][r] = TRUE]

FreeResource == CHOOSE r \in ResourceId : resources[r] = "free"
OccupyResource ==
  /\ \E r \in ResourceId : resources[r] = "free"
  /\ LET res == CHOOSE r \in ResourceId : resources[r] = "free"
     IN resources' = [ resources EXCEPT ![res] = "busy" ]
  /\ UNCHANGED <<tasks, allocations>>

(* A task can release a resource when it has finished using it. *)
ReleaseResource(t, r) ==
  /\ allocations[t][r] = TRUE
  /\ tasks[t] = "running"
  /\ tasks' = [tasks EXCEPT ![t] = "waiting"]
  /\ resources' = [resources EXCEPT ![r] = "free"]
  /\ allocations' = [allocations EXCEPT ![t][r] = FALSE]
       

(* The next-state relation defines valid state transitions *)
ExokernelNext ==
  \E t \in TaskId, r \in ResourceId : 
    (RequestResource(t, r) \/ ReleaseResource(t, r))

----

(* Specification of the overall system behavior *)
ExokernelSpec == ExokernelInit /\ [][ExokernelNext]_<<tasks, resources, allocations>>

\* vars == <<exokernel>>
\* (* Fairness: Ensure that each task eventually gets a chance to request resources *)
\* Fairness == WF_vars(TaskAction)

(* Invariant: No two tasks can hold the same resource simultaneously *)
ResourceExclusivity == 
  \A r \in ResourceId : 
    \A t1, t2 \in TaskId : (t1 # t2) => ~(allocations[t1][r] /\ allocations[t2][r])

\* ASSUME Assumption == tasks \in [TaskId -> TaskState] 

\* THEOREM Init => Fairness
\* BY DEF Init, Fairness
=============================================================================
