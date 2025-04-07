ALLOC_RAW 16        ; allocates 16 bytes in the raw buffer
STORE x             ; stores the address in x
PUSH 128            ; push 128 on tos
LOAD x              ; load from constant pool variable x
PUSH 0              ; push 0 on tos
PUSH 0              ; push 0 on tos
SYSCALL 3           ; syscall tos with 3 arguments
LOAD x              ; load from const pool variable x
DEREF               ; dereference that pointer
STORE y             ; store the value in constant pool variable y
LOAD y              ; load from const pool variable y
DEREF               ; dereference that
LOAD y              ; load from const pool index 1
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
