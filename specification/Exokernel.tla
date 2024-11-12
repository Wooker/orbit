---- MODULE Exokernel ----

EXTENDS Naturals, Sequences

(* Parameters and constants *)
CONSTANT TaskCount, ResourceCount
VARIABLE exokernel

(* TaskId are modeled as a sequence of task IDs, where each task has an index. *)
TaskId == 1..TaskCount

(* ResourceId are modeled as a sequence of resource IDs, where each resource has an index. *)
ResourceId == 1..ResourceCount

TaskState == {"waiting", "running"}
Availability == {"free", "busy"}

(**************************************************************)
(* Type invariant of the specification                        *)
(*                                                            *)
(* exokernel: record with fields tasks, resources, and allocations *)
(**************************************************************)
ExokernelTypeInvariant ==
  /\ exokernel.tasks \in [TaskId -> TaskState]
  /\ exokernel.resources \in [ResourceId -> Availability]
  /\ exokernel.allocations \in [TaskId -> [ResourceId -> BOOLEAN ] ]

----

(* Define Exokernel state as a record containing tasks, resources, and allocations *)
ExokernelInit ==
  exokernel = [
    tasks |-> [ t \in TaskId |-> "waiting" ],
    resources |-> [ r \in ResourceId |-> "free" ],
    allocations |-> [ t \in TaskId |-> [ r \in ResourceId |-> FALSE ] ]
  ]

(* A task can request access to a resource if it's not already in use. *)
RequestResource(t, r) ==
  /\ exokernel.resources[r] = "free"
  /\ exokernel.allocations[t][r] = FALSE
  /\ exokernel.tasks[t] = "waiting"
  /\ exokernel' = 
       [ tasks |-> [exokernel.tasks EXCEPT ![t] = "running"],
          resources |-> [exokernel.resources EXCEPT ![r] = "busy"],
          allocations |-> [exokernel.allocations EXCEPT ![t][r] = TRUE]
       ]

(* A task can release a resource when it has finished using it. *)
ReleaseResource(t, r) ==
  /\ exokernel.allocations[t][r] = TRUE
  /\ exokernel.tasks[t] = "running"
  /\ exokernel' = 
       [ tasks |-> [exokernel.tasks EXCEPT ![t] = "waiting"],
          resources |-> [exokernel.resources EXCEPT ![r] = "free"],
          allocations |-> [exokernel.allocations EXCEPT ![t][r] = FALSE]
       ]

(* The next-state relation defines valid state transitions *)
ExokernelNext ==
  \E t \in TaskId, r \in ResourceId : 
    (RequestResource(t, r) \/ ReleaseResource(t, r))

----

(* Specification of the overall system behavior *)
ExokernelSpec == ExokernelInit /\ [][ExokernelNext]_exokernel

\* vars == <<exokernel>>
\* (* Fairness: Ensure that each task eventually gets a chance to request resources *)
\* Fairness == WF_vars(TaskAction)

(* Invariant: No two tasks can hold the same resource simultaneously *)
ResourceExclusivity == 
  \A r \in ResourceId : 
    \A t1, t2 \in TaskId : (t1 # t2) => ~(exokernel.allocations[t1][r] /\ exokernel.allocations[t2][r])

\* ASSUME Assumption == exokernel.tasks \in [TaskId -> TaskState] 

\* THEOREM Init => Fairness
\* BY DEF Init, Fairness
=============================================================================
