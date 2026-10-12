---- MODULE Port ----
EXTENDS Naturals, Sequences

CONSTANT Length, Data, TasksLength, MaxTaskID
VARIABLE buffer, tasks, task_id
vars == <<buffer, tasks, task_id>>

ChTypeInvariant ==
    /\ Len(buffer) \in 0..Length 
    /\ Len(tasks) \in 0..TasksLength

TaskType ==
    [ id: Nat
    , state: {"waiting", "ready", "blocked_by", "running"}
    , priority: 1..100
    , blocked_id: Nat
    ]

TaskIdConstraint == task_id <= MaxTaskID

----

ChInit ==                                   
    /\ buffer = << >>                       
    /\ tasks = << >>
    /\ task_id = 0

Peek == IF Len(buffer) > 0 THEN Head(buffer) ELSE << >>                           

CreateTask(tasks_seq, current_id, state, priority) ==
    LET next_id   == current_id + 1
        new_task  == [id |-> next_id, state |-> state, blocked_id |-> 0, priority |-> priority]
        new_seq   == Append(tasks_seq, new_task)
    IN [seq |-> new_seq, id |-> next_id]

\* 1. Added a boolean check to see if an ID exists in the sequence
HasTaskWithID(tasks_seq, tid) ==
    \E i \in 1..Len(tasks_seq) : tasks_seq[i].id = tid

\* 2. Only call this when HasTaskWithID evaluates to TRUE
GetTaskIndexByID(tasks_seq, tid) ==
    CHOOSE i \in 1..Len(tasks_seq) : tasks_seq[i].id = tid

PhyRead ==
    /\ Len(tasks) < TasksLength
    /\ LET result == CreateTask(tasks, task_id, "waiting", 50)
       IN  /\ tasks' = result.seq
           /\ task_id' = result.id
           /\ UNCHANGED <<buffer>>

GetMinPriority(tasks_seq) ==
    CHOOSE min_val \in {tasks_seq[i].priority : i \in 1..Len(tasks_seq)} :
        \A other_val \in {tasks_seq[j].priority : j \in 1..Len(tasks_seq)} : min_val <= other_val

GetMinPriorityIndex(tasks_seq) ==
    LET waiting_tasks == {i \in 1..Len(tasks_seq) : tasks_seq[i].state = "waiting"}
    IN  IF waiting_tasks = {} THEN 0
        ELSE LET lowest_prio == CHOOSE min_val \in {tasks_seq[i].priority : i \in waiting_tasks} :
                                    \A other_val \in {tasks_seq[j].priority : j \in waiting_tasks} : min_val <= other_val
             IN  CHOOSE idx \in waiting_tasks : tasks_seq[idx].priority = lowest_prio

RunTask ==
    /\ Len(tasks) > 0
    /\ LET target_index == GetMinPriorityIndex(tasks)
       IN  /\ target_index /= 0
           /\ tasks[target_index].state = "waiting"
           /\ tasks' = [tasks EXCEPT ![target_index].state = "running"]
           /\ UNCHANGED <<buffer, task_id>>

HasBlockedTask(tasks_seq, tid) ==
    \E i \in 1..Len(tasks_seq) : tasks_seq[i].blocked_id = tid

GetTaskBlockedByID(tasks_seq, tid) ==
    CHOOSE i \in 1..Len(tasks_seq) : tasks_seq[i].blocked_id = tid

FinishTask(tid) ==
    /\ Len(tasks) > 0
    /\ HasTaskWithID(tasks, tid) \* Guard: Skip this tid entirely if it doesn't exist
    /\ LET finish_idx == GetTaskIndexByID(tasks, tid)
       IN  /\ tasks[finish_idx].state = "running"
           /\ LET unblocked_tasks == 
                    IF HasBlockedTask(tasks, tid)
                    THEN LET blocked_idx == GetTaskBlockedByID(tasks, tid)
                         IN  [tasks EXCEPT ![blocked_idx].state = "waiting", 
                                           ![blocked_idx].blocked_id = 0]
                    ELSE tasks
              IN
                tasks' = [ i \in 1..(Len(unblocked_tasks)-1) |-> 
                             IF i < finish_idx 
                             THEN unblocked_tasks[i] 
                             ELSE unblocked_tasks[i+1] 
                         ]
    /\ UNCHANGED <<buffer, task_id>>

CallTask(tid) ==
    /\ Len(tasks) < TasksLength
    /\ HasTaskWithID(tasks, tid) \* Guard: Safely verify this task ID exists in the sequence
    /\ LET \* 1. Find where this task is currently located in the sequence
           caller_idx == GetTaskIndexByID(tasks, tid)
           
           \* 2. Create the new task using the caller's actual priority
           result      == CreateTask(tasks, task_id, "waiting", tasks[caller_idx].priority)
           
           \* 3. Find the caller's index inside the NEW expanded sequence
           index       == GetTaskIndexByID(result.seq, tid)
           
           \* 4. Block the caller and link it to the newly created task's ID
           final_tasks == [result.seq EXCEPT ![index].state = "blocked_by", 
                                             ![index].blocked_id = result.id]
       IN
       /\ tasks' = final_tasks
       /\ task_id' = result.id
       /\ UNCHANGED <<buffer>>
    
----

ChNext ==
    \/ PhyRead
    \/ RunTask
    \/ \E tid \in 1..MaxTaskID:
        \/ CallTask(tid)
        \/ FinishTask(tid)

ChSpec == ChInit /\ [][ChNext]_vars /\ SF_vars(RunTask) /\ WF_vars(ChNext)

IsTaskWaiting(tid) ==
    \E i \in 1..Len(tasks) : tasks[i].id = tid /\ tasks[i].state = "waiting"

IsTaskRunning(tid) ==
    \E i \in 1..Len(tasks) : tasks[i].id = tid /\ tasks[i].state = "running"

EachTaskEventuallyRuns ==
    \A tid \in 1..MaxTaskID : IsTaskWaiting(tid) ~> IsTaskRunning(tid)

SystemProcessesTasks ==
    (Len(tasks) > 0) ~> (\E t \in 1..Len(tasks) : tasks[t].state = "running")

====
