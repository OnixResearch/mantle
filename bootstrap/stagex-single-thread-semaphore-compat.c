#include <errno.h>
#include <limits.h>
#include <semaphore.h>

int sem_init(sem_t *sem, int pshared, unsigned value) {
  if (sem == 0) { errno = EINVAL; return -1; }
  if (pshared != 0) { errno = EINVAL; return -1; }
  if (value > INT_MAX) { errno = EINVAL; return -1; }
  sem->__val[0] = (int)value;
  return 0;
}

int sem_destroy(sem_t *sem) {
  if (sem == 0) { errno = EINVAL; return -1; }
  return 0;
}

int sem_post(sem_t *sem) {
  if (sem == 0) { errno = EINVAL; return -1; }
  if (sem->__val[0] == INT_MAX) { errno = EOVERFLOW; return -1; }
  sem->__val[0] += 1;
  return 0;
}

int sem_wait(sem_t *sem) {
  if (sem == 0) { errno = EINVAL; return -1; }
  if (sem->__val[0] <= 0) { errno = EAGAIN; return -1; }
  sem->__val[0] -= 1;
  return 0;
}
