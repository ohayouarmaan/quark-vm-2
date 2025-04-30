printBuffer:
  LOAD allocatedMemory
  LOAD numOfBytesDup
  LOAD numOfBytes
  SUB
  ADD
  DEREF
  PRINT
  LOAD numOfBytes
  PUSH 1
  SUB
  STORE numOfBytes
  LOAD numOfBytes
  JMPNZ printBuffer
  RET


main:
  PUSH_STR "/home/sushi/install.log"
  PUSH 1
  STD_SYSCALL 1
  STORE fd

  PUSH 10
  LOAD fd
  ALLOC_RAW 10
  STORE rawMemory
  LOAD rawMemory
  PUSH 0
  PUSH 2
  STD_SYSCALL 4

  LOAD rawMemory
  DEREF
  STORE allocatedMemory

  PUSH 10
  STORE numOfBytes
  PUSH 10
  STORE numOfBytesDup
  CALL printBuffer
  DEBUG
