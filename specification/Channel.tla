---- MODULE Channel ----
(******************************************************)
(* Channel represents one-way communication primitive *)
(******************************************************)

EXTENDS Naturals, Sequences

CONSTANT Length, Data, Buffers
VARIABLE in, out
vars == <<in, out>>

ChTypeInvariant ==                           
  \A ch \in 1..Buffers: Len(in[ch]) <= Length /\ Len(out[ch]) <= Length

----

ChInit ==                                    
    /\ in  = [ch1 \in 1..Buffers |-> [ ch2 \in 1..Buffers |-> <<>>] ]
    /\ out = [ch1 \in 1..Buffers |-> [ ch2 \in 1..Buffers |-> <<>>] ]

Write(from, to, d) ==                                  
    /\ from \in 1..Buffers
    /\ to \in 1..Buffers
    /\ Len(out[from][to]) < Length                  
    /\ out' = [out EXCEPT ![from][to] = Append(out[from][to], d)]
    /\ UNCHANGED <<in>>

Read(from, to) ==                                      
    /\ from \in 1..Buffers
    /\ to \in 1..Buffers
    /\ Len(out[from][to]) # 0
    /\ Len(in[to][from]) < Length
    /\ in' = [in EXCEPT ![to][from] = Append(in[to][from], Head(out[from][to]))]
    /\ out' = [out EXCEPT ![from][to] = Tail(out[from][to])]

Handle(from, to) ==
    /\ from \in 1..Buffers
    /\ to \in 1..Buffers
    /\ Len(in[to][from]) # 0
    /\ in' = [in EXCEPT ![to][from] = Tail(in[to][from])]
    /\ UNCHANGED <<out>>

ChNext ==                           
    \E from, to \in 1..Buffers:
        \E d \in Data :
            \/ Write(from, to, d)
            \/ Read(from, to)                         
            \/ Handle(from, to)

----

ChSpec ==
    /\ ChInit
    /\ [][ChNext]_vars
    /\ WF_vars(ChNext)
    /\ \A from, to \in 1..Buffers:
        /\ \E d \in Data:
            /\ SF_vars(Write(from, to, d))
            /\ SF_vars(Read(from, to))
            /\ SF_vars(Handle(from, to))


Range(seq) == { seq[i] : i \in 1..Len(seq) }

MessageDelivery ==
    \A ch \in 1..Buffers :
        \A msg \in Data :
            (msg \in Range(in[ch])) ~> ~(msg \in Range(in[ch]))

ReadAfterWrite ==
    \A from, to \in 1..Buffers:
        \A msg \in Data:
            (msg \in Range(out[from])) => (msg \in Range(in[to]))

====
