main:
  PUSH_STR "/home/sushi/projects/proton/libtest.so"
  DLL_LOAD
  STORE dllHandler

  PUSH 3
  PUSH 1
  PUSH_STR "add"
  LOAD dllHandler
  DLL_CALL 2
  DEREF
  DEBUG
  PRINT

