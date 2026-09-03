---- MODULE Exokernel ----

EXTENDS Naturals, Sequences

(* Parameters and constants *)
CONSTANT TaskCount, ResourceCount, KernelCount
VARIABLE tasks, resources, allocations

Id == 1..KernelCount

(* TaskId are modeled as a sequence of task IDs, where each task has an index. *)
TaskId == 1..TaskCount

(* ResourceId are modeled as a sequence of resource IDs, where each resource has an index. *)
ResourceId == 1..ResourceCount

TaskState == {"waiting", "running", "requested"}
TaskType == [state: TaskState, request: [0..KernelCount -> 0..ResourceCount] ]
Availability == {"free", "busy"}

(**************************************************************)
(* Type invariant of the specification                        *)
(**************************************************************)
ExokernelTypeInvariant ==
  /\ tasks \in [Id -> [TaskId -> [state: TaskState, request: [Id -> 0..ResourceCount ] ] ] ]
  /\ resources \in [Id -> [ResourceId -> Availability] ]
  /\ allocations \in [Id -> [TaskId -> [ResourceId -> BOOLEAN ] ] ]

----

(* Define Exokernel state as a record containing tasks, resources, and allocations *)
ExokernelInit ==
  /\ tasks =
    [k \in Id |->
      [t \in TaskId |->
        [state |-> "waiting"
        ,request |->
          [n \in Id |-> 0
          ]
        ]
      ]
    ]
  /\ resources = [ k \in Id |-> [r \in ResourceId |-> "free" ] ]
  /\ allocations = [ k \in Id |-> [t \in TaskId |-> [ r \in ResourceId |-> FALSE ] ] ]

RunTask(k, t) ==
  /\ \A every \in TaskId: tasks[k][every].state = "waiting"
  /\ tasks' = [tasks EXCEPT ![k][t].state = "running"]
  /\ UNCHANGED <<resources, allocations>>

WaitTask(k, t) ==
  /\ \E every \in TaskId: tasks[k][every].state = "running"
  /\ tasks' = [tasks EXCEPT ![k][t].state = "waiting"]
  /\ UNCHANGED <<resources, allocations>>

(* A task can request access to a resource if it's not already in use. *)
RequestResource(from, to, t, r) ==
  /\ tasks[from][t].state = "running"
  /\ tasks' = [tasks EXCEPT ![from][t].state = "requested", ![from][t].request[to] = r]
  /\ UNCHANGED <<resources, allocations>>

RequestFail(from, to, t, r) ==
  /\ tasks[from][t].state = "requested"
  /\ tasks' = [tasks EXCEPT ![from][t].state = "running", ![from][t].request[to] = 0]
  /\ UNCHANGED <<resources, allocations>>

OccupyResource(from, to, t, r) ==
  /\ resources[from][r] = "free"
  /\ allocations[from][t][r] = FALSE
  /\ tasks[from][t].state = "requested"
  /\ tasks[from][t].request[to] = r
  /\ resources' = [resources EXCEPT ![to][r] = "busy"]
  /\ allocations' = [allocations EXCEPT ![from][t][r] = TRUE]
  /\ tasks' = [tasks EXCEPT ![from][t].state = "running", ![from][t].request[to] = 0]

(* A task can release a resource when it has finished using it. *)
ReleaseResource(k, t, r) ==
  /\ allocations[k][t][r] = TRUE
  /\ tasks[k][t].state = "running"
  /\ tasks' = [tasks EXCEPT ![k][t].state = "waiting"]
  /\ resources' = [resources EXCEPT ![k][r] = "free"]
  /\ allocations' = [allocations EXCEPT ![k][t][r] = FALSE]
       
----

(* The next-state relation defines valid state transitions *)
ExokernelNext ==
  \E from, to \in Id, t \in TaskId: 
    \/ RunTask(from, t)
    \/ WaitTask(from, t)
    \/ \E r \in ResourceId:
       \/ RequestResource(from, to, t, r)
       \/ OccupyResource(from, to, t, r)
       \/ RequestFail(from, to, t, t)
       \/ ReleaseResource(from, t, r)

----

(* Specification of the overall system behavior *)
ExokernelSpec == ExokernelInit /\ [][ExokernelNext]_<<tasks, resources, allocations>>

=============================================================================
