%{
#include <stdio.h>
static int yylex(void);
static void yyerror(const char *message);
static int result;
%}
%token NUMBER
%left '+'
%left '*'
%%
input: expression { result = $1; };
expression: NUMBER | expression '+' expression { $$ = $1 + $3; }
                  | expression '*' expression { $$ = $1 * $3; };
%%
static int yylex(void) {
    int c;
    do { c = getchar(); } while (c == ' ' || c == '\n');
    if (c >= '0' && c <= '9') {
        yylval = c - '0';
        return NUMBER;
    }
    return c == EOF ? 0 : c;
}
static void yyerror(const char *message) { (void)message; }
int main(void) {
    if (yyparse() != 0) return 1;
    printf("%d\n", result);
    return 0;
}
