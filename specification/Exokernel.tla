---------------------------- MODULE Exokernel ----------------------------

EXTENDS Naturals, Sequences

(* Parameters and constants *)
CONSTANT TaskCount, ResourceCount
VARIABLE tasks, resources, allocations

(* Tasks are modeled as a sequence of task IDs, where each task has an index. *)
Tasks == 1..TaskCount

(* Resources are modeled as a sequence of resource IDs, where each resource has an index. *)
Resources == 1..ResourceCount

TaskState == {"waiting", "running"}
Availability == {TRUE, FALSE}

(**************************************************************)
(* Type invariant of the specification                        *)
(*                                                            *)
(* tasks: function mapping from Tasks to TaskState            *)
(* resources: function mapping from Resources to Availability *)
(* resources: function mapping from Resources to Availability *)
(**************************************************************)
TypeInvariant ==
  /\ tasks \in [Tasks -> TaskState]
  /\ resources \in [Resources -> Availability]
  /\ allocations \in [Tasks -> [Resources -> {TRUE, FALSE}]]

----

(* The state of the system is represented by the current allocation of resources to tasks.*)
(* allocations[i][j] = TRUE means task i is using resource j.*)
Init == 
  /\ tasks = [ t \in Tasks |-> "waiting" ] 
  /\ resources = [ r \in Resources |-> TRUE ] (* All resources are available *)
  /\ allocations = [ t \in Tasks |-> [ r \in Resources |-> FALSE ] ]

(* A task can request access to a resource if it's not already in use. *)
RequestResource(t, r) ==
  /\ resources[r] = TRUE
  /\ allocations[t][r] = FALSE
  /\ tasks[t] = "waiting"
  /\ tasks' = [tasks EXCEPT ![t] = "running"]
  /\ resources' = [resources EXCEPT ![r] = FALSE]
  /\ allocations' = [allocations EXCEPT ![t][r] = TRUE]

(* A task can release a resource when it has finished using it. *)
ReleaseResource(t, r) ==
  /\ allocations[t][r] = TRUE
  /\ tasks[t] = "running"
  /\ tasks' = [tasks EXCEPT ![t] = "waiting"]
  /\ resources' = [resources EXCEPT ![r] = TRUE]
  /\ allocations' = [allocations EXCEPT ![t][r] = FALSE]

(* Task behavior: tasks can either request or release resources *)
TaskAction == 
  \E t \in Tasks, r \in Resources : 
    (RequestResource(t, r) \/ ReleaseResource(t, r))

DoNothing == UNCHANGED <<tasks, resources, allocations>>

(* The next-state relation defines valid state transitions *)
Next == TaskAction \/ DoNothing

----

(* Specification of the overall system behavior *)
Spec == Init /\ [][Next]_<<tasks, resources, allocations>>

\* vars == <<tasks, resources>>
\* (* Fairness: Ensure that each task eventually gets a chance to request resources *)
\* Fairness == WF_vars(TaskAction)

(* Invariant: No two tasks can hold the same resource simultaneously *)
\* ResourceExclusivity == 
\*   \A r \in Resources : 
\*     \A t1, t2 \in Tasks : (t1 # t2) => ~(allocations[t1][r] /\ allocations[t2][r])

\* ASSUME Assumption == tasks \in [Tasks -> TaskState] 

\* THEOREM Init => Fairness
\* BY DEF Init, Fairness
=============================================================================

