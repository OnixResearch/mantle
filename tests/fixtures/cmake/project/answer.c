#include "answer.h"
extern int cpp_answer(void);
int answer(void) { return 40 + cpp_answer() + FIXTURE_OFFSET; }
