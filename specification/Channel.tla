---- MODULE Channel ----
(******************************************************)
(* Channel represents one-way communication primitive *)
(******************************************************)

EXTENDS Naturals, Sequences

CONSTANT Length, Data
VARIABLE buffer

ChTypeInvariant == Len(buffer) \in 0..Length (**********************************)
                                             (* Type invariant for the channel *)
----

ChInit ==                                   (**************************************)
    /\ buffer = << >>                       (* Create buffer as an empty sequence *)

Peek == IF Len(buffer) > 0 THEN Head(buffer) ELSE << >>                           (*****************************)
                                               (* Peek data from the buffer *)

Incoming == Len(buffer) > 0

Read ==                                     (*****************************)
    /\ Len(buffer) > 0                      (* Read data from the buffer *)
    /\ buffer' = Tail(buffer)

Write(d) ==                                 (****************************)
    /\ Len(buffer) < Length                 (* Write data to the buffer *)
    /\ buffer' = Append(buffer, d)

ChNext ==                           (***********************************************)
    \/ Read                         (* Next state is represented by either reading *)  
    \/ \E d \in Data : Write(d)     (* from the channel or writing to it           *)

----

ChSpec == ChInit /\ [][ChNext]_<<buffer>>
    
====
