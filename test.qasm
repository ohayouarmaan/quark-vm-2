ALLOC_RAW 16        ; allocates 16 bytes in the raw buffer
STORE 0             ; store the pointer of that in constant pool index 0
PUSH 128            ; push 128 on tos
LOAD 0              ; load from constant pool index 0
PUSH 0              ; push 0 on tos
PUSH 0              ; push 0 on tos
SYSCALL 3           ; syscall tos with 3 arguments
LOAD 0              ; load from const pool index 0
DEREF               ; dereference that pointer
STORE 1             ; store the value in constant pool index 1
LOAD 1              ; load from const pool index 1
DEREF               ; dereference that
LOAD 1              ; load from const pool index 1
PUSH 1              ; push 1 on tos
ADD                 ; add
DEREF               ; dereference ptr + 1
PUSH_STR "wow"      ; allocates 3 word in heap and push wow in heap and puts ptr on tos
DEREF               ; dereference the string pointer -> w
PRINT               ; should print w
REF                 ; reference the first letter w
DEREF               ; dereference 
DEREF               ; dereference 
PRINT               ; prints it
DEBUG               ; prints the stack and other info
