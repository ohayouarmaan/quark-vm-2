something:
  PUSH 12
  STORE xavier
  RET

main:
  PUSH 0
  CALL something
  LOAD xavier
  REF
  FREE
  LOAD xavier
  DEREF
  PRINT
  CALL test
  PUSH 0
  PUSH 60
  SYSCALL 1

test:
  PUSH 2
  PRINT
  RET

